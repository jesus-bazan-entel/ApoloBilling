// src/services/realtime_biller.rs
use crate::cache::RedisClient;
use crate::services::ReservationManager;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

const EXTENSION_CHECK_INTERVAL: u64 = 180; // 3 minutes

pub struct RealtimeBiller {
    redis: RedisClient,
    reservation_mgr: Arc<ReservationManager>,
    active_monitors: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl RealtimeBiller {
    pub fn new(
        redis: RedisClient,
        reservation_mgr: Arc<ReservationManager>,
    ) -> Self {
        Self {
            redis,
            reservation_mgr,
            active_monitors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_billing(&self, call_uuid: String, account_type: String) {
        // Solo monitorear prepago
        if account_type != "prepaid" {
            info!("Skipping realtime billing for postpaid call: {}", call_uuid);
            return;
        }

        info!("✅ Starting realtime billing for call: {}", call_uuid);

        let redis = self.redis.clone();
        let call_uuid_clone = call_uuid.clone();

        // Spawn monitoring task
        let handle = tokio::spawn(async move {
            Self::monitor_call(redis, call_uuid_clone).await;
        });

        // Store handle
        let mut monitors = self.active_monitors.write().await;
        monitors.insert(call_uuid, handle);
    }

    pub async fn stop_billing(&self, call_uuid: &str) {
        let mut monitors = self.active_monitors.write().await;
        
        if let Some(handle) = monitors.remove(call_uuid) {
            handle.abort();
            info!("🛑 Stopped billing for call: {}", call_uuid);
        }
    }

    async fn monitor_call(redis: RedisClient, call_uuid: String) {
        let mut check_interval = interval(Duration::from_secs(EXTENSION_CHECK_INTERVAL));

        loop {
            check_interval.tick().await;

            // Check if call still active
            let session_key = format!("call_session:{}", call_uuid);
            match redis.get(&session_key).await {
                Ok(Some(session_data)) => {
                    // Parse session
                    if let Ok(session) = serde_json::from_str::<serde_json::Value>(&session_data) {
                        // Check duration vs max_duration
                        if let (Some(max_duration), Some(start_time_str)) = (
                            session["max_duration"].as_i64(),
                            session["start_time"].as_str(),
                        ) {
                            if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(start_time_str) {
                                let elapsed = (chrono::Utc::now() - start_time.with_timezone(&chrono::Utc)).num_seconds();
                                
                                let time_remaining = max_duration - elapsed;
                                
                                if time_remaining < (EXTENSION_CHECK_INTERVAL as i64) {
                                    warn!(
                                        "⏱️ Call {} approaching max duration. Remaining: {}s",
                                        call_uuid, time_remaining
                                    );
                                    
                                    // TODO: Request extension from ReservationManager
                                    // For MVP, just log warning
                                    // Future: extend_reservation(call_uuid).await
                                }
                            }
                        }
                    }
                }
                Ok(None) => {
                    info!("Call {} ended, stopping monitoring", call_uuid);
                    break;
                }
                Err(e) => {
                    error!("Error checking call {}: {}", call_uuid, e);
                    break;
                }
            }
        }
    }
}
