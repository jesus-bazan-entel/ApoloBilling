//! SessionS API - Control de sesiones en tiempo real
//!
//! Proporciona métodos para:
//! - Autorizar llamadas (pre-auth)
//! - Iniciar sesiones
//! - Actualizar sesiones (debitar durante llamada)
//! - Terminar sesiones

use chrono::{DateTime, Utc};
use tracing::{info, warn, instrument};

use super::client::{CgratesClient, CgratesError};
use super::types::*;

impl CgratesClient {
    /// Autoriza una nueva sesión (CHANNEL_CREATE)
    ///
    /// CGRateS verifica:
    /// - Que la cuenta exista
    /// - Que tenga balance suficiente
    /// - Que exista una tarifa para el destino
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta (account_number)
    /// * `destination` - Número destino
    /// * `origin_id` - UUID único de la llamada
    /// * `request_type` - Tipo de request (*prepaid, *postpaid)
    ///
    /// # Returns
    ///
    /// * `max_usage` - Tiempo máximo en nanosegundos
    /// * `error` - Mensaje de error si no autorizado
    #[instrument(skip(self), fields(account = %account, destination = %destination))]
    pub async fn authorize_session(
        &self,
        account: &str,
        destination: &str,
        origin_id: &str,
        request_type: &str,
    ) -> Result<CGRAuthorizationReply, CgratesError> {
        let args = CGRAuthorizationArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            destination: destination.to_string(),
            origin_id: origin_id.to_string(),
            origin_host: self.origin_host.clone(),
            request_type: request_type.to_string(),
            setup_time: Utc::now(),
            usage: None,
            category: Some("call".to_string()),
            subject: Some(account.to_string()),
        };

        let reply: CGRAuthorizationReply = self
            .call("SessionSv1.AuthorizeEvent", args)
            .await?;

        // Verificar si hay error en la respuesta
        if let Some(ref err) = reply.error {
            let error_upper = err.to_uppercase();
            if error_upper.contains("NOT_ENOUGH_BALANCE") {
                return Err(CgratesError::InsufficientBalance);
            }
            if error_upper.contains("NOT_FOUND") {
                return Err(CgratesError::RateNotFound(err.clone()));
            }
            return Err(CgratesError::AuthorizationDenied(err.clone()));
        }

        info!(
            "CGRateS authorized: origin_id={}, max_usage={:?}ns ({}s)",
            origin_id,
            reply.max_usage,
            reply.max_usage_seconds().unwrap_or(0)
        );

        Ok(reply)
    }

    /// Autoriza con un tiempo específico de uso
    ///
    /// Útil para verificar si hay balance para una duración específica
    #[instrument(skip(self))]
    pub async fn authorize_with_usage(
        &self,
        account: &str,
        destination: &str,
        origin_id: &str,
        request_type: &str,
        usage_seconds: i64,
    ) -> Result<CGRAuthorizationReply, CgratesError> {
        let args = CGRAuthorizationArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            destination: destination.to_string(),
            origin_id: origin_id.to_string(),
            origin_host: self.origin_host.clone(),
            request_type: request_type.to_string(),
            setup_time: Utc::now(),
            usage: Some(usage_seconds * 1_000_000_000), // Convertir a ns
            category: Some("call".to_string()),
            subject: Some(account.to_string()),
        };

        self.call("SessionSv1.AuthorizeEvent", args).await
    }

    /// Inicia una sesión cuando la llamada es contestada (CHANNEL_ANSWER)
    ///
    /// Debita el tiempo inicial y comienza el tracking de la sesión.
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta
    /// * `destination` - Número destino
    /// * `origin_id` - UUID de la llamada
    /// * `request_type` - Tipo de request
    /// * `answer_time` - Timestamp de contestación
    /// * `initial_debit_seconds` - Segundos a debitar inicialmente
    #[instrument(skip(self))]
    pub async fn init_session(
        &self,
        account: &str,
        destination: &str,
        origin_id: &str,
        request_type: &str,
        answer_time: DateTime<Utc>,
        initial_debit_seconds: i64,
    ) -> Result<CGRAuthorizationReply, CgratesError> {
        let args = CGRSessionInitArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            destination: destination.to_string(),
            origin_id: origin_id.to_string(),
            origin_host: self.origin_host.clone(),
            request_type: request_type.to_string(),
            answer_time,
            usage: initial_debit_seconds * 1_000_000_000, // ns
        };

        let reply: CGRAuthorizationReply = self
            .call("SessionSv1.InitiateSession", args)
            .await?;

        info!(
            "CGRateS session initiated: origin_id={}, initial_debit={}s, remaining={:?}s",
            origin_id,
            initial_debit_seconds,
            reply.max_usage_seconds()
        );

        Ok(reply)
    }

    /// Actualiza una sesión activa (debita tiempo adicional)
    ///
    /// Llamado periódicamente durante la llamada para extender el débito.
    ///
    /// # Arguments
    ///
    /// * `origin_id` - UUID de la llamada
    /// * `additional_seconds` - Segundos adicionales a debitar
    #[instrument(skip(self))]
    pub async fn update_session(
        &self,
        origin_id: &str,
        additional_seconds: i64,
    ) -> Result<CGRAuthorizationReply, CgratesError> {
        let args = CGRSessionUpdateArgs {
            tenant: self.tenant.clone(),
            origin_id: origin_id.to_string(),
            origin_host: self.origin_host.clone(),
            usage: additional_seconds * 1_000_000_000, // ns
        };

        let reply: CGRAuthorizationReply = self
            .call("SessionSv1.UpdateSession", args)
            .await?;

        // Verificar si el balance es insuficiente
        if let Some(ref err) = reply.error {
            if err.to_uppercase().contains("NOT_ENOUGH_BALANCE") {
                warn!("Session {} out of balance during update", origin_id);
                return Err(CgratesError::InsufficientBalance);
            }
        }

        info!(
            "CGRateS session updated: origin_id={}, debited={}s, remaining={:?}s",
            origin_id,
            additional_seconds,
            reply.max_usage_seconds()
        );

        Ok(reply)
    }

    /// Termina una sesión (CHANNEL_HANGUP)
    ///
    /// Realiza el débito final basado en el tiempo total de uso.
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta
    /// * `destination` - Número destino
    /// * `origin_id` - UUID de la llamada
    /// * `request_type` - Tipo de request
    /// * `answer_time` - Timestamp de contestación
    /// * `total_usage_seconds` - Tiempo total de uso (billsec)
    #[instrument(skip(self))]
    pub async fn terminate_session(
        &self,
        account: &str,
        destination: &str,
        origin_id: &str,
        request_type: &str,
        answer_time: DateTime<Utc>,
        total_usage_seconds: i64,
    ) -> Result<(), CgratesError> {
        let args = CGRSessionTerminateArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            destination: destination.to_string(),
            origin_id: origin_id.to_string(),
            origin_host: self.origin_host.clone(),
            request_type: request_type.to_string(),
            answer_time,
            usage: total_usage_seconds * 1_000_000_000, // ns
        };

        let _: serde_json::Value = self
            .call("SessionSv1.TerminateSession", args)
            .await?;

        info!(
            "CGRateS session terminated: origin_id={}, total_usage={}s",
            origin_id, total_usage_seconds
        );

        Ok(())
    }

    /// Obtiene información sobre las sesiones activas
    #[instrument(skip(self))]
    pub async fn get_active_sessions(&self) -> Result<Vec<serde_json::Value>, CgratesError> {
        #[derive(serde::Serialize)]
        struct GetSessionsArgs {
            #[serde(rename = "Tenant")]
            tenant: String,
            #[serde(rename = "Limit")]
            limit: Option<i32>,
        }

        let args = GetSessionsArgs {
            tenant: self.tenant.clone(),
            limit: Some(1000),
        };

        let reply: Vec<serde_json::Value> = self
            .call("SessionSv1.GetActiveSessions", args)
            .await
            .unwrap_or_else(|_| vec![]);

        Ok(reply)
    }

    /// Fuerza la terminación de una sesión específica
    #[instrument(skip(self))]
    pub async fn force_disconnect(
        &self,
        origin_id: &str,
    ) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct DisconnectArgs {
            #[serde(rename = "Tenant")]
            tenant: String,
            #[serde(rename = "OriginID")]
            origin_id: String,
        }

        let args = DisconnectArgs {
            tenant: self.tenant.clone(),
            origin_id: origin_id.to_string(),
        };

        let _: serde_json::Value = self
            .call("SessionSv1.ForceDisconnect", args)
            .await?;

        warn!("CGRateS forced disconnect: origin_id={}", origin_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Nota: Estos tests requieren un servidor CGRateS corriendo
    // Ejecutar con: cargo test --features integration-tests

    #[tokio::test]
    #[ignore]
    async fn test_authorize_session() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "cgrates.org",
            5000,
        ).unwrap();

        let result = client.authorize_session(
            "1001",
            "51999888777",
            "test-uuid-123",
            "*prepaid",
        ).await;

        match result {
            Ok(reply) => {
                assert!(reply.is_authorized());
                assert!(reply.max_usage.is_some());
            }
            Err(CgratesError::AccountNotFound(_)) => {
                // Esperado si no hay cuenta configurada
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_full_session_lifecycle() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "cgrates.org",
            5000,
        ).unwrap();

        let origin_id = format!("test-{}", uuid::Uuid::new_v4());
        let answer_time = Utc::now();

        // 1. Authorize
        let auth = client.authorize_session(
            "1001",
            "51999888777",
            &origin_id,
            "*prepaid",
        ).await;

        if auth.is_err() {
            println!("Skipping test - no CGRateS or account not configured");
            return;
        }

        // 2. Init session
        let init = client.init_session(
            "1001",
            "51999888777",
            &origin_id,
            "*prepaid",
            answer_time,
            60,
        ).await;
        assert!(init.is_ok());

        // 3. Update session
        let update = client.update_session(&origin_id, 60).await;
        assert!(update.is_ok());

        // 4. Terminate
        let terminate = client.terminate_session(
            "1001",
            "51999888777",
            &origin_id,
            "*prepaid",
            answer_time,
            120,
        ).await;
        assert!(terminate.is_ok());
    }
}
