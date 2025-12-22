// src/services/cdr_generator.rs
use crate::database::DbPool;
use crate::services::ReservationManager;
use crate::models::ConsumeReservationRequest;
use crate::error::BillingError;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use tracing::{info, warn, error};

pub struct CdrGenerator {
    db_pool: DbPool,
    reservation_mgr: Arc<ReservationManager>,
}

#[derive(Debug, Clone)]
pub struct HangupEvent {
    pub uuid: String,
    pub caller: String,
    pub callee: String,
    pub start_time: DateTime<Utc>,
    pub answer_time: Option<DateTime<Utc>>,
    pub end_time: DateTime<Utc>,
    pub duration: i32,
    pub billsec: i32,
    pub hangup_cause: String,
    pub direction: String,
    pub server_id: String,
}

impl CdrGenerator {
    pub fn new(db_pool: DbPool, reservation_mgr: Arc<ReservationManager>) -> Self {
        Self {
            db_pool,
            reservation_mgr,
        }
    }

    pub async fn generate_cdr(&self, event: HangupEvent) -> Result<i64, BillingError> {
        info!("📝 Generating CDR for call: {}", event.uuid);

        let client = self.db_pool.get().await
            .map_err(|e| {
                error!("❌ Failed to get DB connection: {}", e);
                BillingError::Internal(e.to_string())
            })?;
        
        // Try to get reservation info
        let reservation_row = client
            .query_opt(
                "SELECT br.account_id, br.rate_per_minute, rc.billing_increment
                 FROM balance_reservations br
                 LEFT JOIN rate_cards rc ON br.destination_prefix = rc.destination_prefix
                 WHERE br.call_uuid = $1
                 ORDER BY br.created_at ASC
                 LIMIT 1",
                &[&event.uuid],
            )
            .await
            .map_err(|e| {
                error!("❌ Error querying reservation: {}", e);
                BillingError::Database(e)
            })?;

        let (account_id, rate_per_minute, cost) = if let Some(row) = reservation_row {
            let account_id: i64 = row.try_get(0).map_err(|e| {
                error!("❌ Error getting account_id: {}", e);
                BillingError::Internal(format!("Column 0 error: {}", e))
            })?;
            
            let rate: Decimal = row.try_get(1).map_err(|e| {
                error!("❌ Error getting rate_per_minute: {}", e);
                BillingError::Internal(format!("Column 1 error: {}", e))
            })?;
            
            let increment: i32 = row.try_get::<_, Option<i32>>(2)
                .unwrap_or(Some(6))
                .unwrap_or(6);
            
            // Calculate cost with billing increment
            let rounded_billsec = if increment > 0 && event.billsec > 0 {
                ((event.billsec + increment - 1) / increment) * increment
            } else {
                event.billsec
            };
            
            let minutes = Decimal::from(rounded_billsec) / Decimal::from(60);
            let cost = minutes * rate;
            
            info!(
                "💰 Call cost calculation: {}s → {}s ({}s increment) = {} min × ${}/min = ${}",
                event.billsec, rounded_billsec, increment, minutes, rate, cost
            );
            
            (Some(account_id), Some(rate), Some(cost))
        } else {
            warn!("⚠️  No reservation found for call {}, creating basic CDR without billing", event.uuid);
            (None, None, None)
        };

        // Insert CDR - maneja casos con y sin account_id
        let cdr_id: i64 = client
            .query_one(
                "INSERT INTO cdrs 
                 (uuid, account_id, caller, callee, start_time, answer_time, end_time,
                  duration, billsec, hangup_cause, rate_applied, cost, direction, freeswitch_server_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                 RETURNING id",
                &[
                    &event.uuid,                    // $1
                    &account_id,                    // $2 - puede ser NULL
                    &event.caller,                  // $3
                    &event.callee,                  // $4
                    &event.start_time,              // $5
                    &event.answer_time,             // $6
                    &event.end_time,                // $7
                    &event.duration,                // $8
                    &event.billsec,                 // $9
                    &event.hangup_cause,            // $10
                    &rate_per_minute,               // $11 - puede ser NULL
                    &cost,                          // $12 - puede ser NULL
                    &event.direction,               // $13
                    &event.server_id,               // $14
                ],
            )
            .await
            .map_err(|e| {
                error!("❌ Failed to insert CDR: {}", e);
                error!("   UUID: {}", event.uuid);
                error!("   Account ID: {:?}", account_id);
                error!("   Caller: {}", event.caller);
                error!("   Callee: {}", event.callee);
                error!("   Duration: {}s", event.duration);
                error!("   Billsec: {}s", event.billsec);
                BillingError::Database(e)
            })?
            .get(0);

        // Consume reservation if exists
        if account_id.is_some() && cost.is_some() {
            let consume_req = ConsumeReservationRequest {
                call_uuid: event.uuid.clone(),
                actual_cost: cost.unwrap().to_f64().unwrap_or(0.0),
                actual_billsec: event.billsec,
            };

            match self.reservation_mgr.consume_reservation(&consume_req).await {
                Ok(result) => {
                    info!(
                        "✅ Reservation consumed: Reserved ${}, Consumed ${}, Released ${}",
                        result.total_reserved, result.consumed, result.released
                    );
                }
                Err(e) => {
                    error!("❌ Failed to consume reservation for {}: {}", event.uuid, e);
                    // No retornar error aquí, el CDR ya está guardado
                }
            }
        } else {
            info!("ℹ️  No reservation to consume for call {}", event.uuid);
        }

        info!(
            "✅ CDR generated: ID={}, UUID={}, Duration={}s, Billsec={}s, Cost=${:?}, Cause={}",
            cdr_id, event.uuid, event.duration, event.billsec, 
            cost.map(|c| c.to_f64().unwrap_or(0.0)), 
            event.hangup_cause
        );

        Ok(cdr_id)
    }
}