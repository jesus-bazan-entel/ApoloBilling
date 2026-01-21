// src/services/authorization.rs

use crate::models::{AuthRequest, AuthResponse, Account, AccountStatus, AccountType};
use crate::database::DbPool;
use crate::cache::RedisClient;
use crate::services::ReservationManager;
use crate::error::BillingError;
use std::sync::Arc;
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use tracing::{info, warn, error};
use chrono::{DateTime, Utc};

pub struct AuthorizationService {
    db_pool: DbPool,
    redis: RedisClient,
    reservation_mgr: Arc<ReservationManager>,
}

impl AuthorizationService {
    pub fn new(
        db_pool: DbPool,
        redis: RedisClient,
        reservation_mgr: Arc<ReservationManager>,
    ) -> Self {
        Self {
            db_pool,
            redis,
            reservation_mgr,
        }
    }

    pub async fn authorize(&self, req: &AuthRequest) -> Result<AuthResponse, BillingError> {
        let call_uuid = req.uuid.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        
        info!(
            "🔍 Authorizing call [v2] {}: {} → {}",
            call_uuid, req.caller, req.callee
        );

        // 1. Find account by ANI (caller)
        let account = match self.find_account_by_ani(&req.caller).await? {
            Some(acc) => acc,
            None => {
                warn!("❌ Account not found for caller: {}", req.caller);
                return Ok(AuthResponse {
                    authorized: false,
                    reason: "account_not_found".to_string(),
                    uuid: call_uuid,
                    account_id: None,
                    account_number: None,
                    reservation_id: None,
                    reserved_amount: None,
                    max_duration_seconds: None,
                    rate_per_minute: None,
                });
            }
        };

        // 2. Check account status
        if account.status != AccountStatus::Active {
            warn!("❌ Account {} is {:?}", account.account_number, account.status);
            return Ok(AuthResponse {
                authorized: false,
                reason: format!("account_{:?}", account.status).to_lowercase(),
                uuid: call_uuid,
                account_id: Some(account.id.into()),
                account_number: Some(account.account_number),
                reservation_id: None,
                reserved_amount: None,
                max_duration_seconds: None,
                rate_per_minute: None,
            });
        }

        // 3. Get rate for destination
        let rate = match self.get_rate(&req.callee).await? {
            Some(r) => r,
            None => {
                warn!("❌ No rate found for destination: {}", req.callee);
                return Ok(AuthResponse {
                    authorized: false,
                    reason: "no_rate_found".to_string(),
                    uuid: call_uuid,
                    account_id: Some(account.id.into()),
                    account_number: Some(account.account_number),
                    reservation_id: None,
                    reserved_amount: None,
                    max_duration_seconds: None,
                    rate_per_minute: None,
                });
            }
        };

        info!(
            "📊 Rate found: {} - ${}/min",
            rate.destination_name, rate.rate_per_minute
        );

        // 4. Create reservation
        let reservation_result = self.reservation_mgr
            .create_reservation(
                account.id.into(),
                &call_uuid,
                &req.callee,
                rate.rate_per_minute,
            )
            .await?;

        if !reservation_result.success {
            warn!("❌ Reservation failed: {}", reservation_result.reason);
            return Ok(AuthResponse {
                authorized: false,
                reason: reservation_result.reason,
                uuid: call_uuid,
                account_id: Some(account.id.into()),
                account_number: Some(account.account_number),
                reservation_id: None,
                reserved_amount: None,
                max_duration_seconds: None,
                rate_per_minute: None,
            });
        }

        // 5. AUTHORIZED ✅
        info!(
            "✅ Call AUTHORIZED: {} for account {}",
            call_uuid, account.account_number
        );

        Ok(AuthResponse {
            authorized: true,
            reason: "authorized".to_string(),
            uuid: call_uuid,
            account_id: Some(account.id.into()),
            account_number: Some(account.account_number.clone()),
            reservation_id: Some(reservation_result.reservation_id),
            reserved_amount: Some(reservation_result.reserved_amount),
            max_duration_seconds: Some(reservation_result.max_duration_seconds),
            rate_per_minute: Some(rate.rate_per_minute.to_f64().unwrap_or(0.0)),
        })
    }

    async fn find_account_by_ani(&self, ani: &str) -> Result<Option<Account>, BillingError> {
        let normalized = ani.replace('+', "").replace(' ', "").replace('-', "");

        let client = self.db_pool.get().await
            .map_err(|e| BillingError::Internal(e.to_string()))?;
        
        let row = client
            .query_opt(
                "SELECT id, account_number, account_type::text, balance,
                        max_concurrent_calls, status::text,
                        created_at, updated_at
                FROM accounts
                WHERE (account_number = $1 OR account_number = $2)
                LIMIT 1",
                &[&ani, &normalized],
            )
            .await
            .map_err(|e| {
                error!("❌ Database error finding account: {:?}", e);
                BillingError::Database(e)
            })?;

        match row {
            Some(r) => {
                let id: i32 = r.get(0);
                let account_number: String = r.get(1);
                let account_type_str: String = r.get(2);
                let balance: Decimal = r.get(3);
                let max_concurrent_calls: i32 = r.get(4);
                let status_str: String = r.get(5);
                
                // ✅ TIMESTAMP WITH TIME ZONE se deserializa directamente como DateTime<Utc>
                let created_at: DateTime<Utc> = r.get(6);
                let updated_at: DateTime<Utc> = r.get(7);

                info!(
                    "✅ Found account: {} (ID: {}, Type: {}, Balance: ${}, Status: {})",
                    account_number, id, account_type_str, balance, status_str
                );

                Ok(Some(Account {
                    id,
                    account_number,
                    account_type: AccountType::from_str(&account_type_str),
                    balance,
                    credit_limit: Decimal::ZERO,
                    currency: "USD".to_string(),
                    status: AccountStatus::from_str(&status_str),
                    max_concurrent_calls: Some(max_concurrent_calls),
                    created_at,
                    updated_at,
                }))
            }
            None => {
                info!("ℹ️  No account found for ANI: {} (normalized: {})", ani, normalized);
                Ok(None)
            }
        }
    }

    async fn get_rate(&self, destination: &str) -> Result<Option<crate::models::RateCard>, BillingError> {
        let normalized = destination.replace('+', "");

        let cache_key = format!("rate:{}", &normalized[..std::cmp::min(10, normalized.len())]);
        if let Ok(Some(cached)) = self.redis.get(&cache_key).await {
            if let Ok(rate) = serde_json::from_str(&cached) {
                return Ok(Some(rate));
            }
        }

        let client = self.db_pool.get().await
            .map_err(|e| BillingError::Internal(e.to_string()))?;
        
        let mut prefixes: Vec<String> = Vec::new();
        for i in (1..=normalized.len()).rev() {
            prefixes.push(normalized[..i].to_string());
        }
        info!("🔎 Generated prefixes for {}: {:?}", normalized, prefixes);

        let row = client
            .query_opt(
                "SELECT id, destination_prefix, destination_name, rate_per_minute,
                        billing_increment, connection_fee, 
                        effective_start, effective_end, priority
                FROM rate_cards
                WHERE destination_prefix = ANY($1)
                AND effective_start <= NOW()
                AND (effective_end IS NULL OR effective_end >= NOW())
                ORDER BY LENGTH(destination_prefix) DESC, priority DESC
                LIMIT 1",
                &[&prefixes],
            )
            .await
            .map_err(|e| {
                error!("❌ Database error getting rate: {:?}", e);
                BillingError::Database(e)
            })?;

        match row {
            Some(r) => {
                // TIMESTAMP WITH TIME ZONE se deserializa directamente como DateTime<Utc>
                let effective_start: DateTime<Utc> = r.get(6);
                let effective_end: Option<DateTime<Utc>> = r.try_get(7).ok();

                let rate = crate::models::RateCard {
                    id: r.get(0),
                    destination_prefix: r.get(1),
                    destination_name: r.get(2),
                    rate_per_minute: r.get(3),
                    billing_increment: r.get(4),
                    connection_fee: r.get(5),
                    effective_start,
                    effective_end,
                    priority: r.get(8),
                };

                info!(
                    "✅ Rate card loaded: {} (${}/min, {} sec increment, priority {})",
                    rate.destination_name, rate.rate_per_minute, rate.billing_increment, rate.priority
                );

                if let Ok(json) = serde_json::to_string(&rate) {
                    let _ = self.redis.set(&cache_key, &json, 300).await;
                }

                Ok(Some(rate))
            }
            None => Ok(None),
        }
    }
}
