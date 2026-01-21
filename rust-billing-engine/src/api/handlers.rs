// src/api/handlers.rs
use actix_web::{web, HttpResponse};
use crate::services::{AuthorizationService, ReservationManager};
use crate::models::{AuthRequest, ConsumeReservationRequest, HealthResponse};
use std::sync::Arc;

/// Query parameters for FreeSWITCH authorization (GET request)
#[derive(Debug, serde::Deserialize)]
pub struct FreeSwitchAuthQuery {
    pub caller: String,
    pub callee: String,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        service: "apolo-billing-engine".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn authorize_call(
    req: web::Json<AuthRequest>,
    auth_service: web::Data<Arc<AuthorizationService>>,
) -> HttpResponse {
    match auth_service.authorize(&req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Authorization error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "authorization_failed",
                "message": e.to_string()
            }))
        }
    }
}

/// FreeSWITCH authorization endpoint (GET with query params)
/// Returns plain text response for easy parsing in dialplan:
/// - "AUTHORIZED|max_seconds|rate_per_minute|account_id" on success
/// - "DENIED|reason" on failure
pub async fn freeswitch_authorize(
    query: web::Query<FreeSwitchAuthQuery>,
    auth_service: web::Data<Arc<AuthorizationService>>,
) -> HttpResponse {
    let auth_req = AuthRequest {
        caller: query.caller.clone(),
        callee: query.callee.clone(),
        uuid: query.uuid.clone(),
        direction: query.direction.clone(),
    };

    match auth_service.authorize(&auth_req).await {
        Ok(response) => {
            if response.authorized {
                // Format: AUTHORIZED|max_seconds|rate_per_minute|account_id|reservation_id
                let body = format!(
                    "AUTHORIZED|{}|{}|{}|{}",
                    response.max_duration_seconds.unwrap_or(3600),
                    response.rate_per_minute.unwrap_or(0.0),
                    response.account_id.unwrap_or(0),
                    response.reservation_id.map(|r| r.to_string()).unwrap_or_default()
                );
                HttpResponse::Ok()
                    .content_type("text/plain")
                    .body(body)
            } else {
                // Format: DENIED|reason
                let body = format!("DENIED|{}", response.reason);
                HttpResponse::Ok()
                    .content_type("text/plain")
                    .body(body)
            }
        }
        Err(e) => {
            tracing::error!("FreeSWITCH authorization error: {}", e);
            HttpResponse::Ok()
                .content_type("text/plain")
                .body(format!("DENIED|internal_error:{}", e))
        }
    }
}

pub async fn consume_reservation(
    req: web::Json<ConsumeReservationRequest>,
    reservation_mgr: web::Data<Arc<ReservationManager>>,
) -> HttpResponse {
    match reservation_mgr.consume_reservation(&req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Consume reservation error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "consume_failed",
                "message": e.to_string()
            }))
        }
    }
}
