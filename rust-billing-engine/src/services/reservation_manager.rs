// src/services/reservation_manager.rs
use crate::models::{BalanceReservation, ReservationStatus, ReservationType, ConsumeReservationRequest, ConsumeReservationResponse};
use crate::database::DbPool;
use crate::cache::RedisClient;
use crate::error::BillingError;
use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive}; // Added for to_f64, from_f64, to_i32
use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDateTime}; 
use tracing::{info, warn, error};

// Configuration constants
const INITIAL_RESERVATION_MINUTES: i32 = 5;
const RESERVATION_BUFFER_PERCENT: i32 = 8;
const MIN_RESERVATION_AMOUNT: f64 = 0.30;
const MAX_RESERVATION_AMOUNT: f64 = 30.00;
const RESERVATION_TTL: i64 = 2700; // 45 minutes
const MAX_CONCURRENT_CALLS: i32 = 5;
// const TOTAL_RESERVED_LIMIT_PERCENT: i32 = 85; // Unused warning

pub struct ReservationResult {
    pub success: bool,
    pub reason: String,
    pub reservation_id: Uuid,
    pub reserved_amount: f64,
    pub max_duration_seconds: i32,
}

pub struct ExtensionResult {
    pub success: bool,
    pub reason: String,
    pub additional_reserved: f64,
    pub new_max_duration_seconds: i32,
}

pub struct ReservationManager {
    db_pool: DbPool,
    redis: RedisClient,
}

impl ReservationManager {
    pub fn new(db_pool: DbPool, redis: RedisClient) -> Self {
        Self { db_pool, redis }
    }

    pub async fn create_reservation(
        &self,
        account_id: i64,
        call_uuid: &str,
        destination: &str,
        rate_per_minute: Decimal,
    ) -> Result<ReservationResult, BillingError> {
        // Calculate amount to reserve
        let base_amount = rate_per_minute * Decimal::from(INITIAL_RESERVATION_MINUTES);
        let buffer = base_amount * Decimal::from(RESERVATION_BUFFER_PERCENT) / Decimal::from(100);
        let mut total_reservation = base_amount + buffer;

        // Apply min/max limits
        total_reservation = total_reservation.max(Decimal::from_f64(MIN_RESERVATION_AMOUNT).unwrap());
        total_reservation = total_reservation.min(Decimal::from_f64(MAX_RESERVATION_AMOUNT).unwrap());

        info!(
            "Calculating reservation: base=${}, buffer=${} ({}%), total=${}",
            base_amount, buffer, RESERVATION_BUFFER_PERCENT, total_reservation
        );

        // Check available balance
        let available_balance = self.get_available_balance(account_id).await?;

        if available_balance < total_reservation {
            warn!(
                "Insufficient balance: required ${}, available ${}",
                total_reservation, available_balance
            );
            return Ok(ReservationResult {
                success: false,
                reason: "insufficient_balance".to_string(),
                reservation_id: Uuid::nil(),
                reserved_amount: 0.0,
                max_duration_seconds: 0,
            });
        }

        // Check concurrent limits
        if !self.check_concurrent_limits(account_id, total_reservation).await? {
            return Ok(ReservationResult {
                success: false,
                reason: "concurrent_limit_exceeded".to_string(),
                reservation_id: Uuid::nil(),
                reserved_amount: 0.0,
                max_duration_seconds: 0,
            });
        }

        // Create reservation in database
        let reservation_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::seconds(RESERVATION_TTL);

        let client = self.db_pool.get().await
            .map_err(|e| BillingError::Internal(e.to_string()))?;
        
        // ✅ SOLUCIÓN: Convertir DateTime<Utc> a NaiveDateTime
        let expires_at_naive = expires_at.naive_utc();
        
        // Convert i64 to i32 for account_id column
        let account_id_i32 = account_id as i32;

        let total_reservation_64 = total_reservation.to_f64().unwrap_or(0.0);
        let account_id_i32 = account_id as i32;
        let expires_at_naive = expires_at.naive_utc();
        let dest_prefix_str = String::from(&destination[..std::cmp::min(10, destination.len())]); // Crear string temporal
        let call_uuid_str = call_uuid.to_string();

        client
            .execute(
                "INSERT INTO balance_reservations 
                (id, account_id, call_uuid, reserved_amount, consumed_amount, released_amount,
                status, reservation_type, destination_prefix, rate_per_minute, reserved_minutes,
                expires_at, created_by)
                VALUES ($1, $2, $3, $4, 0, 0, $5, $6, $7, $8, $9, $10, 'system')",
                &[
                    &reservation_id,
                    &account_id_i32,
                    &call_uuid_str,
                    &total_reservation_64,
                    &"active",
                    &"initial",
                    &dest_prefix_str,
                    &rate_per_minute,
                    &INITIAL_RESERVATION_MINUTES,
                    &expires_at_naive,  // ✅ Usar naive_utc() en lugar de expires_at directamente
                ],
            )
            .await
            .map_err(|e| {
                error!("❌ Failed to insert reservation: {}", e);
                BillingError::Database(e)
            })?;

        // Cache in Redis
        let cache_data = serde_json::json!({
            "account_id": account_id,
            "call_uuid": call_uuid,
            "reserved_amount": total_reservation.to_f64().unwrap(),
            "status": "active",
            "rate_per_minute": rate_per_minute.to_f64().unwrap(),
        });

        self.redis
            .set(
                &format!("reservation:{}", reservation_id),
                &cache_data.to_string(),
                RESERVATION_TTL as usize,
            )
            .await
            .map_err(|e| BillingError::Internal(e.to_string()))?;

        // Add to active reservations set
        self.redis
            .sadd(&format!("active_reservations:{}", account_id), &reservation_id.to_string())
            .await
            .map_err(|e| BillingError::Internal(e.to_string()))?;

        // Calculate max duration
        let max_duration_seconds = ((total_reservation / rate_per_minute) * Decimal::from(60))
            .to_i32()
            .unwrap_or(0);

        info!(
            "✅ Reservation created: {} for account {}. Amount: ${}, Max duration: {}s",
            reservation_id, account_id, total_reservation, max_duration_seconds
        );

        Ok(ReservationResult {
            success: true,
            reason: "created".to_string(),
            reservation_id,
            reserved_amount: total_reservation.to_f64().unwrap(),
            max_duration_seconds,
        })
    }

    pub async fn consume_reservation(
        &self,
        req: &ConsumeReservationRequest,
    ) -> Result<ConsumeReservationResponse, BillingError> {
        let actual_cost = Decimal::from_f64(req.actual_cost)
            .ok_or_else(|| BillingError::InvalidRequest("Invalid cost".to_string()))?;

        // Get all active reservations for this call
        let client = self.db_pool.get().await
            .map_err(|e| BillingError::Internal(e.to_string()))?;
        
        let rows = client
            .query(
                "SELECT id, account_id, reserved_amount, consumed_amount, released_amount
                 FROM balance_reservations
                 WHERE call_uuid = $1 AND status = 'active'
                 ORDER BY created_at ASC",
                &[&req.call_uuid],
            )
            .await?;

        if rows.is_empty() {
            error!("No active reservations found for call {}", req.call_uuid);
            return Err(BillingError::ReservationFailed("No active reservations".to_string()));
        }

        let account_id: i64 = rows[0].get(1);
        let mut total_reserved = Decimal::ZERO;
        
        for row in &rows {
            let reserved: Decimal = row.get(2);
            let consumed: Decimal = row.get(3);
            total_reserved += reserved - consumed;
        }

        info!(
            "Processing reservation consumption for call {}. Reserved: ${}, Actual: ${}",
            req.call_uuid, total_reserved, actual_cost
        );

        let (consumed, released) = if actual_cost <= total_reserved {
            // Normal case
            self.consume_normal(&client, &rows, actual_cost, account_id, &req.call_uuid).await?;
            (actual_cost, total_reserved - actual_cost)
        } else {
            // Deficit case
            self.consume_deficit(&client, &rows, actual_cost, total_reserved, account_id, &req.call_uuid).await?;
            (total_reserved, Decimal::ZERO)
        };

        // Cleanup Redis
        for row in &rows {
            let reservation_id: Uuid = row.get(0);
            let _ = self.redis.delete(&format!("reservation:{}", reservation_id)).await;
            let _ = self.redis.srem(&format!("active_reservations:{}", account_id), &reservation_id.to_string()).await;
        }

        info!(
            "✅ Reservation consumed for call {}. Reserved: ${}, Consumed: ${}, Released: ${}",
            req.call_uuid, total_reserved, consumed, released
        );

        Ok(ConsumeReservationResponse {
            success: true,
            total_reserved: total_reserved.to_f64().unwrap(),
            consumed: consumed.to_f64().unwrap(),
            released: released.to_f64().unwrap(),
        })
    }

    async fn get_available_balance(&self, account_id: i64) -> Result<Decimal, BillingError> {
        let account_id_i32 = account_id as i32;
        
        let client = self.db_pool.get().await
            .map_err(|e| BillingError::Internal(e.to_string()))?;
        
        let balance_row = client
            .query_one("SELECT balance FROM accounts WHERE id = $1", &[&account_id_i32])
            .await?;
        let balance: Decimal = balance_row.get(0);

        let reserved_row = client
            .query_one(
                "SELECT COALESCE(SUM(reserved_amount - consumed_amount), 0)
                 FROM balance_reservations
                 WHERE account_id = $1 AND status = 'active'",
                &[&account_id_i32],
            )
            .await?;
        let total_reserved: Decimal = reserved_row.get(0);

        Ok(balance - total_reserved)
    }

    async fn check_concurrent_limits(
        &self,
        account_id: i64,
        _new_reservation: Decimal,
    ) -> Result<bool, BillingError> {
        let active_count = self.redis
            .scard(&format!("active_reservations:{}", account_id))
            .await
            .map_err(|e| BillingError::Internal(e.to_string()))?;

        Ok(active_count < MAX_CONCURRENT_CALLS)
    }

    async fn consume_normal(
        &self,
        client: &deadpool_postgres::Client,
        rows: &[tokio_postgres::Row],
        actual_cost: Decimal,
        account_id: i64,
        call_uuid: &str,
    ) -> Result<(), BillingError> {
        let account_id_i32 = account_id as i32;
        let mut remaining = actual_cost;

        // Consume FIFO
        for row in rows {
            if remaining <= Decimal::ZERO {
                break;
            }

            let reservation_id: Uuid = row.get(0);
            let reserved: Decimal = row.get(2);
            let consumed: Decimal = row.get(3);
            let available = reserved - consumed;

            let consume_from_this = remaining.min(available);

            client
                .execute(
                    "UPDATE balance_reservations
                     SET consumed_amount = consumed_amount + $1,
                         status = CASE
                             WHEN consumed_amount + $1 >= reserved_amount THEN 'fully_consumed'
                             ELSE 'partially_consumed'
                         END,
                         consumed_at = NOW()
                     WHERE id = $2",
                    &[&consume_from_this, &reservation_id],
                )
                .await?;

            remaining -= consume_from_this;
        }

        // Deduct from account balance
        client
            .execute(
                "UPDATE accounts SET balance = balance - $1, updated_at = NOW() WHERE id = $2",
                &[&actual_cost, &account_id_i32],
            )
            .await?;

        // Log transaction
        client
            .execute(
                "INSERT INTO balance_transactions 
                 (account_id, amount, previous_balance, new_balance, transaction_type, reason, call_uuid)
                 SELECT $1, $2, balance + $2, balance, 'reservation_consume', $3, $4
                 FROM accounts WHERE id = $1",
                &[
                    &account_id_i32,
                    &(-actual_cost),
                    &format!("Consumed reservation for call {}", call_uuid),
                    &call_uuid,
                ],
            )
            .await?;

        Ok(())
    }

    async fn consume_deficit(
        &self,
        client: &deadpool_postgres::Client,
        rows: &[tokio_postgres::Row],
        actual_cost: Decimal,
        total_reserved: Decimal,
        account_id: i64,
        call_uuid: &str,
    ) -> Result<(), BillingError> {
        let account_id_i32 = account_id as i32;
        
        // Mark all as fully consumed
        for row in rows {
            let reservation_id: Uuid = row.get(0);
            let reserved: Decimal = row.get(2);

            client
                .execute(
                    "UPDATE balance_reservations
                     SET consumed_amount = $1,
                         status = 'fully_consumed',
                         consumed_at = NOW()
                     WHERE id = $2",
                    &[&reserved, &reservation_id],
                )
                .await?;
        }

        // Deduct FULL cost
        client
            .execute(
                "UPDATE accounts SET balance = balance - $1, updated_at = NOW() WHERE id = $2",
                &[&actual_cost, &account_id_i32],
            )
            .await?;

        let deficit = actual_cost - total_reserved;

        // Log transaction
        client
            .execute(
                "INSERT INTO balance_transactions 
                 (account_id, amount, previous_balance, new_balance, transaction_type, reason, call_uuid)
                 SELECT $1, $2, balance + $2, balance, 'reservation_consume', $3, $4
                 FROM accounts WHERE id = $1",
                &[
                    &account_id_i32,
                    &(-actual_cost),
                    &format!("DEFICIT: Consumed ${} reserved + ${} deficit for call {}", total_reserved, deficit, call_uuid),
                    &call_uuid,
                ],
            )
            .await?;

        error!(
            "🚨 RESERVATION DEFICIT: Account {}, Call {}, Deficit: ${}",
            account_id, call_uuid, deficit
        );

        Ok(())
    }

    /// Extend an existing reservation when call is approaching max duration
    pub async fn extend_reservation(
        &self,
        call_uuid: &str,
        additional_minutes: i32,
    ) -> Result<ExtensionResult, BillingError> {
        info!("🔄 Attempting to extend reservation for call: {}", call_uuid);

        // Get existing active reservations
        let client = self.db_pool.get().await
            .map_err(|e| BillingError::Internal(e.to_string()))?;
        
        let rows = client
            .query(
                "SELECT id, account_id, rate_per_minute, reserved_amount, consumed_amount
                 FROM balance_reservations
                 WHERE call_uuid = $1 AND status = 'active'
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&call_uuid],
            )
            .await?;

        if rows.is_empty() {
            warn!("❌ No active reservation found for call: {}", call_uuid);
            return Ok(ExtensionResult {
                success: false,
                reason: "no_active_reservation".to_string(),
                additional_reserved: 0.0,
                new_max_duration_seconds: 0,
            });
        }

        let row = &rows[0];
        let reservation_id: Uuid = row.get(0);
        let account_id: i64 = row.get(1);
        let rate_per_minute: Decimal = row.get(2);
        let current_reserved: Decimal = row.get(3);
        let consumed: Decimal = row.get(4);

        // Calculate extension amount
        let base_amount = rate_per_minute * Decimal::from(additional_minutes);
        let buffer = base_amount * Decimal::from(RESERVATION_BUFFER_PERCENT) / Decimal::from(100);
        let mut extension_amount = base_amount + buffer;

        // Apply limits
        extension_amount = extension_amount.max(Decimal::from_f64(MIN_RESERVATION_AMOUNT).unwrap());
        extension_amount = extension_amount.min(Decimal::from_f64(MAX_RESERVATION_AMOUNT).unwrap());

        info!(
            "Extension calculation: base=${}, buffer=${} ({}%), total=${}",
            base_amount, buffer, RESERVATION_BUFFER_PERCENT, extension_amount
        );

        // Check available balance
        let available_balance = self.get_available_balance(account_id).await?;

        if available_balance < extension_amount {
            warn!(
                "❌ Insufficient balance for extension: required ${}, available ${}",
                extension_amount, available_balance
            );
            return Ok(ExtensionResult {
                success: false,
                reason: "insufficient_balance".to_string(),
                additional_reserved: 0.0,
                new_max_duration_seconds: 0,
            });
        }

        // Create new extension reservation
        let extension_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::seconds(RESERVATION_TTL);
        let expires_at_naive = expires_at.naive_utc();

        // Get destination prefix from original reservation
        let dest_row = client
            .query_one(
                "SELECT destination_prefix FROM balance_reservations WHERE id = $1",
                &[&reservation_id],
            )
            .await?;
        let destination_prefix: String = dest_row.get(0);

        // Convert i64 to i32 for account_id column
        let account_id_i32 = account_id as i32;
        
        client
            .execute(
                "INSERT INTO balance_reservations 
                (id, account_id, call_uuid, reserved_amount, consumed_amount, released_amount,
                status, reservation_type, destination_prefix, rate_per_minute, reserved_minutes,
                expires_at, created_by)
                VALUES ($1, $2, $3, $4, 0, 0, $5, $6, $7, $8, $9, $10, 'system_extension')",
                &[
                    &extension_id,
                    &account_id_i32,
                    &call_uuid,
                    &extension_amount,
                    &"active",
                    &"extension",
                    &destination_prefix,
                    &rate_per_minute,
                    &additional_minutes,
                    &expires_at_naive,
                ],
            )
            .await
            .map_err(|e| {
                error!("❌ Failed to insert extension reservation: {}", e);
                BillingError::Database(e)
            })?;

        // Update Redis cache
        let cache_data = serde_json::json!({
            "account_id": account_id,
            "call_uuid": call_uuid,
            "reserved_amount": extension_amount.to_f64().unwrap(),
            "status": "active",
            "rate_per_minute": rate_per_minute.to_f64().unwrap(),
            "type": "extension",
        });

        self.redis
            .set(
                &format!("reservation:{}", extension_id),
                &cache_data.to_string(),
                RESERVATION_TTL as usize,
            )
            .await
            .map_err(|e| BillingError::Internal(e.to_string()))?;

        // Add to active reservations set
        self.redis
            .sadd(&format!("active_reservations:{}", account_id), &extension_id.to_string())
            .await
            .map_err(|e| BillingError::Internal(e.to_string()))?;

        // Calculate new max duration
        let total_reserved = current_reserved + extension_amount - consumed;
        let new_max_duration_seconds = ((total_reserved / rate_per_minute) * Decimal::from(60))
            .to_i32()
            .unwrap_or(0);

        info!(
            "✅ Reservation extended: {} for call {}. Extension: ${}, New max duration: {}s",
            extension_id, call_uuid, extension_amount, new_max_duration_seconds
        );

        Ok(ExtensionResult {
            success: true,
            reason: "extended".to_string(),
            additional_reserved: extension_amount.to_f64().unwrap(),
            new_max_duration_seconds,
        })
    }
}
