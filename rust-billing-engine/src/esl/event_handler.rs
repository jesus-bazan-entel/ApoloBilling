// src/esl/event_handler.rs
use crate::services::{AuthorizationService, RealtimeBiller, CdrGenerator};
use crate::models::AuthRequest;
use crate::esl::{event::EslEvent, connection::EslConnection};
use crate::database::DbPool;
use crate::cache::RedisClient;
use std::sync::Arc;
use chrono::Utc;
use tracing::{info, warn, error};

pub struct EventHandler {
    server_id: String,
    auth_service: Arc<AuthorizationService>,
    realtime_biller: Arc<RealtimeBiller>,
    cdr_generator: Arc<CdrGenerator>,
    db_pool: DbPool,
    redis: RedisClient,
    connection: Arc<EslConnection>,  // ✅ Agregado
}

impl EventHandler {
    pub fn new(
        server_id: String,
        auth_service: Arc<AuthorizationService>,
        realtime_biller: Arc<RealtimeBiller>,
        cdr_generator: Arc<CdrGenerator>,
        db_pool: DbPool,
        redis: RedisClient,
        connection: Arc<EslConnection>,  // ✅ Agregado
    ) -> Self {
        Self {
            server_id,
            auth_service,
            realtime_biller,
            cdr_generator,
            db_pool,
            redis,
            connection,  // ✅ Agregado
        }
    }

    pub async fn handle_event(&self, event: &EslEvent) {
        if let Some(event_name) = event.event_name() {
            match event_name.as_str() {
                "CHANNEL_CREATE" => self.handle_channel_create(event).await,
                "CHANNEL_ANSWER" => self.handle_channel_answer(event).await,
                "CHANNEL_HANGUP_COMPLETE" => self.handle_channel_hangup(event).await,
                _ => {}
            }
        }
    }

    async fn handle_channel_create(&self, event: &EslEvent) {
        let uuid = match event.unique_id() {
            Some(id) => id.clone(),
            None => return,
        };

        let caller = event.caller().cloned().unwrap_or_default();
        let callee = event.callee().cloned().unwrap_or_default();

        info!("📞 CHANNEL_CREATE: {} - {} → {}", uuid, caller, callee);

        // Authorize call
        let auth_req = AuthRequest {
            caller: caller.clone(),
            callee: callee.clone(),
            uuid: Some(uuid.clone()),
        };

        match self.auth_service.authorize(&auth_req).await {
            Ok(response) => {
                if !response.authorized {
                    warn!("❌ Call DENIED: {} - Reason: {}", uuid, response.reason);
                    
                    // ✅ Send uuid_kill command to FreeSWITCH
                    let kill_command = format!("api uuid_kill {} CALL_REJECTED\n\n", uuid);
                    match self.connection.send_command(&kill_command).await {
                        Ok(result) => {
                            info!("🔪 Sent kill command for call {}: {}", uuid, result.trim());
                        }
                        Err(e) => {
                            error!("❌ Failed to send kill command for call {}: {}", uuid, e);
                        }
                    }

                } else {
                    info!("✅ Call AUTHORIZED: {}", uuid);
                }
            }
            Err(e) => {
                error!("Error authorizing call {}: {}", uuid, e);
            }
        }
    }

    async fn handle_channel_answer(&self, event: &EslEvent) {
        let uuid = match event.unique_id() {
            Some(id) => id.clone(),
            None => return,
        };

        info!("✅ CHANNEL_ANSWER: {}", uuid);

        // Start realtime billing for prepaid
        // TODO: Get account type from session
        // For now, assume prepaid
        self.realtime_biller.start_billing(uuid, "prepaid".to_string()).await;
    }

    async fn handle_channel_hangup(&self, event: &EslEvent) {
        let uuid = match event.unique_id() {
            Some(id) => id.clone(),
            None => return,
        };

        let caller = event.caller().cloned().unwrap_or_default();
        let callee = event.callee().cloned().unwrap_or_default();
        let duration = event.duration().unwrap_or(0);
        let billsec = event.billsec().unwrap_or(0);
        let hangup_cause = event.hangup_cause().cloned().unwrap_or_else(|| "UNKNOWN".to_string());

        info!(
            "📴 CHANNEL_HANGUP: {} - Duration: {}s, Billsec: {}s, Cause: {}",
            uuid, duration, billsec, hangup_cause
        );

        // Stop realtime billing
        self.realtime_biller.stop_billing(&uuid).await;

        // ✅ Limpia Redis para llamadas rechazadas (que no tienen reservación)
        if let Ok(client) = self.db_pool.get().await {
            match client
                .query_opt(
                    "SELECT account_id FROM balance_reservations WHERE call_uuid = $1 LIMIT 1",
                    &[&uuid]
                )
                .await 
            {
                Ok(None) => {
                    // No hay reservación, limpia cualquier estado residual en Redis
                    info!("🧹 Cleaning up Redis for non-reserved call: {}", uuid);
                    let _ = self.redis.delete(&format!("call_state:{}", uuid)).await;
                }
                Ok(Some(_)) => {
                    // Hay reservación, el CDR se encargará de limpiarla
                }
                Err(e) => {
                    error!("Error checking reservation for {}: {}", uuid, e);
                }
            }
        }

        // Generate CDR
        let hangup_event = crate::services::cdr_generator::HangupEvent {
            uuid: uuid.clone(),
            caller,
            callee,
            start_time: event.start_time().unwrap_or_else(Utc::now),  // ✅ Parse from event
            answer_time: event.answer_time(),  // ✅ Parse from event
            end_time: event.end_time().unwrap_or_else(Utc::now),  // ✅ Parse from event
            duration,
            billsec,
            hangup_cause,
            direction: "outbound".to_string(), // TODO: Get from event
            server_id: self.server_id.clone(),
        };

        if let Err(e) = self.cdr_generator.generate_cdr(hangup_event).await {
            error!("Error generating CDR for {}: {}", uuid, e);
        }
    }
}