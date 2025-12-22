// src/api/handlers.rs
use actix_web::{web, HttpResponse};
use crate::services::{AuthorizationService, ReservationManager};
use crate::models::{AuthRequest, ConsumeReservationRequest, HealthResponse};
use std::sync::Arc;

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
