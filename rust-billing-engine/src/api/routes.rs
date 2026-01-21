// src/api/routes.rs
use actix_web::web;
use crate::api::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(handlers::health_check))
            .route("/authorize", web::post().to(handlers::authorize_call))
            .route("/reservation/consume", web::post().to(handlers::consume_reservation))
            // FreeSWITCH-specific endpoint (GET with query params)
            .route("/freeswitch/authorize", web::get().to(handlers::freeswitch_authorize))
    );
}
