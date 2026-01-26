# Análisis de Integración: CGRateS + ApoloBilling

## 1. Resumen Ejecutivo

Este documento analiza cómo integrar CGRateS con el sistema de billing ApoloBilling existente, utilizando los módulos principales de CGRateS:

- **SessionS**: Control de sesiones en tiempo real
- **RatingS**: Motor de tarificación (LPM)
- **AccountS**: Gestión de cuentas y saldos
- **CDRE**: Exportación de CDRs

### Modelo de Integración: Híbrido

```
┌─────────────────────────────────────────────────────────────────────┐
│  ARQUITECTURA HÍBRIDA                                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  FreeSWITCH ──ESL──▶ Rust Billing Engine ──JSON-RPC──▶ CGRateS     │
│                      [ORQUESTADOR]                    [MOTOR RATING]│
│                            │                               │        │
│                            ▼                               ▼        │
│                      PostgreSQL                     Redis + MySQL   │
│                   (CDRs, Config)                  (Balances, Rates) │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

El Rust Billing Engine se mantiene como **orquestador** de eventos ESL, mientras CGRateS asume la responsabilidad del **rating** y **accounting**.

---

## 2. Mapeo de Componentes

### 2.1 Componentes a Reemplazar

| Componente Actual | Archivo | Módulo CGRateS | Acción |
|-------------------|---------|----------------|--------|
| `AuthorizationService.get_rate()` | `authorization.rs:200-280` | RatingS | REEMPLAZAR |
| `ReservationManager.create_reservation()` | `reservation_manager.rs:50-150` | AccountS.MaxUsage | REEMPLAZAR |
| `ReservationManager.consume_reservation()` | `reservation_manager.rs:200-350` | SessionS.TerminateSession | REEMPLAZAR |
| `RealtimeBiller.monitor_call()` | `realtime_biller.rs:50-120` | SessionS.UpdateSession | REEMPLAZAR |

### 2.2 Componentes a Mantener

| Componente | Archivo | Razón |
|------------|---------|-------|
| `EventHandler` | `esl/event_handler.rs` | Interfaz ESL con FreeSWITCH |
| `CdrGenerator` | `services/cdr_generator.rs` | CDRs en PostgreSQL (principal) |
| `find_account_by_ani()` | `authorization.rs` | Lookup local para account_id |
| API REST | `rust-backend/` | Gestión de usuarios, zonas, etc. |

---

## 3. Estructura del Cliente CGRateS en Rust

### 3.1 Nuevo Módulo: `src/cgrates/`

```
rust-billing-engine/
├── src/
│   ├── cgrates/                    # NUEVO
│   │   ├── mod.rs                  # Exports
│   │   ├── client.rs               # HTTP JSON-RPC 2.0 client
│   │   ├── types.rs                # Estructuras de datos CGRateS
│   │   ├── sessions.rs             # SessionS API
│   │   ├── ratings.rs              # RatingS API
│   │   └── accounts.rs             # AccountS API
│   └── ...
```

### 3.2 Cliente JSON-RPC Base

```rust
// src/cgrates/client.rs

use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

pub struct CgratesClient {
    http_client: Client,
    base_url: String,
    tenant: String,
    request_id: AtomicU64,
}

impl CgratesClient {
    pub fn new(base_url: &str, tenant: &str, timeout_ms: u64) -> Result<Self, CgratesError> {
        let http_client = ClientBuilder::new()
            .timeout(Duration::from_millis(timeout_ms))
            .pool_max_idle_per_host(20)  // Connection pooling
            .build()?;

        Ok(Self {
            http_client,
            base_url: base_url.to_string(),
            tenant: tenant.to_string(),
            request_id: AtomicU64::new(1),
        })
    }

    pub async fn call<T, R>(&self, method: &str, params: T) -> Result<R, CgratesError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: vec![params],
            id: self.request_id.fetch_add(1, Ordering::SeqCst),
        };

        let response = self.http_client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await?
            .json::<JsonRpcResponse<R>>()
            .await?;

        if let Some(error) = response.error {
            return Err(CgratesError::RpcError(error.code, error.message));
        }

        response.result.ok_or(CgratesError::EmptyResponse)
    }
}
```

### 3.3 Tipos de Datos CGRateS

```rust
// src/cgrates/types.rs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Solicitud de autorización de sesión
#[derive(Debug, Serialize)]
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
    pub request_type: String,  // "*prepaid", "*postpaid", "*pseudoprepaid"

    #[serde(rename = "SetupTime")]
    pub setup_time: DateTime<Utc>,
}

/// Respuesta de autorización
#[derive(Debug, Deserialize)]
pub struct CGRAuthorizationReply {
    #[serde(rename = "MaxUsage")]
    pub max_usage: Option<i64>,  // Nanosegundos

    #[serde(rename = "ResourceAllocation")]
    pub resource_allocation: Option<String>,

    #[serde(rename = "Error")]
    pub error: Option<String>,
}

/// Evento de terminación de sesión
#[derive(Debug, Serialize)]
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
    pub usage: i64,  // Nanosegundos
}

/// Solicitud de costo
#[derive(Debug, Serialize)]
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
```

---

## 4. Flujos de Billing Modificados

### 4.1 Autorización (CHANNEL_CREATE)

**Flujo Actual:**
```
EventHandler → AuthorizationService.authorize()
                ├── find_account_by_ani()      [PostgreSQL]
                ├── get_rate() [LPM]           [PostgreSQL]
                └── create_reservation()        [PostgreSQL + Redis]
```

**Flujo con CGRateS:**
```
EventHandler → AuthorizationService.authorize_with_cgrates()
                ├── find_account_by_ani()      [PostgreSQL] (solo para account_id)
                └── cgrates.authorize_session() [CGRateS SessionS]
                    └── Internamente: RatingS + AccountS.MaxUsage
```

**Código Modificado:**

```rust
// src/services/authorization.rs

impl AuthorizationService {
    pub async fn authorize(&self, req: &AuthRequest) -> Result<AuthResponse, BillingError> {
        // Feature flag para migración gradual
        if self.config.cgrates_enabled {
            self.authorize_with_cgrates(req).await
        } else {
            self.authorize_legacy(req).await
        }
    }

    async fn authorize_with_cgrates(&self, req: &AuthRequest) -> Result<AuthResponse, BillingError> {
        let call_uuid = req.uuid.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

        // 1. Buscar cuenta localmente (para account_id y metadata)
        let account = self.find_account_by_ani(&req.caller).await?
            .ok_or(BillingError::AccountNotFound)?;

        // 2. Verificar estado localmente
        if account.status != AccountStatus::Active {
            return Ok(AuthResponse::denied("account_suspended", &call_uuid));
        }

        // 3. Determinar tipo de request
        let request_type = match account.account_type {
            AccountType::Prepaid => "*prepaid",
            AccountType::Postpaid => "*postpaid",
        };

        // 4. Llamar a CGRateS SessionS.AuthorizeEvent
        let cgr_reply = match self.cgrates.authorize_session(
            &account.account_number,
            &req.callee,
            &call_uuid,
            request_type,
        ).await {
            Ok(reply) => reply,
            Err(CgratesError::InsufficientBalance) => {
                return Ok(AuthResponse::denied("insufficient_balance", &call_uuid));
            }
            Err(CgratesError::RateNotFound(_)) => {
                return Ok(AuthResponse::denied("no_rate_found", &call_uuid));
            }
            Err(e) => {
                error!("CGRateS error: {}", e);
                // Fallback a sistema legacy si CGRateS falla
                if self.config.cgrates_fallback_enabled {
                    return self.authorize_legacy(req).await;
                }
                return Err(BillingError::Internal(e.to_string()));
            }
        };

        // 5. Calcular max_duration desde respuesta CGRateS
        let max_duration_seconds = cgr_reply.max_usage
            .map(|ns| (ns / 1_000_000_000) as i32)
            .unwrap_or(3600);

        // 6. Almacenar sesión en Redis (para tracking interno)
        self.store_session_info(&call_uuid, &account, max_duration_seconds).await?;

        Ok(AuthResponse {
            authorized: true,
            reason: "authorized".to_string(),
            uuid: call_uuid,
            account_id: Some(account.id.into()),
            account_number: Some(account.account_number),
            max_duration_seconds: Some(max_duration_seconds),
            // CGRateS maneja reservaciones internamente
            reservation_id: None,
            reserved_amount: None,
            rate_per_minute: None,  // Obtener de CGRateS si es necesario
        })
    }
}
```

### 4.2 Billing en Tiempo Real (Durante Llamada)

**Flujo con CGRateS:**
```
RealtimeBiller.monitor_call()
    └── Cada 180s: cgrates.update_session(origin_id, additional_usage)
        └── CGRateS debita automáticamente
        └── Si balance insuficiente → retorna error → hangup
```

```rust
// src/services/realtime_biller.rs

impl RealtimeBiller {
    async fn monitor_call_cgrates(
        cgrates: Arc<CgratesClient>,
        call_uuid: String,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(180));

        loop {
            interval.tick().await;

            // Actualizar sesión en CGRateS (debitar 180 segundos adicionales)
            match cgrates.update_session(&call_uuid, 180).await {
                Ok(reply) => {
                    if let Some(max_usage) = reply.max_usage {
                        let remaining_secs = max_usage / 1_000_000_000;
                        if remaining_secs < 60 {
                            warn!("Call {} low balance: {}s remaining", call_uuid, remaining_secs);
                        }
                    }
                }
                Err(CgratesError::InsufficientBalance) => {
                    warn!("Call {} out of balance, will be terminated", call_uuid);
                    // CGRateS enviará señal de desconexión
                    break;
                }
                Err(e) => {
                    error!("CGRateS update error: {}", e);
                    break;
                }
            }
        }
    }
}
```

### 4.3 Generación de CDR (CHANNEL_HANGUP)

**Flujo con CGRateS:**
```
CdrGenerator.generate_cdr()
    ├── cgrates.terminate_session()    [Débito final]
    ├── cgrates.get_cost()             [Costo exacto]
    ├── INSERT INTO cdrs               [PostgreSQL]
    └── sync_balance_from_cgrates()    [Sincronizar saldo]
```

```rust
// src/services/cdr_generator.rs

impl CdrGenerator {
    pub async fn generate_cdr(&self, event: HangupEvent) -> Result<i64, BillingError> {
        let session = self.get_session_info(&event.uuid).await?;

        // 1. Terminar sesión en CGRateS (débito final)
        if let Some(ref sess) = session {
            if let Some(answer_time) = event.answer_time {
                self.cgrates.terminate_session(
                    &sess.account_number,
                    &event.callee,
                    &event.uuid,
                    &sess.request_type,
                    answer_time,
                    event.billsec as i64,
                ).await.ok();  // Continuar aunque falle
            }
        }

        // 2. Obtener costo de CGRateS
        let cost = if let Some(ref sess) = session {
            self.cgrates.get_cost(
                &sess.account_number,
                &event.callee,
                event.billsec as i64,
            ).await.unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

        // 3. Insertar CDR en PostgreSQL (fuente primaria)
        let cdr_id = self.insert_cdr_to_postgres(&event, session.as_ref(), cost).await?;

        // 4. Sincronizar balance desde CGRateS a PostgreSQL
        if let Some(ref sess) = session {
            self.sync_balance_from_cgrates(&sess.account_number).await?;
        }

        Ok(cdr_id)
    }

    async fn sync_balance_from_cgrates(&self, account_number: &str) -> Result<(), BillingError> {
        let cgr_balance = self.cgrates.get_account_balance(account_number).await?;

        self.db_pool.get().await?.execute(
            "UPDATE accounts SET balance = $1, cgrates_synced_at = NOW() WHERE account_number = $2",
            &[&cgr_balance, &account_number],
        ).await?;

        Ok(())
    }
}
```

---

## 5. Configuración de CGRateS

### 5.1 Variables de Entorno Nuevas

```bash
# .env additions for rust-billing-engine

# CGRateS Integration
CGRATES_ENABLED=true
CGRATES_URL=http://127.0.0.1:2080/jsonrpc
CGRATES_TENANT=cgrates.org
CGRATES_TIMEOUT_MS=50
CGRATES_FALLBACK_ENABLED=true  # Fallback a billing legacy si CGRateS falla
```

### 5.2 Configuración CGRateS (cgrates.json)

```json
{
  "general": {
    "node_id": "apolo_billing",
    "log_level": 6,
    "rounding_decimals": 4
  },
  "listen": {
    "rpc_json": "127.0.0.1:2080"
  },
  "sessions": {
    "enabled": true,
    "resources_conns": ["*internal"],
    "rals_conns": ["*internal"],
    "cdrs_conns": ["*internal"],
    "debit_interval": "180s",
    "min_dur_low_balance": "30s"
  },
  "rals": {
    "enabled": true
  },
  "cdrs": {
    "enabled": true,
    "store_cdrs": true
  },
  "data_db": {
    "db_type": "*redis",
    "db_host": "127.0.0.1",
    "db_port": 6379,
    "db_name": "10"
  },
  "stor_db": {
    "db_type": "*mysql",
    "db_host": "127.0.0.1",
    "db_port": 3306,
    "db_name": "cgrates",
    "db_user": "cgrates",
    "db_password": "CGRateS.org"
  }
}
```

---

## 6. Sincronización de Datos

### 6.1 Sincronización de Tarifas: PostgreSQL → CGRateS

Las tarifas se gestionan en el frontend React y se almacenan en `rate_cards`.
Se necesita un servicio de sincronización hacia CGRateS:

```rust
// src/services/rate_sync.rs

pub struct RateSyncService {
    db_pool: DbPool,
    cgrates: Arc<CgratesClient>,
}

impl RateSyncService {
    /// Sincroniza todas las rate_cards a CGRateS RatingProfiles
    pub async fn sync_all_rates(&self) -> Result<usize, BillingError> {
        let client = self.db_pool.get().await?;

        let rows = client.query(
            "SELECT destination_prefix, destination_name, rate_per_minute,
                    billing_increment, connection_fee
             FROM rate_cards
             WHERE effective_end IS NULL OR effective_end > NOW()",
            &[],
        ).await?;

        let mut synced = 0;
        for row in rows {
            let prefix: String = row.get(0);
            let rate: Decimal = row.get(2);
            let increment: i32 = row.get(3);

            // Crear RatingProfile en CGRateS
            self.cgrates.set_rating_profile(&prefix, rate, increment).await?;
            synced += 1;
        }

        info!("Synced {} rates to CGRateS", synced);
        Ok(synced)
    }
}
```

### 6.2 Sincronización de Cuentas: PostgreSQL → CGRateS

```rust
// src/services/account_sync.rs

impl AccountSyncService {
    /// Sincroniza una cuenta a CGRateS AccountS
    pub async fn sync_account(&self, account_number: &str) -> Result<(), BillingError> {
        let client = self.db_pool.get().await?;

        let row = client.query_one(
            "SELECT balance, account_type FROM accounts WHERE account_number = $1",
            &[&account_number],
        ).await?;

        let balance: Decimal = row.get(0);

        // Establecer balance en CGRateS
        self.cgrates.set_balance(account_number, balance).await?;

        Ok(())
    }

    /// Sincroniza balance desde CGRateS a PostgreSQL
    pub async fn sync_from_cgrates(&self, account_number: &str) -> Result<(), BillingError> {
        let cgr_balance = self.cgrates.get_account_balance(account_number).await?;

        let client = self.db_pool.get().await?;
        client.execute(
            "UPDATE accounts SET balance = $1, cgrates_synced_at = NOW()
             WHERE account_number = $2",
            &[&cgr_balance, &account_number],
        ).await?;

        Ok(())
    }
}
```

---

## 7. Cambios en Base de Datos

### 7.1 Nuevas Columnas

```sql
-- Agregar referencia a CGRateS en accounts
ALTER TABLE accounts
ADD COLUMN cgrates_tenant VARCHAR(50) DEFAULT 'cgrates.org',
ADD COLUMN cgrates_synced_at TIMESTAMP WITH TIME ZONE;

-- Agregar flag de exportación en CDRs
ALTER TABLE cdrs
ADD COLUMN cgrates_exported BOOLEAN DEFAULT FALSE,
ADD COLUMN cgrates_cgrid VARCHAR(100);

-- Índice para sincronización
CREATE INDEX idx_accounts_cgrates_sync ON accounts(cgrates_synced_at);
CREATE INDEX idx_cdrs_cgrates_export ON cdrs(cgrates_exported) WHERE cgrates_exported = FALSE;
```

### 7.2 Tabla de Log de Sincronización

```sql
CREATE TABLE cgrates_sync_log (
    id BIGSERIAL PRIMARY KEY,
    entity_type VARCHAR(50) NOT NULL,    -- 'account', 'rate', 'destination'
    entity_id VARCHAR(100) NOT NULL,
    operation VARCHAR(20) NOT NULL,       -- 'create', 'update', 'delete', 'sync'
    direction VARCHAR(20) NOT NULL,       -- 'to_cgrates', 'from_cgrates'
    sync_status VARCHAR(20) NOT NULL,     -- 'pending', 'success', 'failed'
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    synced_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_cgrates_sync_pending ON cgrates_sync_log(sync_status)
WHERE sync_status = 'pending';
```

---

## 8. Plan de Implementación

### Fase 1: Infraestructura (Semana 1-2)

| Tarea | Archivos | Prioridad |
|-------|----------|-----------|
| Instalar CGRateS (Docker o nativo) | Docker Compose | P0 |
| Crear módulo `src/cgrates/` | `mod.rs`, `client.rs`, `types.rs` | P0 |
| Implementar cliente JSON-RPC | `client.rs` | P0 |
| Agregar configuración CGRateS | `config.rs`, `.env` | P0 |
| Tests unitarios del cliente | `tests/cgrates_test.rs` | P1 |

### Fase 2: SessionS (Semana 3-4)

| Tarea | Archivos | Prioridad |
|-------|----------|-----------|
| Wrapper SessionS | `cgrates/sessions.rs` | P0 |
| Modificar AuthorizationService | `services/authorization.rs` | P0 |
| Modificar RealtimeBiller | `services/realtime_biller.rs` | P0 |
| Feature flag `cgrates_enabled` | `config.rs` | P0 |
| Tests de integración | `tests/integration/` | P1 |

### Fase 3: RatingS + AccountS (Semana 5-6)

| Tarea | Archivos | Prioridad |
|-------|----------|-----------|
| Wrapper RatingS | `cgrates/ratings.rs` | P0 |
| Wrapper AccountS | `cgrates/accounts.rs` | P0 |
| Servicio de sincronización | `services/rate_sync.rs`, `services/account_sync.rs` | P0 |
| Modificar CdrGenerator | `services/cdr_generator.rs` | P1 |
| Migración de tarifas | Script SQL/Rust | P1 |

### Fase 4: Testing y Producción (Semana 7-8)

| Tarea | Prioridad |
|-------|-----------|
| Load testing (500 llamadas concurrentes) | P0 |
| Benchmarks de latencia (< 50ms) | P0 |
| Testing de failover (CGRateS caído) | P0 |
| Documentación | P1 |
| Deployment a producción | P1 |

---

## 9. Consideraciones de Rendimiento

### 9.1 Requisitos del PRD

| Métrica | Target | Estrategia |
|---------|--------|------------|
| Latencia autorización | < 50ms | Connection pooling, timeout 50ms |
| Llamadas concurrentes | 500+ | CGRateS SessionS + Redis |
| Pérdida de CDRs | 0% | PostgreSQL primario, CDRE secundario |

### 9.2 Optimizaciones

```rust
// Connection pooling para CGRateS
let http_client = ClientBuilder::new()
    .pool_max_idle_per_host(20)           // 20 conexiones idle
    .pool_idle_timeout(Duration::from_secs(90))
    .timeout(Duration::from_millis(50))   // 50ms timeout
    .tcp_keepalive(Duration::from_secs(60))
    .build()?;
```

### 9.3 Fallback a Sistema Legacy

```rust
pub struct CgratesClientWithFallback {
    cgrates: Arc<CgratesClient>,
    legacy: Arc<LegacyAuthorizationService>,
    fallback_enabled: bool,
}

impl CgratesClientWithFallback {
    pub async fn authorize(&self, req: &AuthRequest) -> Result<AuthResponse, BillingError> {
        match self.cgrates.authorize_session(...).await {
            Ok(reply) => Ok(reply.into()),
            Err(e) if self.fallback_enabled => {
                warn!("CGRateS unavailable, using legacy: {}", e);
                self.legacy.authorize(req).await
            }
            Err(e) => Err(BillingError::Internal(e.to_string())),
        }
    }
}
```

---

## 10. Resumen de Cambios por Archivo

### rust-billing-engine/

| Archivo | Acción | Descripción |
|---------|--------|-------------|
| `src/cgrates/mod.rs` | **NUEVO** | Módulo CGRateS |
| `src/cgrates/client.rs` | **NUEVO** | Cliente JSON-RPC |
| `src/cgrates/types.rs` | **NUEVO** | Tipos de datos CGRateS |
| `src/cgrates/sessions.rs` | **NUEVO** | Wrapper SessionS |
| `src/cgrates/ratings.rs` | **NUEVO** | Wrapper RatingS |
| `src/cgrates/accounts.rs` | **NUEVO** | Wrapper AccountS |
| `src/config.rs` | MODIFICAR | Agregar config CGRateS |
| `src/services/authorization.rs` | MODIFICAR | Usar CGRateS SessionS |
| `src/services/realtime_biller.rs` | MODIFICAR | Usar CGRateS UpdateSession |
| `src/services/cdr_generator.rs` | MODIFICAR | Agregar TerminateSession |
| `src/services/reservation_manager.rs` | DEPRECAR | Reemplazado por AccountS |
| `Cargo.toml` | MODIFICAR | Agregar dependencias |

### Dependencias Nuevas (Cargo.toml)

```toml
[dependencies]
# ... existentes ...

# CGRateS Integration
hostname = "0.3"
thiserror = "1.0"  # Si no existe
```

---

## 11. Próximos Pasos Recomendados

1. **Instalar CGRateS** en ambiente de desarrollo (Docker recomendado)
2. **Crear estructura básica** del módulo `cgrates/`
3. **Implementar cliente JSON-RPC** con tests unitarios
4. **Configurar CGRateS** con tarifas de prueba
5. **Probar flujo de autorización** end-to-end
6. **Migrar gradualmente** con feature flag
7. **Load testing** antes de producción

---

## 12. Ventajas de la Integración

| Aspecto | Sistema Actual | Con CGRateS |
|---------|---------------|-------------|
| **Rating** | LPM custom en PostgreSQL | Motor dedicado, caching |
| **Accounting** | ReservationManager custom | AccountS con transacciones |
| **Escalabilidad** | Limitada por DB queries | CGRateS distribuido |
| **Real-time debiting** | Loop cada 180s | Integrado en SessionS |
| **Mantenimiento** | Todo en Rust | Rating/Accounting en CGRateS |

## 13. Desventajas / Trade-offs

| Aspecto | Consideración |
|---------|---------------|
| **Complejidad** | Nuevo componente (CGRateS) a operar |
| **Latencia** | Llamada HTTP adicional (~10-20ms) |
| **Sincronización** | Mantener consistencia PostgreSQL ↔ CGRateS |
| **Curva de aprendizaje** | Configuración de CGRateS |
| **Dependencia** | Sistema depende de CGRateS disponible |
