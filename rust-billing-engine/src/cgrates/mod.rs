//! CGRateS Integration Module
//!
//! Este módulo proporciona integración con CGRateS para:
//! - SessionS: Control de sesiones en tiempo real
//! - RatingS: Motor de tarificación
//! - AccountS: Gestión de cuentas y saldos
//!
//! # Uso
//!
//! ```rust,ignore
//! use crate::cgrates::CgratesClient;
//!
//! let client = CgratesClient::new(
//!     "http://127.0.0.1:2080/jsonrpc",
//!     "cgrates.org",
//!     50,  // timeout_ms
//! )?;
//!
//! // Autorizar sesión
//! let reply = client.authorize_session(
//!     "account_number",
//!     "destination",
//!     "call_uuid",
//!     "*prepaid",
//! ).await?;
//! ```

mod client;
mod types;
mod sessions;
mod ratings;
mod accounts;

pub use client::{CgratesClient, CgratesError};
pub use types::*;
