//! AccountS API - Gestión de cuentas y saldos
//!
//! Proporciona métodos para:
//! - Consultar balances
//! - Agregar/establecer balances
//! - Sincronizar cuentas desde PostgreSQL

use rust_decimal::Decimal;
use tracing::{debug, info, instrument};

use super::client::{CgratesClient, CgratesError};
use super::types::*;

impl CgratesClient {
    /// Obtiene el balance monetario de una cuenta
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta
    ///
    /// # Returns
    ///
    /// Balance disponible
    #[instrument(skip(self))]
    pub async fn get_account_balance(
        &self,
        account: &str,
    ) -> Result<Decimal, CgratesError> {
        let args = CGRGetAccountArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
        };

        let reply: CGRAccountBalance = self.call("APIerSv2.GetAccount", args).await?;

        let balance = reply.monetary_balance();

        debug!("CGRateS account balance: account={}, balance={}", account, balance);

        Ok(balance)
    }

    /// Obtiene información completa de una cuenta
    #[instrument(skip(self))]
    pub async fn get_account(
        &self,
        account: &str,
    ) -> Result<CGRAccountBalance, CgratesError> {
        let args = CGRGetAccountArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
        };

        self.call("APIerSv2.GetAccount", args).await
    }

    /// Establece el balance de una cuenta
    ///
    /// Sobrescribe el balance actual.
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta
    /// * `amount` - Nuevo balance
    #[instrument(skip(self))]
    pub async fn set_balance(
        &self,
        account: &str,
        amount: Decimal,
    ) -> Result<(), CgratesError> {
        let args = CGRSetBalanceArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            balance_type: balance_types::MONETARY.to_string(),
            value: amount.to_string().parse().unwrap_or(0.0),
            balance: None,
        };

        let _: serde_json::Value = self.call("APIerSv1.SetBalance", args).await?;

        info!("CGRateS balance set: account={}, balance={}", account, amount);

        Ok(())
    }

    /// Agrega balance a una cuenta (topup)
    ///
    /// Suma al balance existente.
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta
    /// * `amount` - Cantidad a agregar
    #[instrument(skip(self))]
    pub async fn add_balance(
        &self,
        account: &str,
        amount: Decimal,
    ) -> Result<(), CgratesError> {
        let args = CGRAddBalanceArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            balance_type: balance_types::MONETARY.to_string(),
            value: amount.to_string().parse().unwrap_or(0.0),
        };

        let _: serde_json::Value = self.call("APIerSv1.AddBalance", args).await?;

        info!("CGRateS balance added: account={}, amount={}", account, amount);

        Ok(())
    }

    /// Descuenta balance de una cuenta
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta
    /// * `amount` - Cantidad a descontar (valor positivo)
    #[instrument(skip(self))]
    pub async fn debit_balance(
        &self,
        account: &str,
        amount: Decimal,
    ) -> Result<(), CgratesError> {
        // Para descontar, usamos AddBalance con valor negativo
        let negative_amount = -amount;

        let args = CGRAddBalanceArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            balance_type: balance_types::MONETARY.to_string(),
            value: negative_amount.to_string().parse().unwrap_or(0.0),
        };

        let _: serde_json::Value = self.call("APIerSv1.AddBalance", args).await?;

        info!("CGRateS balance debited: account={}, amount={}", account, amount);

        Ok(())
    }

    /// Crea o actualiza una cuenta en CGRateS
    ///
    /// # Arguments
    ///
    /// * `account` - Número de cuenta
    /// * `initial_balance` - Balance inicial
    /// * `disabled` - Si la cuenta está deshabilitada
    #[instrument(skip(self))]
    pub async fn set_account(
        &self,
        account: &str,
        initial_balance: Decimal,
        disabled: bool,
    ) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct SetAccountArgs {
            #[serde(rename = "Tenant")]
            tenant: String,
            #[serde(rename = "Account")]
            account: String,
            #[serde(rename = "ActionPlanId")]
            action_plan_id: Option<String>,
            #[serde(rename = "Overwrite")]
            overwrite: bool,
            #[serde(rename = "ReloadScheduler")]
            reload_scheduler: bool,
        }

        let args = SetAccountArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            action_plan_id: None,
            overwrite: true,
            reload_scheduler: false,
        };

        let _: serde_json::Value = self.call("APIerSv1.SetAccount", args).await?;

        // Establecer el balance inicial
        if initial_balance != Decimal::ZERO {
            self.set_balance(account, initial_balance).await?;
        }

        // Deshabilitar si es necesario
        if disabled {
            self.disable_account(account).await?;
        }

        info!("CGRateS account created/updated: account={}, balance={}", account, initial_balance);

        Ok(())
    }

    /// Deshabilita una cuenta (suspende)
    #[instrument(skip(self))]
    pub async fn disable_account(&self, account: &str) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct DisableArgs {
            #[serde(rename = "Tenant")]
            tenant: String,
            #[serde(rename = "Account")]
            account: String,
            #[serde(rename = "Disabled")]
            disabled: bool,
        }

        let args = DisableArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            disabled: true,
        };

        let _: serde_json::Value = self.call("APIerSv1.SetAccount", args).await?;

        info!("CGRateS account disabled: account={}", account);

        Ok(())
    }

    /// Habilita una cuenta (activa)
    #[instrument(skip(self))]
    pub async fn enable_account(&self, account: &str) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct EnableArgs {
            #[serde(rename = "Tenant")]
            tenant: String,
            #[serde(rename = "Account")]
            account: String,
            #[serde(rename = "Disabled")]
            disabled: bool,
        }

        let args = EnableArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
            disabled: false,
        };

        let _: serde_json::Value = self.call("APIerSv1.SetAccount", args).await?;

        info!("CGRateS account enabled: account={}", account);

        Ok(())
    }

    /// Elimina una cuenta de CGRateS
    #[instrument(skip(self))]
    pub async fn remove_account(&self, account: &str) -> Result<(), CgratesError> {
        #[derive(serde::Serialize)]
        struct RemoveArgs {
            #[serde(rename = "Tenant")]
            tenant: String,
            #[serde(rename = "Account")]
            account: String,
        }

        let args = RemoveArgs {
            tenant: self.tenant.clone(),
            account: account.to_string(),
        };

        let _: serde_json::Value = self.call("APIerSv1.RemoveAccount", args).await?;

        info!("CGRateS account removed: account={}", account);

        Ok(())
    }

    /// Verifica si una cuenta existe en CGRateS
    #[instrument(skip(self))]
    pub async fn account_exists(&self, account: &str) -> Result<bool, CgratesError> {
        match self.get_account(account).await {
            Ok(_) => Ok(true),
            Err(CgratesError::AccountNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Obtiene el uso acumulado de una cuenta
    #[instrument(skip(self))]
    pub async fn get_account_usage(
        &self,
        account: &str,
    ) -> Result<AccountUsage, CgratesError> {
        let balance_info = self.get_account(account).await?;

        // Parsear la información de uso del BalanceMap
        let monetary = balance_info.monetary_balance();

        Ok(AccountUsage {
            account: account.to_string(),
            monetary_balance: monetary,
            // CGRateS también puede trackear minutos, SMS, data, etc.
        })
    }
}

/// Información de uso de cuenta
#[derive(Debug, Clone)]
pub struct AccountUsage {
    pub account: String,
    pub monetary_balance: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_account_operations() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "cgrates.org",
            5000,
        ).unwrap();

        let test_account = "test_account_123";

        // 1. Create account
        let create_result = client.set_account(
            test_account,
            Decimal::new(10000, 2), // $100.00
            false,
        ).await;
        assert!(create_result.is_ok());

        // 2. Get balance
        let balance = client.get_account_balance(test_account).await.unwrap();
        assert_eq!(balance, Decimal::new(10000, 2));

        // 3. Add balance (topup)
        client.add_balance(test_account, Decimal::new(5000, 2)).await.unwrap();
        let new_balance = client.get_account_balance(test_account).await.unwrap();
        assert_eq!(new_balance, Decimal::new(15000, 2)); // $150.00

        // 4. Debit balance
        client.debit_balance(test_account, Decimal::new(2500, 2)).await.unwrap();
        let after_debit = client.get_account_balance(test_account).await.unwrap();
        assert_eq!(after_debit, Decimal::new(12500, 2)); // $125.00

        // 5. Cleanup
        client.remove_account(test_account).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_account_exists() {
        let client = CgratesClient::new(
            "http://localhost:2080/jsonrpc",
            "cgrates.org",
            5000,
        ).unwrap();

        let exists = client.account_exists("nonexistent_account_xyz").await.unwrap();
        assert!(!exists);
    }
}
