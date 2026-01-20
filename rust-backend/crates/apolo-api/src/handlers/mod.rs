//! HTTP request handlers

pub mod account;
pub mod active_call;
pub mod auth;
pub mod cdr;
pub mod dashboard;
pub mod management;
pub mod rate;
pub mod rate_card;
pub mod reservation;

pub use account::configure as configure_accounts;
pub use active_call::configure as configure_active_calls;
pub use active_call::create_cdr;
pub use auth::configure as configure_auth;
pub use cdr::*;
pub use dashboard::configure as configure_dashboard;
pub use management::configure as configure_management;
pub use rate::configure as configure_rates;
pub use rate_card::configure as configure_rate_cards;
pub use reservation::configure as configure_reservations;
