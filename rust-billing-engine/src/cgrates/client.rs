//! Cliente HTTP JSON-RPC 2.0 para CGRateS
//!
//! Proporciona comunicación de bajo nivel con el servidor CGRateS.

use reqwest::{Client, ClientBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, instrument};

use super::types::{JsonRpcRequest, JsonRpcResponse};

/// Cliente para comunicación con CGRateS
pub struct CgratesClient {
    http_client: Client,
    base_url: String,
    tenant: String,
    origin_host: String,
    request_id: AtomicU64,
}

/// Errores del cliente CGRateS
#[derive(Debug, Error)]
pub enum CgratesError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("HTTP error: status {0}")]
    HttpError(u16),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("RPC error {0}: {1}")]
    RpcError(i32, String),

    #[error("Empty response from CGRateS")]
    EmptyResponse,

    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    #[error("Insufficient balance for account")]
    InsufficientBalance,

    #[error("Rate not found for destination: {0}")]
    RateNotFound(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Timeout: request took longer than {0}ms")]
    Timeout(u64),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl CgratesClient {
    /// Crea un nuevo cliente CGRateS
    ///
    /// # Arguments
    ///
    /// * `base_url` - URL del endpoint JSON-RPC (ej: "http://127.0.0.1:2080/jsonrpc")
    /// * `tenant` - Tenant de CGRateS (ej: "cgrates.org")
    /// * `timeout_ms` - Timeout para requests en milisegundos
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = CgratesClient::new(
    ///     "http://127.0.0.1:2080/jsonrpc",
    ///     "cgrates.org",
    ///     50,
    /// )?;
    /// ```
    pub fn new(base_url: &str, tenant: &str, timeout_ms: u64) -> Result<Self, CgratesError> {
        let http_client = ClientBuilder::new()
            .timeout(Duration::from_millis(timeout_ms))
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .map_err(|e| CgratesError::Connection(e.to_string()))?;

        let origin_host = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "apolo-billing".to_string());

        Ok(Self {
            http_client,
            base_url: base_url.to_string(),
            tenant: tenant.to_string(),
            origin_host,
            request_id: AtomicU64::new(1),
        })
    }

    /// Crea un cliente desde variables de entorno
    ///
    /// Espera las siguientes variables:
    /// - CGRATES_URL
    /// - CGRATES_TENANT
    /// - CGRATES_TIMEOUT_MS
    pub fn from_env() -> Result<Self, CgratesError> {
        let base_url = std::env::var("CGRATES_URL")
            .map_err(|_| CgratesError::Config("CGRATES_URL not set".to_string()))?;

        let tenant = std::env::var("CGRATES_TENANT")
            .unwrap_or_else(|_| "cgrates.org".to_string());

        let timeout_ms: u64 = std::env::var("CGRATES_TIMEOUT_MS")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .map_err(|_| CgratesError::Config("Invalid CGRATES_TIMEOUT_MS".to_string()))?;

        Self::new(&base_url, &tenant, timeout_ms)
    }

    /// Obtiene el siguiente ID de request
    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Obtiene el tenant configurado
    pub fn tenant(&self) -> &str {
        &self.tenant
    }

    /// Obtiene el origin_host (hostname del servidor)
    pub fn origin_host(&self) -> &str {
        &self.origin_host
    }

    /// Ejecuta una llamada JSON-RPC a CGRateS
    ///
    /// # Arguments
    ///
    /// * `method` - Método JSON-RPC (ej: "SessionSv1.AuthorizeEvent")
    /// * `params` - Parámetros del método
    ///
    /// # Returns
    ///
    /// Resultado deserializado o error
    #[instrument(skip(self, params), fields(method = %method))]
    pub async fn call<T, R>(&self, method: &str, params: T) -> Result<R, CgratesError>
    where
        T: Serialize + std::fmt::Debug,
        R: DeserializeOwned,
    {
        let request_id = self.next_id();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: vec![params],
            id: request_id,
        };

        debug!(
            "CGRateS request: method={}, id={}",
            method, request_id
        );

        let response = self
            .http_client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    CgratesError::Timeout(50)
                } else {
                    CgratesError::Connection(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            error!("CGRateS HTTP error: status={}", status);
            return Err(CgratesError::HttpError(status.as_u16()));
        }

        let body = response.text().await.map_err(|e| {
            CgratesError::ParseError(format!("Failed to read response body: {}", e))
        })?;

        debug!("CGRateS response: {}", body);

        let rpc_response: JsonRpcResponse<R> = serde_json::from_str(&body).map_err(|e| {
            CgratesError::ParseError(format!("Failed to parse JSON: {} - Body: {}", e, body))
        })?;

        if let Some(err) = rpc_response.error {
            let error_msg = err.message.to_uppercase();

            // Mapear errores conocidos de CGRateS
            if error_msg.contains("NOT_ENOUGH_BALANCE") {
                return Err(CgratesError::InsufficientBalance);
            }
            if error_msg.contains("NOT_FOUND") {
                if error_msg.contains("ACCOUNT") {
                    return Err(CgratesError::AccountNotFound(error_msg));
                }
                return Err(CgratesError::RateNotFound(error_msg));
            }
            if error_msg.contains("UNAUTHORIZED") || error_msg.contains("DENIED") {
                return Err(CgratesError::AuthorizationDenied(err.message));
            }

            return Err(CgratesError::RpcError(err.code, err.message));
        }

        rpc_response.result.ok_or(CgratesError::EmptyResponse)
    }

    /// Verifica la conectividad con CGRateS
    pub async fn health_check(&self) -> Result<bool, CgratesError> {
        #[derive(Serialize)]
        struct PingArgs {}

        #[derive(Deserialize)]
        struct PingReply(String);

        let result: PingReply = self.call("CoreSv1.Ping", PingArgs {}).await?;
        Ok(result.0 == "Pong")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "cgrates.org",
            50,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_tenant() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "test.tenant",
            50,
        ).unwrap();

        assert_eq!(client.tenant(), "test.tenant");
    }

    #[test]
    fn test_request_id_increment() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "cgrates.org",
            50,
        ).unwrap();

        let id1 = client.next_id();
        let id2 = client.next_id();
        let id3 = client.next_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }
}
