// src/services/authorization.rs
use crate::models::{AuthRequest, AuthResponse, Account, AccountStatus, AccountType};
use crate::database::DbPool;
use crate::cache::RedisClient;
use crate::services::ReservationManager;
use crate::error::BillingError;
use std::sync::Arc;
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive; // Added for to_f64
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
        // Normalize ANI
        let normalized = ani.replace('+', "").replace(' ', "").replace('-', "");

        let client = self.db_pool.get().await
            .map_err(|e| BillingError::Internal(e.to_string()))?;
        
        let row = client
            .query_opt(
                "SELECT id, account_number, account_type, balance, 
                        COALESCE(max_concurrent_calls, 5) as max_concurrent_calls,
                        status, 
                        created_at AT TIME ZONE 'UTC', updated_at AT TIME ZONE 'UTC'
                FROM accounts
                WHERE account_number = $1 OR account_number = $2
                AND status = 'ACTIVE'
                LIMIT 1",
                &[&ani, &normalized],
            )
            .await
            .map_err(|e| {
                error!("❌ Database error finding account: {}", e);
                BillingError::Database(e)  // ✅ CORREGIDO
            })?;

        match row {
            Some(r) => {
                // Extrae cada campo con manejo de errores individual
                let id: i32 = r.try_get(0).map_err(|e| {
                    error!("❌ Error getting id (column 0): {}", e);
                    BillingError::Internal(format!("Column 0 error: {}", e))
                })?;
                
                let account_number: String = r.try_get(1).map_err(|e| {
                    error!("❌ Error getting account_number (column 1): {}", e);
                    BillingError::Internal(format!("Column 1 error: {}", e))
                })?;
                
                let account_type_str: String = r.try_get(2).map_err(|e| {
                    error!("❌ Error getting account_type (column 2): {}", e);
                    BillingError::Internal(format!("Column 2 error: {}", e))
                })?;
                
                let balance: Decimal = r.try_get(3).map_err(|e| {
                    error!("❌ Error getting balance (column 3): {}", e);
                    BillingError::Internal(format!("Column 3 error: {}", e))
                })?;
                
                let max_concurrent_calls: i32 = r.try_get(4).map_err(|e| {
                    error!("❌ Error getting max_concurrent_calls (column 4): {}", e);
                    BillingError::Internal(format!("Column 4 error: {}", e))
                })?;
                
                let status_str: String = r.try_get(5).map_err(|e| {
                    error!("❌ Error getting status (column 5): {}", e);
                    BillingError::Internal(format!("Column 5 error: {}", e))
                })?;
                
                let created_at: DateTime<Utc> = r.try_get(6).map_err(|e| {
                    error!("❌ Error getting created_at (column 6): {}", e);
                    BillingError::Internal(format!("Column 6 error: {}", e))
                })?;
                
                let updated_at: DateTime<Utc> = r.try_get(7).map_err(|e| {
                    error!("❌ Error getting updated_at (column 7): {}", e);
                    BillingError::Internal(format!("Column 7 error: {}", e))
                })?;

                info!(
                    "✅ Found account: {} (ID: {}, Type: {}, Balance: ${}, Status: {})",
                    account_number, id, account_type_str, balance, status_str
                );

                Ok(Some(Account {
                    id,
                    account_number,
                    account_type: AccountType::from_str(&account_type_str),
                    balance,
                    credit_limit: Decimal::ZERO,  // No credit_limit in schema
                    currency: "USD".to_string(),  // Default currency
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

        // Try cache first
        let cache_key = format!("rate:{}", &normalized[..std::cmp::min(10, normalized.len())]);
        if let Ok(Some(cached)) = self.redis.get(&cache_key).await {
            if let Ok(rate) = serde_json::from_str(&cached) {
                return Ok(Some(rate));
            }
        }

        // Query database with longest prefix match
        let client = self.db_pool.get().await
            .map_err(|e| BillingError::Internal(e.to_string()))?;
        
        // Generate all possible prefixes (descending length)
        let mut prefixes = Vec::new();
        for i in (1..=normalized.len()).rev() {
            prefixes.push(&normalized[..i]);
        }
        info!("🔎 Generated prefixes for {}: {:?}", normalized, prefixes);

        let row = client
            .query_opt(
                "SELECT id, destination_prefix, destination_name, rate_per_minute,
                        billing_increment, connection_fee, 
                        effective_start AT TIME ZONE 'UTC', 
                        effective_end AT TIME ZONE 'UTC', 
                        priority
                FROM rate_cards
                WHERE destination_prefix = ANY($1)
                AND effective_start <= NOW()
                AND (effective_end IS NULL OR effective_end >= NOW())
                ORDER BY LENGTH(destination_prefix) DESC, priority DESC
                LIMIT 1",
                &[&prefixes],
            )
            .await?;

        match row {
            Some(r) => {
                let rate = crate::models::RateCard {
                    id: r.get(0),
                    destination_prefix: r.get(1),
                    destination_name: r.get(2),
                    rate_per_minute: r.get(3),
                    billing_increment: r.get(4),
                    connection_fee: r.get(5),
                    effective_start: r.get(6),
                    effective_end: r.get(7),
                    priority: r.get(8),
                };

                // Cache result
                if let Ok(json) = serde_json::to_string(&rate) {
                    let _ = self.redis.set(&cache_key, &json, 300).await; // 5 min TTL
                }

                Ok(Some(rate))
            }
            None => Ok(None),
        }
    }

}
