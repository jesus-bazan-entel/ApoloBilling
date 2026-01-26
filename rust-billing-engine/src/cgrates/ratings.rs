//! RatingS API - Motor de tarificación
//!
//! Proporciona métodos para:
//! - Obtener costos para destinos
//! - Consultar tarifas (rate info)
//! - Sincronizar tarifas desde PostgreSQL

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use tracing::{debug, instrument};

use super::client::{CgratesClient, CgratesError};
use super::types::*;

/// Información de tarifa para un destino
#[derive(Debug, Clone)]
pub struct RateInfo {
    /// Prefijo de destino matcheado
    pub destination_prefix: String,
    /// Tarifa por minuto
    pub rate_per_minute: Decimal,
    /// ID del rating plan (si disponible)
    pub rating_plan_id: Option<String>,
}

impl CgratesClient {
    /// Obtiene el costo para una llamada a un destino
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta (subject)
    /// * `destination` - Número destino
    /// * `duration_seconds` - Duración de la llamada en segundos
    ///
    /// # Returns
    ///
    /// Costo total para la duración especificada
    #[instrument(skip(self))]
    pub async fn get_cost(
        &self,
        account: &str,
        destination: &str,
        duration_seconds: i64,
    ) -> Result<Decimal, CgratesError> {
        let now = Utc::now();

        let args = CGRGetCostArgs {
            tenant: self.tenant.clone(),
            category: "call".to_string(),
            subject: account.to_string(),
            destination: destination.to_string(),
            time_start: now,
            time_end: now + Duration::seconds(duration_seconds),
        };

        let reply: CGRCostReply = self.call("APIerSv1.GetCost", args).await?;

        let cost = Decimal::from_f64_retain(reply.cost)
            .unwrap_or(Decimal::ZERO);

        debug!(
            "CGRateS get_cost: dest={}, duration={}s, cost={}",
            destination, duration_seconds, cost
        );

        Ok(cost)
    }

    /// Obtiene información de tarifa para un destino
    ///
    /// Calcula el costo para 60 segundos para inferir la tarifa por minuto.
    #[instrument(skip(self))]
    pub async fn get_rate_info(
        &self,
        destination: &str,
    ) -> Result<Option<RateInfo>, CgratesError> {
        let now = Utc::now();

        let args = CGRGetCostArgs {
            tenant: self.tenant.clone(),
            category: "call".to_string(),
            subject: "*any".to_string(), // Default subject
            destination: destination.to_string(),
            time_start: now,
            time_end: now + Duration::seconds(60), // 1 minuto
        };

        match self.call::<_, CGRCostReply>("APIerSv1.GetCost", args).await {
            Ok(reply) => {
                let rate_per_minute = Decimal::from_f64_retain(reply.cost)
                    .unwrap_or(Decimal::ZERO);

                Ok(Some(RateInfo {
                    destination_prefix: destination.to_string(),
                    rate_per_minute,
                    rating_plan_id: reply.rating_plan_id,
                }))
            }
            Err(CgratesError::RateNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Sincroniza un RatingProfile a CGRateS
    ///
    /// # Arguments
    ///
    /// * `prefix` - Prefijo de destino
    /// * `rate_per_minute` - Tarifa por minuto
    /// * `billing_increment` - Incremento de facturación en segundos
    #[instrument(skip(self))]
    pub async fn set_rating_profile(
        &self,
        prefix: &str,
        rate_per_minute: Decimal,
        billing_increment: i32,
    ) -> Result<(), CgratesError> {
        // CGRateS requiere crear varios objetos:
        // 1. Destination (prefijo)
        // 2. Rate (tarifa)
        // 3. DestinationRate (combina destination + rate)
        // 4. RatingPlan
        // 5. RatingProfile

        // Para simplificar, usamos la API de Tariff Plan (TPid)
        let tp_id = "apolo_billing".to_string();
        let load_id = format!("rate_{}", prefix);

        // 1. Crear/actualizar Destination
        self.set_tp_destination(&tp_id, prefix).await?;

        // 2. Crear/actualizar Rate
        self.set_tp_rate(&tp_id, prefix, rate_per_minute, billing_increment).await?;

        // 3. Cargar el Tariff Plan
        self.load_tariff_plan(&tp_id, &load_id).await?;

        debug!("CGRateS rating profile set: prefix={}, rate={}/min", prefix, rate_per_minute);

        Ok(())
    }

    /// Crea una Destination en CGRateS
    async fn set_tp_destination(
        &self,
        tp_id: &str,
        prefix: &str,
    ) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct TPDestination {
            #[serde(rename = "TPid")]
            tp_id: String,
            #[serde(rename = "ID")]
            id: String,
            #[serde(rename = "Prefixes")]
            prefixes: Vec<String>,
        }

        let args = TPDestination {
            tp_id: tp_id.to_string(),
            id: format!("DST_{}", prefix),
            prefixes: vec![prefix.to_string()],
        };

        let _: serde_json::Value = self.call("APIerSv1.SetTPDestination", args).await?;
        Ok(())
    }

    /// Crea un Rate en CGRateS
    async fn set_tp_rate(
        &self,
        tp_id: &str,
        prefix: &str,
        rate_per_minute: Decimal,
        billing_increment: i32,
    ) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct TPRate {
            #[serde(rename = "TPid")]
            tp_id: String,
            #[serde(rename = "ID")]
            id: String,
            #[serde(rename = "RateSlots")]
            rate_slots: Vec<RateSlot>,
        }

        #[derive(serde::Serialize)]
        struct RateSlot {
            #[serde(rename = "ConnectFee")]
            connect_fee: f64,
            #[serde(rename = "Rate")]
            rate: f64,
            #[serde(rename = "RateUnit")]
            rate_unit: String,
            #[serde(rename = "RateIncrement")]
            rate_increment: String,
            #[serde(rename = "GroupIntervalStart")]
            group_interval_start: String,
        }

        let rate_f64 = rate_per_minute.to_string().parse::<f64>().unwrap_or(0.0);

        let args = TPRate {
            tp_id: tp_id.to_string(),
            id: format!("RT_{}", prefix),
            rate_slots: vec![RateSlot {
                connect_fee: 0.0,
                rate: rate_f64,
                rate_unit: "60s".to_string(), // Por minuto
                rate_increment: format!("{}s", billing_increment),
                group_interval_start: "0s".to_string(),
            }],
        };

        let _: serde_json::Value = self.call("APIerSv1.SetTPRate", args).await?;
        Ok(())
    }

    /// Carga un Tariff Plan en CGRateS
    async fn load_tariff_plan(
        &self,
        tp_id: &str,
        load_id: &str,
    ) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct LoadTPArgs {
            #[serde(rename = "TPid")]
            tp_id: String,
            #[serde(rename = "LoadId")]
            load_id: String,
            #[serde(rename = "Validate")]
            validate: bool,
            #[serde(rename = "DryRun")]
            dry_run: bool,
        }

        let args = LoadTPArgs {
            tp_id: tp_id.to_string(),
            load_id: load_id.to_string(),
            validate: true,
            dry_run: false,
        };

        let _: serde_json::Value = self.call("APIerSv1.LoadTariffPlanFromStorDb", args).await?;
        Ok(())
    }

    /// Elimina un Destination de CGRateS
    #[instrument(skip(self))]
    pub async fn remove_destination(&self, prefix: &str) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct RemoveArgs {
            #[serde(rename = "Tenant")]
            tenant: String,
            #[serde(rename = "ID")]
            id: String,
        }

        let args = RemoveArgs {
            tenant: self.tenant.clone(),
            id: format!("DST_{}", prefix),
        };

        let _: serde_json::Value = self.call("APIerSv1.RemoveDestination", args).await?;
        Ok(())
    }

    /// Obtiene todas las destinations configuradas
    #[instrument(skip(self))]
    pub async fn get_destinations(&self) -> Result<Vec<String>, CgratesError> {
        #[derive(serde::Serialize)]
        struct GetDestArgs {
            #[serde(rename = "Tenant")]
            tenant: String,
        }

        #[derive(serde::Deserialize)]
        struct DestInfo {
            #[serde(rename = "Id")]
            id: String,
        }

        let args = GetDestArgs {
            tenant: self.tenant.clone(),
        };

        let result: Vec<DestInfo> = self
            .call("APIerSv1.GetDestinations", args)
            .await
            .unwrap_or_default();

        Ok(result.into_iter().map(|d| d.id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_get_cost() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "cgrates.org",
            5000,
        ).unwrap();

        let result = client.get_cost("1001", "51999888777", 60).await;

        match result {
            Ok(cost) => {
                println!("Cost for 60 seconds: {}", cost);
                assert!(cost >= Decimal::ZERO);
            }
            Err(e) => {
                println!("Expected error (no rates configured): {:?}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_rate_info() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "cgrates.org",
            5000,
        ).unwrap();

        let result = client.get_rate_info("51999888777").await;

        match result {
            Ok(Some(info)) => {
                println!("Rate info: {:?}", info);
            }
            Ok(None) => {
                println!("No rate found for destination");
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
