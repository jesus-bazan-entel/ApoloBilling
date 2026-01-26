//! Tipos de datos para la API JSON-RPC de CGRateS
//!
//! Estos tipos mapean las estructuras de datos de CGRateS
//! según la documentación oficial.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ============================================================================
// JSON-RPC 2.0 Types
// ============================================================================

/// Request JSON-RPC 2.0
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<T>,
    pub id: u64,
}

/// Response JSON-RPC 2.0
#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

/// Error JSON-RPC 2.0
#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

// ============================================================================
// SessionS Types
// ============================================================================

/// Argumentos para SessionSv1.AuthorizeEvent
#[derive(Debug, Clone, Serialize)]
pub struct CGRAuthorizationArgs {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Account")]
    pub account: String,

    #[serde(rename = "Destination")]
    pub destination: String,

    #[serde(rename = "OriginID")]
    pub origin_id: String,

    #[serde(rename = "OriginHost")]
    pub origin_host: String,

    #[serde(rename = "RequestType")]
    pub request_type: String,

    #[serde(rename = "SetupTime")]
    pub setup_time: DateTime<Utc>,

    #[serde(rename = "Usage", skip_serializing_if = "Option::is_none")]
    pub usage: Option<i64>,

    #[serde(rename = "Category", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    #[serde(rename = "Subject", skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

/// Respuesta de SessionSv1.AuthorizeEvent
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CGRAuthorizationReply {
    #[serde(rename = "MaxUsage")]
    pub max_usage: Option<i64>,

    #[serde(rename = "ResourceAllocation")]
    pub resource_allocation: Option<String>,

    #[serde(rename = "Attributes")]
    pub attributes: Option<serde_json::Value>,

    #[serde(rename = "Error")]
    pub error: Option<String>,
}

/// Argumentos para SessionSv1.InitiateSession
#[derive(Debug, Clone, Serialize)]
pub struct CGRSessionInitArgs {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Account")]
    pub account: String,

    #[serde(rename = "Destination")]
    pub destination: String,

    #[serde(rename = "OriginID")]
    pub origin_id: String,

    #[serde(rename = "OriginHost")]
    pub origin_host: String,

    #[serde(rename = "RequestType")]
    pub request_type: String,

    #[serde(rename = "AnswerTime")]
    pub answer_time: DateTime<Utc>,

    #[serde(rename = "Usage")]
    pub usage: i64,
}

/// Argumentos para SessionSv1.UpdateSession
#[derive(Debug, Clone, Serialize)]
pub struct CGRSessionUpdateArgs {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "OriginID")]
    pub origin_id: String,

    #[serde(rename = "OriginHost")]
    pub origin_host: String,

    #[serde(rename = "Usage")]
    pub usage: i64,
}

/// Argumentos para SessionSv1.TerminateSession
#[derive(Debug, Clone, Serialize)]
pub struct CGRSessionTerminateArgs {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Account")]
    pub account: String,

    #[serde(rename = "Destination")]
    pub destination: String,

    #[serde(rename = "OriginID")]
    pub origin_id: String,

    #[serde(rename = "OriginHost")]
    pub origin_host: String,

    #[serde(rename = "RequestType")]
    pub request_type: String,

    #[serde(rename = "AnswerTime")]
    pub answer_time: DateTime<Utc>,

    #[serde(rename = "Usage")]
    pub usage: i64,
}

// ============================================================================
// RatingS Types
// ============================================================================

/// Argumentos para APIerSv1.GetCost
#[derive(Debug, Clone, Serialize)]
pub struct CGRGetCostArgs {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Category")]
    pub category: String,

    #[serde(rename = "Subject")]
    pub subject: String,

    #[serde(rename = "Destination")]
    pub destination: String,

    #[serde(rename = "TimeStart")]
    pub time_start: DateTime<Utc>,

    #[serde(rename = "TimeEnd")]
    pub time_end: DateTime<Utc>,
}

/// Respuesta de APIerSv1.GetCost
#[derive(Debug, Clone, Deserialize)]
pub struct CGRCostReply {
    #[serde(rename = "Cost")]
    pub cost: f64,

    #[serde(rename = "RatingPlanId")]
    pub rating_plan_id: Option<String>,
}

// ============================================================================
// AccountS Types
// ============================================================================

/// Argumentos para APIerSv2.GetAccount
#[derive(Debug, Clone, Serialize)]
pub struct CGRGetAccountArgs {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Account")]
    pub account: String,
}

/// Respuesta de APIerSv2.GetAccount
#[derive(Debug, Clone, Deserialize)]
pub struct CGRAccountBalance {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Account")]
    pub account: String,

    #[serde(rename = "BalanceMap")]
    pub balance_map: serde_json::Value,

    #[serde(rename = "Disabled")]
    pub disabled: Option<bool>,
}

/// Argumentos para APIerSv1.SetBalance
#[derive(Debug, Clone, Serialize)]
pub struct CGRSetBalanceArgs {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Account")]
    pub account: String,

    #[serde(rename = "BalanceType")]
    pub balance_type: String,

    #[serde(rename = "Value")]
    pub value: f64,

    #[serde(rename = "Balance")]
    pub balance: Option<CGRBalanceFilter>,
}

/// Filtro de balance para operaciones
#[derive(Debug, Clone, Serialize)]
pub struct CGRBalanceFilter {
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub balance_type: Option<String>,

    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

/// Argumentos para APIerSv1.AddBalance
#[derive(Debug, Clone, Serialize)]
pub struct CGRAddBalanceArgs {
    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Account")]
    pub account: String,

    #[serde(rename = "BalanceType")]
    pub balance_type: String,

    #[serde(rename = "Value")]
    pub value: f64,
}

// ============================================================================
// Rate Profile Types (para sincronización)
// ============================================================================

/// Argumentos para APIerSv1.SetTPRatingProfile
#[derive(Debug, Clone, Serialize)]
pub struct CGRRatingProfileArgs {
    #[serde(rename = "TPid")]
    pub tp_id: String,

    #[serde(rename = "LoadId")]
    pub load_id: String,

    #[serde(rename = "Tenant")]
    pub tenant: String,

    #[serde(rename = "Category")]
    pub category: String,

    #[serde(rename = "Subject")]
    pub subject: String,

    #[serde(rename = "RatingPlanActivations")]
    pub rating_plan_activations: Vec<CGRRatingPlanActivation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CGRRatingPlanActivation {
    #[serde(rename = "ActivationTime")]
    pub activation_time: String,

    #[serde(rename = "RatingPlanId")]
    pub rating_plan_id: String,

    #[serde(rename = "FallbackSubjects")]
    pub fallback_subjects: Option<String>,
}

// ============================================================================
// Request Types (constantes)
// ============================================================================

/// Tipos de request soportados por CGRateS
pub mod request_types {
    pub const PREPAID: &str = "*prepaid";
    pub const POSTPAID: &str = "*postpaid";
    pub const PSEUDOPREPAID: &str = "*pseudoprepaid";
    pub const RATED: &str = "*rated";
    pub const NONE: &str = "*none";
}

/// Tipos de balance en CGRateS
pub mod balance_types {
    pub const MONETARY: &str = "*monetary";
    pub const VOICE: &str = "*voice";
    pub const SMS: &str = "*sms";
    pub const DATA: &str = "*data";
}

// ============================================================================
// Helper Functions
// ============================================================================

impl CGRAuthorizationReply {
    /// Convierte max_usage de nanosegundos a segundos
    pub fn max_usage_seconds(&self) -> Option<i64> {
        self.max_usage.map(|ns| ns / 1_000_000_000)
    }

    /// Verifica si la autorización fue exitosa
    pub fn is_authorized(&self) -> bool {
        self.error.is_none() && self.max_usage.is_some()
    }

    /// Obtiene el mensaje de error si existe
    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl CGRAccountBalance {
    /// Obtiene el balance monetario del account
    pub fn monetary_balance(&self) -> Decimal {
        if let Some(monetary) = self.balance_map.get("*monetary") {
            if let Some(balances) = monetary.as_array() {
                if let Some(first) = balances.first() {
                    if let Some(value) = first.get("Value") {
                        if let Some(v) = value.as_f64() {
                            return Decimal::from_f64_retain(v)
                                .unwrap_or(Decimal::ZERO);
                        }
                    }
                }
            }
        }
        Decimal::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_reply_max_usage_seconds() {
        let reply = CGRAuthorizationReply {
            max_usage: Some(300_000_000_000), // 300 segundos en ns
            ..Default::default()
        };

        assert_eq!(reply.max_usage_seconds(), Some(300));
    }

    #[test]
    fn test_auth_reply_is_authorized() {
        let success = CGRAuthorizationReply {
            max_usage: Some(300_000_000_000),
            error: None,
            ..Default::default()
        };
        assert!(success.is_authorized());

        let failure = CGRAuthorizationReply {
            max_usage: None,
            error: Some("NOT_ENOUGH_BALANCE".to_string()),
            ..Default::default()
        };
        assert!(!failure.is_authorized());
    }

    #[test]
    fn test_serialize_auth_args() {
        let args = CGRAuthorizationArgs {
            tenant: "cgrates.org".to_string(),
            account: "1001".to_string(),
            destination: "51999888777".to_string(),
            origin_id: "uuid-123".to_string(),
            origin_host: "fs1".to_string(),
            request_type: "*prepaid".to_string(),
            setup_time: Utc::now(),
            usage: None,
            category: None,
            subject: None,
        };

        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains("\"Tenant\":\"cgrates.org\""));
        assert!(json.contains("\"RequestType\":\"*prepaid\""));
    }
}
