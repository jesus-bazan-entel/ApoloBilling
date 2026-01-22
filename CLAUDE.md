# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ApoloBilling is a real-time telecommunications billing platform for FreeSWITCH PBX environments. It handles call authorization, balance reservations, real-time billing, and CDR (Call Detail Records) generation.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      React Frontend (:3000)                      │
│                    (Proxy → localhost:8000)                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Rust Backend (:8000)                        │
│  - Auth (JWT + Argon2)        - CDRs (queries, export, stats)   │
│  - Accounts CRUD + Topup      - Active Calls                    │
│  - Rate Cards CRUD + LPM      - Reservations                    │
│  - Zones/Prefixes/Tariffs     - Dashboard Stats                 │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│   Rust Billing Engine   │     │   PostgreSQL + Redis    │
│        (:9000)          │     │                         │
│   ESL, real-time        │     │                         │
│   billing               │     │                         │
└─────────────────────────┘     └─────────────────────────┘
```

**Three main components:**
- **frontend/** - React 19 + TypeScript + Vite SPA for dashboard UI
- **rust-backend/** - High-performance Rust API (Actix-web) for all CRUD operations, CDR queries, authentication
- **rust-billing-engine/** - Real-time billing processor via FreeSWITCH ESL

## Common Commands

### Rust Backend (Main API Server)
```bash
cd rust-backend
cp .env.example .env              # Configure database URL and settings
cargo build --release
cargo run --release               # Start server on :8000
cargo test                        # Run tests
```

### Frontend (React/TypeScript)
```bash
cd frontend
npm install
npm run dev        # Dev server on :3000 (proxies API to :8000)
npm run build      # Production build (runs tsc then vite build)
npm run lint       # ESLint check
npm run preview    # Preview production build
```

### Rust Billing Engine
```bash
cd rust-billing-engine
cargo build --release
cargo run                         # Start billing engine on :9000
cargo test                        # Run all tests
cargo test --test full_integration_test   # Run integration tests
```

### Rust Billing Engine - Systemd Service

El motor de billing se ejecuta como servicio systemd (`apolo-billing-engine.service`).

```bash
# Recargar systemd (después de modificar el archivo .service)
systemctl daemon-reload

# Habilitar inicio automático
systemctl enable apolo-billing-engine

# Iniciar el servicio
systemctl start apolo-billing-engine

# Verificar estado
systemctl status apolo-billing-engine
```

**Comandos útiles:**
```bash
# Ver logs en tiempo real
journalctl -u apolo-billing-engine -f

# Reiniciar servicio
systemctl restart apolo-billing-engine

# Detener servicio
systemctl stop apolo-billing-engine

# Ver últimos logs
journalctl -u apolo-billing-engine -n 100
```

**Archivo de servicio:** `/etc/systemd/system/apolo-billing-engine.service`

## Key Entry Points

- `rust-backend/src/main.rs` - Actix-web server setup, all API routes
- `frontend/src/App.tsx` - React router and main app component
- `rust-billing-engine/src/main.rs` - Billing engine initialization

## Database Schema (PostgreSQL)

Core tables in `apolo_billing` database:
- `accounts` - Customer accounts (prepaid/postpaid, balance, credit_limit)
- `rate_cards` - Destination rates by prefix (LPM matching)
- `cdrs` - Call detail records with cost, duration, hangup cause
- `balance_reservations` - Active call balance holds
- `balance_transactions` - Recharges, consumptions, refunds

## Billing Flow

1. **CHANNEL_CREATE** → `AuthorizationService` validates account, reserves initial balance
2. **CHANNEL_ANSWER** → `RealtimeBiller` starts periodic reservation extensions
3. **CHANNEL_HANGUP_COMPLETE** → `CdrGenerator` creates CDR, commits balance consumption

## Frontend Structure

```
frontend/src/
├── pages/                # Route components (Dashboard, CDR, Accounts, Rates, etc.)
├── components/           # Reusable UI (Layout, DataTable, StatCard)
├── services/api.ts       # Axios client, all API endpoints
└── lib/utils.ts          # Utility functions
```

## Rust Backend Structure (Main API)

```
rust-backend/
├── src/main.rs           # Actix-web server setup, route configuration
└── crates/
    ├── apolo-api/        # HTTP handlers and DTOs
    │   ├── handlers/     # All API endpoint handlers
    │   │   ├── auth.rs        # Login/logout/me/register
    │   │   ├── account.rs     # CRUD cuentas + topup
    │   │   ├── rate_card.rs   # Rate cards CRUD + LPM search
    │   │   ├── rate.rs        # Rates legacy endpoints
    │   │   ├── cdr.rs         # CDRs list, get, export, stats
    │   │   ├── active_call.rs # Active calls + create CDR
    │   │   ├── management.rs  # Zonas/Prefijos/Tarifas
    │   │   ├── dashboard.rs   # Dashboard stats
    │   │   └── reservation.rs # Balance reservations
    │   └── dto/               # Request/response types
    ├── apolo-db/         # PostgreSQL repositories
    ├── apolo-auth/       # JWT + Argon2 authentication
    ├── apolo-cache/      # Redis caching layer
    └── apolo-core/       # Shared models and traits
```

**API Endpoints (Port 8000):**

Authentication:
- `POST /api/v1/auth/login` - Login, returns JWT in cookie
- `POST /api/v1/auth/logout` - Clear cookie
- `GET /api/v1/auth/me` - Current user info
- `POST /api/v1/auth/register` - Create user (admin only)

Accounts:
- `GET /api/v1/accounts` - List with pagination
- `POST /api/v1/accounts` - Create account
- `GET/PUT /api/v1/accounts/{id}` - Get/update account
- `POST /api/v1/accounts/{id}/topup` - Add balance

Rate Cards:
- `GET /api/v1/rate-cards` - List with filters
- `POST /api/v1/rate-cards` - Create rate
- `GET/PUT/DELETE /api/v1/rate-cards/{id}` - CRUD operations
- `POST /api/v1/rate-cards/bulk` - Bulk import
- `GET /api/v1/rate-cards/search/{phone}` - LPM search

CDRs:
- `GET /api/v1/cdrs` - List with pagination and filters
- `GET /api/v1/cdrs/{id}` - Get single CDR
- `GET /api/v1/cdrs/export` - Streaming export (CSV/JSON/JSONL)
- `GET /api/v1/cdrs/stats` - Aggregated statistics
- `POST /api/v1/cdrs` - Create CDR

Active Calls & Reservations:
- `GET /api/v1/active-calls` - List active calls
- `POST /api/v1/active-calls` - Report/upsert call
- `DELETE /api/v1/active-calls/{call_id}` - Remove call
- `GET /api/v1/reservations` - List reservations

Management (Zones/Prefixes/Tariffs):
- `GET/POST /api/v1/zonas` - CRUD zones
- `GET/POST /api/v1/prefijos` - CRUD prefixes
- `GET/POST /api/v1/tarifas` - CRUD tariffs
- `POST /api/v1/sync-rate-cards` - Sync rate cards

Dashboard:
- `GET /api/v1/stats` - Dashboard statistics
- `GET /api/v1/health` - Health check

## Rust Billing Engine Structure

```
rust-billing-engine/src/
├── services/             # Core business logic
│   ├── authorization.rs  # Call authorization
│   ├── realtime_biller.rs # Active call billing
│   ├── cdr_generator.rs  # CDR creation
│   └── reservation_manager.rs # Balance management
├── esl/                  # FreeSWITCH Event Socket Layer
├── api/                  # HTTP API routes
├── database/             # PostgreSQL queries and pool
├── models/               # Data structures
└── cache.rs              # Redis client
```

## Environment Variables

```bash
# Database (shared by all components)
DATABASE_URL=postgresql://apolo_user:PASSWORD@localhost:5432/apolo_billing
DATABASE_MAX_CONNECTIONS=20
REDIS_URL=redis://localhost:6379

# Rust Backend (Main API)
RUST_SERVER_HOST=0.0.0.0
RUST_SERVER_PORT=8000
RUST_SERVER_WORKERS=4
CORS_ORIGINS=http://localhost:3000,http://127.0.0.1:3000

# JWT Authentication
JWT_SECRET=apolo-billing-secret-key-change-in-production
JWT_EXPIRATION_SECS=1800

# Rust Billing Engine
ESL_HOST=127.0.0.1
ESL_PORT=8021

# Logging
LOG_LEVEL=info
RUST_LOG=apolo_billing=debug,apolo_api=debug,actix_web=info
```

## Testing

```bash
# Rust billing engine - has ESL simulator for testing without FreeSWITCH
cd rust-billing-engine
cargo test

# Frontend linting
cd frontend
npm run lint
```

## Rate Matching

Rates use Longest Prefix Match (LPM) on `destination_prefix`. The most specific matching prefix wins based on priority.

---

# FLUJO DE TARIFICACIÓN DEL MOTOR DE BILLING

## Visión General

El motor de billing (`rust-billing-engine`) procesa llamadas en **3 fases principales**, manejando eventos ESL (Event Socket Layer) de FreeSWITCH:

```
┌─────────────────────────────────────────────────────────────┐
│  FLUJO COMPLETO DE TARIFICACIÓN DE UNA LLAMADA             │
└─────────────────────────────────────────────────────────────┘

1️⃣  CHANNEL_CREATE (Incoming Call)
    └─ Authorize call, verify account, find rate (LPM)
       └─ Create initial balance reservation
          └─ max_duration calculado

2️⃣  CHANNEL_ANSWER (Call Connected)
    └─ Start realtime monitoring
       └─ Check every 180s if extension needed

3️⃣  CHANNEL_HANGUP_COMPLETE (Call Ended)
    └─ Stop monitoring
       └─ Generate CDR + consume reservation
          └─ Update account balance
```

## Fase 1: CHANNEL_CREATE - Autorización

**Archivo:** `rust-billing-engine/src/services/authorization.rs`

Cuando entra una llamada, `AuthorizationService.authorize()` ejecuta:

```
1. Busca cuenta por ANI (número llamante)
   └─ Query: SELECT * FROM accounts WHERE account_number = $1
   └─ Si no existe → DENY (reason: "account_not_found")

2. Verifica estado de cuenta
   └─ Si status != 'active' → DENY (reason: "account_suspended")

3. Busca tarifa con LPM (Longest Prefix Match)
   └─ Ver sección "Algoritmo LPM" abajo
   └─ Si no hay tarifa → DENY (reason: "no_rate_found")

4. Crea reserva de balance
   └─ ReservationManager.create_reservation()
   └─ Si balance insuficiente → DENY (reason: "insufficient_balance")
```

### Algoritmo LPM (Longest Prefix Match)

Para destino `541156000`:

```
Step 1: Generar todos los prefijos
        ["5", "54", "541", "5411", "54115", "541156", ...]

Step 2: Query Database
        SELECT * FROM rate_cards
        WHERE destination_prefix IN (ALL prefixes)
        AND effective_start <= NOW()
        AND (effective_end IS NULL OR effective_end >= NOW())
        ORDER BY LENGTH(destination_prefix) DESC, priority DESC
        LIMIT 1

Step 3: El prefijo más largo que coincida GANA
        Ejemplo: si existe "5411" y "54115", prefiere "54115"
```

### Cálculo de Reserva Inicial

```
base = rate_per_minute × 5 minutos
buffer = base × 8%
total = clamp(base + buffer, $0.30, $30.00)
max_duration = (total / rate_per_minute) × 60 segundos
```

**Ejemplo:**
- Rate: $0.15/min
- base = 0.15 × 5 = $0.75
- buffer = $0.75 × 8% = $0.06
- total = $0.81
- max_duration = ($0.81 / 0.15) × 60 = 324 segundos (~5.4 min)

## Fase 2: CHANNEL_ANSWER - Monitoreo en Tiempo Real

**Archivo:** `rust-billing-engine/src/services/realtime_biller.rs`

`RealtimeBiller` inicia un loop de monitoreo cada **180 segundos**:

```
for each active call:
    time_remaining = max_duration - elapsed

    if (time_remaining < 240 segundos):
        ReservationManager.extend_reservation(+3 minutos)
        └─ INSERT nueva fila en balance_reservations (type='extension')
        └─ Actualiza max_duration en Redis
```

Esto permite llamadas largas sin cortar prematuramente por agotamiento de reserva.

## Fase 3: CHANNEL_HANGUP_COMPLETE - Generación CDR

**Archivo:** `rust-billing-engine/src/services/cdr_generator.rs`

### Cálculo de Costo

```
1. Obtener billsec (segundos facturables) del evento ESL
2. Redondear al billing_increment de la tarifa:
   billsec_rounded = ceil(billsec / increment) × increment
3. Convertir a minutos:
   minutes = billsec_rounded / 60
4. Calcular costo:
   cost = minutes × rate_per_minute
```

**Ejemplo:**
- billsec: 45 segundos
- billing_increment: 6 segundos
- billsec_rounded = ceil(45/6) × 6 = 48 segundos
- minutes = 48/60 = 0.8
- cost = 0.8 × $0.15 = $0.12

### Inserción CDR

```sql
INSERT INTO cdrs (
    uuid, account_id, caller, callee,
    start_time, answer_time, end_time,
    duration, billsec, hangup_cause,
    rate_applied, cost, direction
) VALUES (...)
```

### Consumo de Reserva

**Archivo:** `rust-billing-engine/src/services/reservation_manager.rs`

```
ReservationManager.consume_reservation():
│
├─ BEGIN TRANSACTION (con row locks)
│
├─ Query reservas activas para la llamada:
│   SELECT * FROM balance_reservations
│   WHERE call_uuid = $1 AND status = 'active'
│   FOR UPDATE
│
├─ Comparar actual_cost vs total_reserved:
│
│   CASO NORMAL (actual_cost <= reserved):
│   ├─ UPDATE balance_reservations SET consumed_amount = actual_cost
│   ├─ UPDATE accounts SET balance = balance - actual_cost
│   └─ INSERT INTO balance_transactions (log)
│
│   CASO DEFICIT (actual_cost > reserved):
│   ├─ Marcar todas las reservas como fully_consumed
│   ├─ Descontar costo COMPLETO del balance (puede ir negativo)
│   ├─ Si deficit > $10.00 → AUTO-SUSPEND cuenta
│   └─ Log deficit transaction
│
└─ COMMIT + cleanup Redis
```

## Gestión de Reservas (Balance Reservations)

### Ciclo de Vida

```
1. CREATE (CHANNEL_CREATE)
   └─ INSERT balance_reservations (status='active', type='initial')

2. EXTEND (durante llamada activa)
   └─ INSERT balance_reservations (status='active', type='extension')

3. CONSUME (CHANNEL_HANGUP_COMPLETE)
   └─ UPDATE status='partially_consumed' o 'fully_consumed'
   └─ Liberar amount no usado al balance

4. EXPIRED (cleanup job)
   └─ UPDATE status='expired' si expires_at < NOW()
```

### Control de Concurrencia

- Máximo 5 llamadas simultáneas por cuenta (configurable)
- Verificado en Redis: `SCARD active_reservations:{account_id}`

## Ejemplo Completo de una Llamada

```
Cuenta: balance=$10.00, status=active
Llamada: 5491234567890 → 5411567890 (Buenos Aires)
Tarifa match: "5411" → $0.15/min, increment=6s
Duración real: 45 segundos

Timeline:
─────────────────────────────────────────────────────────
T+0ms     CHANNEL_CREATE
          ├─ Account found ✓
          ├─ Rate found: $0.15/min ✓
          ├─ Reserva: $0.81 (5min + 8% buffer)
          └─ max_duration: 324s

T+50ms    CHANNEL_ANSWER
          └─ Monitoreo iniciado (check cada 180s)

T+45000ms CHANNEL_HANGUP_COMPLETE
          ├─ billsec: 45s
          ├─ billsec_rounded: 48s (ceil(45/6)*6)
          ├─ cost: $0.12 (0.8min × $0.15)
          ├─ CDR insertado
          ├─ Reserva consumida: $0.12
          └─ Balance: $10.00 - $0.12 = $9.88
─────────────────────────────────────────────────────────
```

## Tablas de Base de Datos Involucradas

| Tabla | Propósito |
|-------|-----------|
| `accounts` | Saldo, estado, tipo (prepaid/postpaid) |
| `rate_cards` | Tarifas por prefijo (LPM) |
| `balance_reservations` | Reservas activas durante llamadas |
| `cdrs` | Registros de facturación finales |
| `balance_transactions` | Log de movimientos de balance |

## Archivos Clave del Motor

| Archivo | Función |
|---------|---------|
| `services/authorization.rs` | Autoriza llamadas, busca tarifas LPM |
| `services/realtime_biller.rs` | Monitorea llamadas activas, extiende reservas |
| `services/cdr_generator.rs` | Genera CDR, calcula costo final |
| `services/reservation_manager.rs` | CRUD de reservas de balance |
| `esl/event_handler.rs` | Procesa eventos ESL de FreeSWITCH |

## Manejo de Errores

| Escenario | Acción |
|-----------|--------|
| Cuenta no encontrada | DENY + uuid_kill |
| Cuenta suspendida | DENY + uuid_kill |
| Sin tarifa para destino | DENY + uuid_kill |
| Balance insuficiente | DENY + uuid_kill |
| Límite concurrencia excedido | DENY + uuid_kill |
| Deficit > $10.00 | Auto-suspend cuenta |

## Cache Redis

```
rate:{prefix}              → Rate card data (TTL: 300s)
call_session:{uuid}        → Session metadata (max_duration, start_time)
reservation:{id}           → Reservation data (TTL: 2700s)
active_reservations:{id}   → SET de reservation IDs por cuenta
```

---

# SISTEMA DE RESERVA DE SALDO (Balance Reservations)

## Propósito

El sistema de reserva de saldo garantiza que una cuenta tenga fondos suficientes durante toda la duración de una llamada. Es el mecanismo central que previene llamadas sin fondos y permite el cobro correcto al finalizar.

## Flujo Completo de Reserva

```
┌─────────────────────────────────────────────────────────────┐
│  LLAMADA INICIA (CHANNEL_CREATE)                            │
├─────────────────────────────────────────────────────────────┤
│  Balance: S/10.00                                           │
│  Tarifa:  S/0.15/min                                        │
│  Reserva: S/0.81 (5 min + 8% buffer)                        │
│  Balance después: S/10.00 - S/0.81 = S/9.19                 │
│  max_duration: 324s                                         │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼ (180s después, si queda < 240s)
┌─────────────────────────────────────────────────────────────┐
│  EXTENSIÓN DE RESERVA                                       │
├─────────────────────────────────────────────────────────────┤
│  Extensión: +S/0.45 (3 min adicionales)                     │
│  Balance: S/9.19 - S/0.45 = S/8.74                          │
│  max_duration: 324s + 180s = 504s                           │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼ (llamada termina a los 45s)
┌─────────────────────────────────────────────────────────────┐
│  CONSUMO (CHANNEL_HANGUP)                                   │
├─────────────────────────────────────────────────────────────┤
│  Duración real: 45s                                         │
│  billsec_rounded: 48s (increment=6s)                        │
│  Costo real: (48/60) × S/0.15 = S/0.12                      │
│  Reservado: S/0.81                                          │
│  Excedente devuelto: S/0.81 - S/0.12 = S/0.69               │
│  Balance final: S/9.19 + S/0.69 = S/9.88                    │
└─────────────────────────────────────────────────────────────┘
```

## Fase 1: Creación de Reserva (CHANNEL_CREATE)

**Archivo:** `rust-billing-engine/src/services/authorization.rs`

Cuando inicia una llamada, el sistema ejecuta:

```
1. Busca la cuenta por número llamante (ANI)
   └─ Query: SELECT * FROM accounts WHERE account_number = $1
   └─ Si no existe → DENY (reason: "account_not_found")

2. Verifica estado de cuenta
   └─ Si status != 'active' → DENY (reason: "account_suspended")

3. Busca tarifa con LPM (Longest Prefix Match)
   └─ Si no hay tarifa → DENY (reason: "no_rate_found")

4. Calcula la reserva inicial:
   base = rate_per_minute × 5 minutos
   buffer = base × 8%
   total = clamp(base + buffer, S/0.30, S/30.00)
   max_duration = (total / rate_per_minute) × 60 segundos

5. Crea reserva de balance
   └─ ReservationManager.create_reservation()
   └─ Descuenta del balance disponible
   └─ Si balance insuficiente → DENY (reason: "insufficient_balance")
```

### Ejemplo de Cálculo

| Parámetro | Valor |
|-----------|-------|
| Tarifa | S/0.15/min |
| Base (5 min) | S/0.15 × 5 = S/0.75 |
| Buffer (8%) | S/0.75 × 0.08 = S/0.06 |
| **Total Reserva** | **S/0.81** |
| max_duration | (S/0.81 / S/0.15) × 60 = **324 seg** |

## Fase 2: Extensión de Reserva (Durante la llamada)

**Archivo:** `rust-billing-engine/src/services/realtime_biller.rs`

El `RealtimeBiller` monitorea cada **180 segundos**:

```rust
for each active_call:
    time_remaining = max_duration - elapsed

    if time_remaining < 240 segundos:
        extend_reservation(+3 minutos)
        // Descuenta más saldo del balance
        // Actualiza max_duration en Redis
```

Esto permite llamadas largas sin cortes prematuros por agotamiento de reserva.

## Fase 3: Consumo de Reserva (CHANNEL_HANGUP)

**Archivo:** `rust-billing-engine/src/services/reservation_manager.rs`

```
ReservationManager.consume_reservation():
│
├─ BEGIN TRANSACTION (con row locks)
│
├─ Query reservas activas para la llamada:
│   SELECT * FROM balance_reservations
│   WHERE call_uuid = $1 AND status = 'active'
│   FOR UPDATE
│
├─ Calcula costo real:
│   billsec_rounded = ceil(billsec / increment) × increment
│   cost = (billsec_rounded / 60) × rate_per_minute
│
├─ Compara actual_cost vs total_reserved:
│
│   CASO NORMAL (actual_cost <= reserved):
│   ├─ UPDATE balance_reservations SET consumed_amount = actual_cost
│   ├─ Devuelve excedente: balance += (reserved - actual_cost)
│   └─ INSERT INTO balance_transactions (log)
│
│   CASO DEFICIT (actual_cost > reserved):
│   ├─ Marcar todas las reservas como fully_consumed
│   ├─ Descontar costo COMPLETO del balance (puede ir negativo)
│   ├─ Si deficit > S/10.00 → AUTO-SUSPEND cuenta
│   └─ Log deficit transaction
│
└─ COMMIT + cleanup Redis
```

## Tablas de Base de Datos

### balance_reservations

```sql
CREATE TABLE balance_reservations (
    id SERIAL PRIMARY KEY,
    account_id INTEGER REFERENCES accounts(id),
    call_uuid VARCHAR(100) NOT NULL,
    reserved_amount DECIMAL(12,4) NOT NULL,
    consumed_amount DECIMAL(12,4) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'active',  -- active, partially_consumed, fully_consumed, expired
    type VARCHAR(20) NOT NULL,            -- initial, extension
    destination_prefix VARCHAR(20),
    rate_per_minute DECIMAL(10,6),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_reservations_call_uuid ON balance_reservations(call_uuid);
CREATE INDEX idx_reservations_account_status ON balance_reservations(account_id, status);
```

### balance_transactions

```sql
CREATE TABLE balance_transactions (
    id SERIAL PRIMARY KEY,
    account_id INTEGER REFERENCES accounts(id),
    amount DECIMAL(12,4) NOT NULL,
    type VARCHAR(20) NOT NULL,        -- reservation, consumption, refund, topup, deficit
    reference_id VARCHAR(100),        -- call_uuid o reservation_id
    balance_before DECIMAL(12,4),
    balance_after DECIMAL(12,4),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

## Estados de Reserva

| Estado | Descripción |
|--------|-------------|
| `active` | Reserva vigente durante llamada activa |
| `partially_consumed` | Llamada terminó, se usó parte de la reserva |
| `fully_consumed` | Llamada terminó, se usó toda la reserva |
| `expired` | Reserva expiró sin consumirse (cleanup job) |

## Control de Concurrencia

- **Máximo 5 llamadas simultáneas** por cuenta (configurable)
- Verificado en Redis: `SCARD active_reservations:{account_id}`
- Si se excede el límite → llamada denegada

## Manejo de Déficit

Cuando el costo real excede lo reservado:

```
SI deficit <= S/10.00:
    └─ Se permite, balance puede ir negativo temporalmente

SI deficit > S/10.00:
    ├─ Se cobra el costo completo
    ├─ Cuenta se SUSPENDE automáticamente
    └─ Log: "Auto-suspended due to excessive deficit"
```

## Cache Redis para Reservas

```
reservation:{id}           → Datos de reserva (TTL: 2700s / 45min)
active_reservations:{id}   → SET de reservation IDs activos por cuenta
call_session:{uuid}        → Metadata de sesión (max_duration, start_time, rate)
```

## Archivos Clave

| Archivo | Función |
|---------|---------|
| `services/reservation_manager.rs` | CRUD de reservas, consumo, extensión |
| `services/authorization.rs` | Crea reserva inicial al autorizar |
| `services/realtime_biller.rs` | Extiende reservas durante llamadas |
| `services/cdr_generator.rs` | Dispara consumo de reserva al generar CDR |

---

# CORRECCIONES Y MEJORAS RECIENTES (Enero 2026)

## Panel de Llamadas Activas - Tiempo Real

### Problema Resuelto
La duración de las llamadas activas no se actualizaba en tiempo real en la interfaz web.

### Solución Implementada
**Archivo:** `frontend/src/pages/ActiveCalls.tsx`

1. **Timer de actualización cada segundo:**
```typescript
const [, setTick] = useState(0)

useEffect(() => {
  const interval = setInterval(() => {
    setTick(t => t + 1)
  }, 1000)
  return () => clearInterval(interval)
}, [])
```

2. **Cálculo de duración en tiempo real:**
```typescript
// En la columna de duración:
const startTime = new Date(call.start_time).getTime()
const now = Date.now()
const durationSec = Math.max(0, Math.floor((now - startTime) / 1000))
```

3. **Cálculo de costo estimado en tiempo real:**
```typescript
const rate = call.rate_per_minute || 0
const cost = (durationSec / 60) * rate
```

### Resultado
- Duración se actualiza cada segundo
- Costo estimado se actualiza cada segundo
- Costo total de todas las llamadas se actualiza en tiempo real

---

## WebSocket - Manejo de Mensajes

### Problema Resuelto
El frontend no procesaba correctamente los mensajes `active_calls` del backend.

### Solución Implementada
**Archivo:** `frontend/src/hooks/useWebSocket.ts`

```typescript
case 'active_calls':
  const allCalls = message.data as ActiveCall[]
  setActiveCalls(allCalls)
  break
case 'pong':
  // Heartbeat response - no action needed
  break
```

**Archivo:** `frontend/src/types/index.ts`
```typescript
export interface WSMessage {
  type: 'active_calls' | 'call_start' | 'call_update' | 'call_end' | 'stats_update' | 'pong' | 'error'
  data: ActiveCall | ActiveCall[] | DashboardStats | { message: string }
}
```

---

## FreeSWITCH Dialplan - Autorización con Billing

### Problema Resuelto
Después de autorizar una llamada, FreeSWITCH no ejecutaba el bridge porque las condiciones del dialplan no continuaban evaluándose.

### Archivo
`/etc/freeswitch/dialplan/from-pbx.xml`

### Correcciones

1. **Agregar `break="never"` a las condiciones:**
```xml
<!-- Antes: la condición fallaba y detenía el procesamiento -->
<condition field="${billing_response}" expression="^DENIED">

<!-- Después: continúa evaluando las siguientes condiciones -->
<condition field="${billing_response}" expression="^DENIED" break="never">
```

2. **Escapar caracteres `&` en URLs (XML entities):**
```xml
<!-- Incorrecto (causa error de parseo XML): -->
<action application="set" data="billing_response=${curl(...?caller=${billing_caller}&callee=${billing_destination}&uuid=${uuid})}"/>

<!-- Correcto: -->
<action application="set" data="billing_response=${curl(...?caller=${billing_caller}&amp;callee=${billing_destination}&amp;uuid=${uuid})}"/>
```

### Importante
- **Ownership del archivo:** El archivo debe pertenecer al usuario `freeswitch:freeswitch`
- **Validar XML:** Usar `xmllint --noout /etc/freeswitch/dialplan/from-pbx.xml`
- **Recargar:** Ejecutar `fs_cli -x "reloadxml"` después de cambios

---

## CDR Generator - Nombres de Columnas

### Problema Resuelto
Error de base de datos al insertar CDRs porque los nombres de columnas no coincidían con el esquema.

### Archivo
`rust-billing-engine/src/services/cdr_generator.rs`

### Corrección
```rust
// Columnas incorrectas → correctas:
// uuid           → call_uuid
// caller         → caller_number
// callee         → called_number
// rate_applied   → rate_per_minute

"INSERT INTO cdrs
 (call_uuid, account_id, caller_number, called_number, start_time, answer_time, end_time,
  duration, billsec, hangup_cause, rate_per_minute, cost, direction, freeswitch_server_id)
 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
 RETURNING id"
```

---

## Timestamps PostgreSQL - DateTime<Utc>

### Problema Resuelto
Errores de serialización al insertar/consultar campos `TIMESTAMP WITH TIME ZONE`.

### Archivos Afectados
- `rust-billing-engine/src/services/authorization.rs`
- `rust-billing-engine/src/services/cdr_generator.rs`
- `rust-billing-engine/src/services/reservation_manager.rs`

### Corrección
```rust
// Incorrecto: usar NaiveDateTime para TIMESTAMP WITH TIME ZONE
let created_at_naive: NaiveDateTime = row.get(6);

// Correcto: usar DateTime<Utc> directamente
let created_at: DateTime<Utc> = row.get(6);
```

**Regla:** PostgreSQL `TIMESTAMP WITH TIME ZONE` requiere `DateTime<Utc>` con `tokio-postgres`, no `NaiveDateTime`.

---

## Gestión de Zonas - CRUD Completo

### Mejoras Implementadas
**Archivo:** `frontend/src/pages/Zones.tsx`

1. **Operaciones CRUD completas:**
   - Crear zona
   - Editar zona
   - Eliminar zona con confirmación

2. **Campos adicionales:**
   - `zone_code` - Código de zona (ej: PE-LIM, US-NYC)
   - `region_name` - Nombre de región (ej: Sudamérica)

3. **Manejo de errores:**
   - Mensajes de error visibles en modales
   - Callbacks `onError` en mutations

4. **Columna de acciones:**
```typescript
{
  key: 'actions',
  header: 'Acciones',
  render: (zone: Zone) => (
    <div className="flex items-center space-x-2">
      <button onClick={() => setEditingZone(zone)}>
        <Pencil className="w-4 h-4" />
      </button>
      <button onClick={() => setDeletingZone(zone)}>
        <Trash2 className="w-4 h-4" />
      </button>
    </div>
  ),
}
```

### Type Definition
**Archivo:** `frontend/src/types/index.ts`
```typescript
export interface Zone {
  id: number
  zone_name: string
  zone_code?: string      // Nuevo
  zone_type?: string
  network_type?: string
  region_name?: string    // Nuevo
  description?: string
  enabled?: boolean
  created_at?: string
  updated_at?: string
}
```

---

## Gestión de Tarifas - Relación con Zonas

### Mejora Implementada
**Archivo:** `frontend/src/pages/Rates.tsx`

### Cambio
En lugar de escribir manualmente el nombre del destino, ahora se selecciona una zona desde un dropdown.

```typescript
// Cargar zonas disponibles
const { data: zones = [] } = useQuery({
  queryKey: ['zones'],
  queryFn: fetchZones,
})

// Selector de zona en el formulario
<select
  value={formData.zone_id || ''}
  onChange={(e) => handleZoneChange(e.target.value)}
>
  <option value="">Seleccionar zona...</option>
  {zones.map((zone) => (
    <option key={zone.id} value={zone.id}>
      {zone.zone_name} {zone.description ? `- ${zone.description}` : ''}
    </option>
  ))}
</select>

// Al seleccionar zona, se establece destination_name automáticamente
const handleZoneChange = (zoneId: string) => {
  const zone = zones.find(z => z.id === Number(zoneId))
  setFormData({
    ...formData,
    zone_id: zone?.id,
    destination_name: zone?.zone_name || ''
  })
}
```

---

## Active Calls - Tabla en Base de Datos

### Mejora Implementada
**Archivo:** `rust-billing-engine/src/esl/event_handler.rs`

El billing engine ahora inserta y elimina registros de la tabla `active_calls`:

```rust
// En CHANNEL_CREATE (después de autorización exitosa):
INSERT INTO active_calls (call_uuid, caller_number, callee_number, ...)
VALUES ($1, $2, $3, ...)

// En CHANNEL_HANGUP:
DELETE FROM active_calls WHERE call_uuid = $1
```

---

## Página de Consulta de CDRs - Correcciones

### Problema 1: Formato de Respuesta de Paginación

**Archivo:** `frontend/src/api/client.ts`

La API devuelve paginación anidada, pero el frontend esperaba campos planos:

```typescript
// API devuelve:
{ data: [...], pagination: { total, page, per_page, total_pages } }

// Frontend esperaba:
{ data: [...], total, page, per_page, total_pages }
```

**Solución:** Transformar la respuesta en `fetchCDRs`:
```typescript
export const fetchCDRs = async (...): Promise<PaginatedResponse<CDR>> => {
  const { data } = await api.get(`/cdrs?${params.toString()}`)

  // Transform API response format
  if (data.pagination) {
    return {
      data: data.data,
      total: data.pagination.total,
      page: data.pagination.page,
      per_page: data.pagination.per_page,
      total_pages: data.pagination.total_pages,
    }
  }
  return data
}
```

### Problema 2: Campo `cost` como String

**Archivo:** `frontend/src/pages/CDR.tsx`

La API devuelve `cost` como string (`"0.9000"`) en lugar de número, causando errores al hacer operaciones matemáticas.

**Solución:** Parsear el costo a número:
```typescript
// Antes (fallaba):
${(cdr.total_cost ?? cdr.cost ?? 0).toFixed(4)}

// Después (correcto):
const cost = parseFloat(String(cdr.total_cost ?? cdr.cost ?? 0)) || 0
${cost.toFixed(4)}
```

### Problema 3: Campos `duration` y `billsec` Nulos

**Archivo:** `frontend/src/pages/CDR.tsx`

Los campos `duration` y `billsec` podían ser `null`, causando error en `formatDuration()`.

**Solución:** Valor por defecto:
```typescript
// Antes:
formatDuration(cdr.duration)

// Después:
formatDuration(cdr.duration ?? 0)
formatDuration(cdr.billsec ?? 0)
```

### Cálculo del Total Facturado

También corregido para parsear strings:
```typescript
{(data.data ?? []).reduce((sum, cdr) =>
  sum + (parseFloat(String(cdr.total_cost ?? cdr.cost ?? 0)) || 0), 0
).toFixed(2)}
```

---

## Resumen de Archivos Modificados

| Archivo | Cambios |
|---------|---------|
| `frontend/src/pages/ActiveCalls.tsx` | Timer 1s, cálculo real-time duración/costo |
| `frontend/src/pages/CDR.tsx` | Parseo de cost string→number, null checks |
| `frontend/src/pages/Zones.tsx` | CRUD completo, modales edit/delete |
| `frontend/src/pages/Rates.tsx` | Selector dropdown de zonas |
| `frontend/src/api/client.ts` | Transformación respuesta paginación CDRs |
| `frontend/src/hooks/useWebSocket.ts` | Manejo mensajes `active_calls`, `pong` |
| `frontend/src/types/index.ts` | Zone: +zone_code, +region_name |
| `rust-billing-engine/src/services/cdr_generator.rs` | Nombres columnas CDR |
| `rust-billing-engine/src/services/authorization.rs` | DateTime<Utc> timestamps |
| `rust-billing-engine/src/services/reservation_manager.rs` | DateTime<Utc> expires_at |
| `rust-billing-engine/src/esl/event_handler.rs` | INSERT/DELETE active_calls |
| `/etc/freeswitch/dialplan/from-pbx.xml` | break="never", &amp; entities |

---

# GIT Y GITHUB

## Verificar Estado de Cambios

```bash
# Ver estado actual
git status

# Ver resumen de cambios
git diff --stat

# Ver commits locales pendientes de push
git log origin/main..HEAD --oneline
```

## Proceso para Subir Cambios a GitHub

### Paso 1: Agregar archivos al staging

```bash
# Opción A: Agregar todos los cambios
git add .

# Opción B: Agregar selectivamente por directorio
git add frontend/
git add rust-backend/
git add .claude/
git add CLAUDE.md

# Opción C: Agregar archivos específicos
git add archivo1.ts archivo2.rs
```

### Paso 2: Crear commit

```bash
# Commit simple
git commit -m "descripción del cambio"

# Commit con mensaje multilínea (recomendado)
git commit -m "$(cat <<'EOF'
feat: Descripción corta del cambio

- Detalle 1 de los cambios realizados
- Detalle 2 de los cambios realizados
- Detalle 3 de los cambios realizados

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

### Paso 3: Subir a GitHub

```bash
# Push a la rama main
git push origin main

# Si es la primera vez o hay problemas
git push -u origin main
```

## Convenciones de Commits

| Prefijo | Uso |
|---------|-----|
| `feat:` | Nueva funcionalidad |
| `fix:` | Corrección de bug |
| `docs:` | Cambios en documentación |
| `refactor:` | Refactorización sin cambio funcional |
| `test:` | Agregar o modificar tests |
| `chore:` | Tareas de mantenimiento |

## Ejemplo Completo

```bash
# 1. Verificar cambios
git status

# 2. Agregar todo
git add .

# 3. Crear commit descriptivo
git commit -m "$(cat <<'EOF'
feat: Migración completa Python → Rust backend

- Migrar endpoints de FastAPI a Actix-web
- Agregar frontend React + TypeScript + Vite
- Configurar autenticación JWT con Argon2
- Agregar skills de Claude Code
- Actualizar documentación

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"

# 4. Subir a GitHub
git push origin main
```

## Resolver Conflictos

```bash
# Si hay cambios remotos, primero hacer pull
git pull origin main

# Si hay conflictos, resolverlos y luego:
git add .
git commit -m "fix: Resolver conflictos de merge"
git push origin main
```

## Ver Historial

```bash
# Ver últimos commits
git log --oneline -10

# Ver commits con cambios
git log --stat -5

# Ver diferencias con remote
git diff origin/main
```

---

# SKILLS DE CLAUDE CODE

## Skill: frontend-design

### Descripción
Skill para crear interfaces frontend distintivas y de alta calidad, evitando estéticas genéricas de IA. Genera código production-ready con atención excepcional a detalles estéticos.

### Ubicación
```
.claude/skills/frontend-design/SKILL.md
```

### Instalación

1. Crear directorio de skills:
```bash
mkdir -p /opt/ApoloBilling/.claude/skills/frontend-design
```

2. Crear archivo SKILL.md con el contenido del skill:
```bash
# Descargar desde el repositorio oficial de Claude Code
# https://github.com/anthropics/claude-code/blob/main/plugins/frontend-design/skills/frontend-design/SKILL.md
```

3. El skill se auto-descubre automáticamente por Claude Code

### Uso

**Invocación directa:**
```
/frontend-design
```

**Uso implícito:** Claude aplicará el skill automáticamente cuando detecte tareas de diseño frontend.

### Qué hace el skill

| Aspecto | Descripción |
|---------|-------------|
| **Propósito** | Crear interfaces frontend distintivas y memorables |
| **Evita** | Fuentes genéricas (Inter, Roboto, Arial), gradientes púrpura, layouts predecibles |
| **Enfoque** | Tipografía única, paletas cohesivas, animaciones con propósito, composición espacial creativa |

### Direcciones estéticas soportadas
- Minimalismo brutal
- Caos maximalista
- Retro-futurista
- Orgánico/natural
- Lujo/refinado
- Juguetón/toy-like
- Editorial/magazine
- Brutalista/raw
- Art deco/geométrico
- Soft/pastel
- Industrial/utilitario

### Áreas de enfoque

**Tipografía:**
- Fuentes distintivas y con carácter
- Parejas de fuentes display + body

**Color y tema:**
- Paletas cohesivas con CSS variables
- Colores dominantes con acentos marcados

**Movimiento:**
- Animaciones CSS para micro-interacciones
- Motion library para React
- Reveals escalonados con animation-delay

**Composición espacial:**
- Layouts inesperados
- Asimetría y superposición
- Flujo diagonal y elementos que rompen la grilla

**Fondos y detalles:**
- Gradient meshes, texturas de ruido
- Patrones geométricos, transparencias
- Sombras dramáticas, bordes decorativos

### Ejemplo de uso

```
Usuario: Crea un dashboard moderno para ApoloBilling

Claude: (Aplica skill frontend-design)
1. Analiza el contexto (billing, telecomunicaciones, profesional)
2. Elige dirección estética (ej: industrial/utilitario con toques de lujo)
3. Selecciona tipografía distintiva (ej: JetBrains Mono + Outfit)
4. Define paleta cohesiva con CSS variables
5. Implementa código React/TypeScript production-ready
6. Agrega animaciones y micro-interacciones
```

---

# DEPLOYMENT COMPLETO EN DEBIAN 12

Esta guía detalla el proceso completo para desplegar ApoloBilling desde cero en un servidor Debian 12, incluyendo la instalación de todas las dependencias, configuración de servicios y puesta en producción.

## 1. Requisitos del Sistema

### Hardware Mínimo

| Componente | Mínimo | Recomendado |
|------------|--------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 2 GB | 4+ GB |
| Disco | 20 GB SSD | 50+ GB SSD |
| Red | 100 Mbps | 1 Gbps |

### Software Base

| Componente | Versión |
|------------|---------|
| Sistema Operativo | Debian 12 (Bookworm) |
| PostgreSQL | 15+ |
| Redis | 7+ |
| Rust | 1.75+ (stable) |
| Node.js | 20 LTS |
| Nginx | 1.22+ |

### Puertos Requeridos

| Puerto | Servicio | Descripción |
|--------|----------|-------------|
| 22 | SSH | Acceso remoto |
| 80 | HTTP | Redirección a HTTPS |
| 443 | HTTPS | Frontend y API |
| 3000 | Frontend Dev | Solo desarrollo |
| 5432 | PostgreSQL | Base de datos (local) |
| 6379 | Redis | Caché (local) |
| 8000 | Rust Backend | API REST (interno) |
| 8021 | FreeSWITCH ESL | Event Socket (interno) |
| 9000 | Billing Engine | Tarificación (interno) |

---

## 2. Preparación del Servidor

### 2.1 Actualizar Sistema

```bash
# Actualizar repositorios y paquetes
sudo apt update && sudo apt upgrade -y

# Instalar herramientas básicas
sudo apt install -y \
    curl \
    wget \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    gnupg \
    lsb-release \
    software-properties-common \
    htop \
    vim \
    unzip
```

### 2.2 Crear Usuario del Sistema

```bash
# Crear usuario apolo para ejecutar los servicios
sudo useradd -r -m -s /bin/bash apolo

# Crear directorio de instalación
sudo mkdir -p /opt/ApoloBilling
sudo chown apolo:apolo /opt/ApoloBilling
```

---

## 3. Instalación de Dependencias

### 3.1 PostgreSQL 15

```bash
# Instalar PostgreSQL
sudo apt install -y postgresql postgresql-contrib

# Iniciar y habilitar servicio
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Verificar instalación
psql --version
```

### 3.2 Redis 7

```bash
# Instalar Redis
sudo apt install -y redis-server

# Configurar Redis para systemd
sudo sed -i 's/supervised no/supervised systemd/' /etc/redis/redis.conf

# Reiniciar Redis
sudo systemctl restart redis-server
sudo systemctl enable redis-server

# Verificar instalación
redis-cli ping
# Respuesta esperada: PONG
```

### 3.3 Rust (via rustup)

```bash
# Instalar Rust como usuario apolo
sudo -u apolo bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'

# Cargar variables de entorno
sudo -u apolo bash -c 'source $HOME/.cargo/env && rustc --version'

# Verificar instalación
sudo -u apolo bash -c 'source $HOME/.cargo/env && cargo --version'
```

### 3.4 Node.js 20 LTS

```bash
# Agregar repositorio NodeSource
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -

# Instalar Node.js
sudo apt install -y nodejs

# Verificar instalación
node --version
npm --version
```

### 3.5 Nginx

```bash
# Instalar Nginx
sudo apt install -y nginx

# Iniciar y habilitar servicio
sudo systemctl start nginx
sudo systemctl enable nginx

# Verificar instalación
nginx -v
```

---

## 4. Configuración de PostgreSQL

### 4.1 Crear Usuario y Base de Datos

```bash
# Conectar como usuario postgres
sudo -u postgres psql

# Ejecutar en consola psql:
```

```sql
-- Crear usuario
CREATE USER apolo_user WITH PASSWORD 'TU_PASSWORD_SEGURO_AQUI';

-- Crear base de datos
CREATE DATABASE apolo_billing OWNER apolo_user;

-- Otorgar permisos
GRANT ALL PRIVILEGES ON DATABASE apolo_billing TO apolo_user;

-- Salir
\q
```

### 4.2 Configurar Acceso Local

```bash
# Editar pg_hba.conf para permitir acceso local con password
sudo vim /etc/postgresql/15/main/pg_hba.conf

# Agregar o modificar la línea:
# local   apolo_billing   apolo_user                      md5

# Reiniciar PostgreSQL
sudo systemctl restart postgresql
```

### 4.3 Crear Schema Completo

```bash
# Conectar a la base de datos
psql -U apolo_user -d apolo_billing -h localhost
```

```sql
-- ============================================
-- SCHEMA COMPLETO DE APOLOBILLING
-- ============================================

-- Tabla: accounts (Cuentas de clientes)
CREATE TABLE IF NOT EXISTS accounts (
    id SERIAL PRIMARY KEY,
    account_number VARCHAR(50) UNIQUE NOT NULL,
    account_name VARCHAR(200) NOT NULL,
    account_type VARCHAR(20) NOT NULL DEFAULT 'PREPAID',
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    balance DECIMAL(12, 4) NOT NULL DEFAULT 0.0000,
    currency VARCHAR(3) NOT NULL DEFAULT 'PEN',
    credit_limit DECIMAL(12, 4) DEFAULT 0.0000,
    max_concurrent_calls INTEGER DEFAULT 5,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by VARCHAR(100) DEFAULT 'system',
    updated_by VARCHAR(100) DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_accounts_number ON accounts(account_number);
CREATE INDEX IF NOT EXISTS idx_accounts_status ON accounts(status);
CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(account_type);

-- Tabla: rate_cards (Tarifas por destino)
CREATE TABLE IF NOT EXISTS rate_cards (
    id SERIAL PRIMARY KEY,
    rate_name VARCHAR(200) NOT NULL,
    destination_prefix VARCHAR(20) NOT NULL,
    destination_name VARCHAR(200),
    rate_per_minute DECIMAL(10, 6) NOT NULL,
    billing_increment INTEGER NOT NULL DEFAULT 6,
    initial_increment_seconds INTEGER NOT NULL DEFAULT 6,
    connection_fee DECIMAL(10, 6) DEFAULT 0.0,
    priority INTEGER NOT NULL DEFAULT 100,
    effective_start TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    effective_end TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by VARCHAR(100) DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_rate_cards_prefix ON rate_cards(destination_prefix);
CREATE INDEX IF NOT EXISTS idx_rate_cards_priority ON rate_cards(priority DESC);
CREATE INDEX IF NOT EXISTS idx_rate_cards_dates ON rate_cards(effective_start, effective_end);
CREATE INDEX IF NOT EXISTS idx_rate_cards_prefix_priority ON rate_cards(destination_prefix, priority DESC);

-- Tabla: balance_reservations (Reservas de balance)
CREATE TABLE IF NOT EXISTS balance_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    call_uuid VARCHAR(100) NOT NULL,
    reserved_amount DECIMAL(12, 4) NOT NULL DEFAULT 0.0000,
    consumed_amount DECIMAL(12, 4) NOT NULL DEFAULT 0.0000,
    released_amount DECIMAL(12, 4) NOT NULL DEFAULT 0.0000,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    type VARCHAR(20) NOT NULL DEFAULT 'initial',
    destination_prefix VARCHAR(20),
    rate_per_minute DECIMAL(10, 6),
    reserved_minutes INTEGER,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    consumed_at TIMESTAMP WITH TIME ZONE,
    released_at TIMESTAMP WITH TIME ZONE,
    created_by VARCHAR(100) DEFAULT 'system',
    updated_by VARCHAR(100) DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_reservations_account ON balance_reservations(account_id);
CREATE INDEX IF NOT EXISTS idx_reservations_call ON balance_reservations(call_uuid);
CREATE INDEX IF NOT EXISTS idx_reservations_status ON balance_reservations(status);
CREATE INDEX IF NOT EXISTS idx_reservations_expires ON balance_reservations(expires_at);
CREATE INDEX IF NOT EXISTS idx_reservations_account_status ON balance_reservations(account_id, status);

-- Tabla: balance_transactions (Log de transacciones)
CREATE TABLE IF NOT EXISTS balance_transactions (
    id BIGSERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    amount DECIMAL(12, 4) NOT NULL,
    previous_balance DECIMAL(12, 4) NOT NULL,
    new_balance DECIMAL(12, 4) NOT NULL,
    type VARCHAR(20) NOT NULL,
    reason TEXT,
    call_uuid VARCHAR(100),
    reservation_id UUID REFERENCES balance_reservations(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by VARCHAR(100) DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_transactions_account ON balance_transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_call ON balance_transactions(call_uuid);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON balance_transactions(type);
CREATE INDEX IF NOT EXISTS idx_transactions_created ON balance_transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_reservation ON balance_transactions(reservation_id);

-- Tabla: cdrs (Call Detail Records)
CREATE TABLE IF NOT EXISTS cdrs (
    id BIGSERIAL PRIMARY KEY,
    call_uuid VARCHAR(100) UNIQUE NOT NULL,
    account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    caller_number VARCHAR(50) NOT NULL,
    called_number VARCHAR(50) NOT NULL,
    destination_prefix VARCHAR(20),
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    answer_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    duration INTEGER NOT NULL DEFAULT 0,
    billsec INTEGER NOT NULL DEFAULT 0,
    rate_per_minute DECIMAL(10, 6),
    cost DECIMAL(12, 4) DEFAULT 0.0000,
    hangup_cause VARCHAR(50),
    direction VARCHAR(20),
    freeswitch_server_id VARCHAR(100),
    reservation_id UUID REFERENCES balance_reservations(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_cdr_uuid ON cdrs(call_uuid);
CREATE INDEX IF NOT EXISTS idx_cdr_account ON cdrs(account_id);
CREATE INDEX IF NOT EXISTS idx_cdr_caller ON cdrs(caller_number);
CREATE INDEX IF NOT EXISTS idx_cdr_callee ON cdrs(called_number);
CREATE INDEX IF NOT EXISTS idx_cdr_start_time ON cdrs(start_time);
CREATE INDEX IF NOT EXISTS idx_cdr_account_start ON cdrs(account_id, start_time);
CREATE INDEX IF NOT EXISTS idx_cdr_reservation ON cdrs(reservation_id);

-- Tabla: active_calls (Llamadas activas)
CREATE TABLE IF NOT EXISTS active_calls (
    id SERIAL PRIMARY KEY,
    call_id VARCHAR(100) UNIQUE NOT NULL,
    calling_number VARCHAR(50),
    called_number VARCHAR(50),
    direction VARCHAR(20),
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    answer_time TIMESTAMP WITH TIME ZONE,
    current_duration INTEGER DEFAULT 0,
    current_cost DECIMAL(12, 4) DEFAULT 0.0000,
    rate_per_minute DECIMAL(10, 6),
    connection_id VARCHAR(100),
    server VARCHAR(100),
    client_id INTEGER,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    status VARCHAR(20) DEFAULT 'active'
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_active_calls_call_id ON active_calls(call_id);
CREATE INDEX IF NOT EXISTS idx_active_calls_client ON active_calls(client_id);
CREATE INDEX IF NOT EXISTS idx_active_calls_start ON active_calls(start_time);

-- Tabla: usuarios (Usuarios del sistema)
CREATE TABLE IF NOT EXISTS usuarios (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    nombre VARCHAR(100),
    apellido VARCHAR(100),
    email VARCHAR(255),
    role VARCHAR(20) NOT NULL DEFAULT 'operator',
    activo BOOLEAN NOT NULL DEFAULT true,
    ultimo_login TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_usuarios_username ON usuarios(username);
CREATE INDEX IF NOT EXISTS idx_usuarios_email ON usuarios(email);

-- Tabla: zonas (Zonas de destino)
CREATE TABLE IF NOT EXISTS zonas (
    id SERIAL PRIMARY KEY,
    zone_name VARCHAR(200) NOT NULL,
    zone_code VARCHAR(50),
    zone_type VARCHAR(50),
    network_type VARCHAR(50),
    region_name VARCHAR(200),
    description TEXT,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Tabla: prefijos (Prefijos de destino)
CREATE TABLE IF NOT EXISTS prefijos (
    id SERIAL PRIMARY KEY,
    prefix VARCHAR(20) NOT NULL,
    zone_id INTEGER REFERENCES zonas(id) ON DELETE SET NULL,
    description TEXT,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_prefijos_prefix ON prefijos(prefix);
CREATE INDEX IF NOT EXISTS idx_prefijos_zone ON prefijos(zone_id);

-- Tabla: tarifas (Tarifas por zona)
CREATE TABLE IF NOT EXISTS tarifas (
    id SERIAL PRIMARY KEY,
    zone_id INTEGER REFERENCES zonas(id) ON DELETE CASCADE,
    rate_per_minute DECIMAL(10, 6) NOT NULL,
    billing_increment INTEGER DEFAULT 6,
    connection_fee DECIMAL(10, 6) DEFAULT 0.0,
    effective_start TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    effective_end TIMESTAMP WITH TIME ZONE,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tarifas_zone ON tarifas(zone_id);
CREATE INDEX IF NOT EXISTS idx_tarifas_dates ON tarifas(effective_start, effective_end);

-- ============================================
-- VISTAS ÚTILES
-- ============================================

-- Vista: Balance disponible por cuenta
CREATE OR REPLACE VIEW v_available_balance AS
SELECT
    a.id,
    a.account_number,
    a.account_name,
    a.balance,
    COALESCE(SUM(br.reserved_amount - br.consumed_amount), 0) as total_reserved,
    a.balance - COALESCE(SUM(br.reserved_amount - br.consumed_amount), 0) as available_balance
FROM accounts a
LEFT JOIN balance_reservations br ON a.id = br.account_id AND br.status = 'active'
GROUP BY a.id, a.account_number, a.account_name, a.balance;

-- Vista: Resumen de CDRs por cuenta
CREATE OR REPLACE VIEW v_cdr_summary AS
SELECT
    account_id,
    DATE(start_time) as call_date,
    COUNT(*) as total_calls,
    SUM(duration) as total_duration_seconds,
    SUM(billsec) as total_billsec_seconds,
    SUM(cost) as total_cost
FROM cdrs
GROUP BY account_id, DATE(start_time);

-- ============================================
-- PERMISOS FINALES
-- ============================================

GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO apolo_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO apolo_user;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO apolo_user;
```

---

## 5. Clonar Repositorio desde GitHub

```bash
# Cambiar a usuario apolo
sudo -u apolo bash

# Ir al directorio de instalación
cd /opt/ApoloBilling

# Clonar repositorio
git clone https://github.com/jesus-bazan-entel/ApoloBilling.git .

# Verificar contenido
ls -la
```

---

## 6. Compilar Rust Backend

```bash
# Como usuario apolo
sudo -u apolo bash
source ~/.cargo/env
cd /opt/ApoloBilling/rust-backend

# Crear archivo .env
cat > .env << 'EOF'
# Database
DATABASE_URL=postgresql://apolo_user:TU_PASSWORD_SEGURO_AQUI@localhost:5432/apolo_billing
DATABASE_MAX_CONNECTIONS=20

# Server
RUST_SERVER_HOST=0.0.0.0
RUST_SERVER_PORT=8000
RUST_SERVER_WORKERS=4

# CORS (ajustar a tu dominio en producción)
CORS_ORIGINS=http://localhost:3000,http://127.0.0.1:3000,https://tu-dominio.com

# JWT Authentication (CAMBIAR EN PRODUCCIÓN)
JWT_SECRET=cambiar-esta-clave-secreta-en-produccion-con-algo-aleatorio-y-largo
JWT_EXPIRATION_SECS=1800

# Redis
REDIS_URL=redis://127.0.0.1:6379

# Logging
LOG_LEVEL=info
RUST_LOG=apolo_billing=info,apolo_api=info,actix_web=info
EOF

# Compilar en modo release (puede tomar varios minutos)
cargo build --release

# Verificar binario
ls -la target/release/apolo-billing
```

---

## 7. Compilar Rust Billing Engine

```bash
# Como usuario apolo
sudo -u apolo bash
source ~/.cargo/env
cd /opt/ApoloBilling/rust-billing-engine

# Crear archivo .env
cat > .env << 'EOF'
# Environment
ENVIRONMENT=production
HOST=127.0.0.1
PORT=9000

# Database
DATABASE_URL=postgresql://apolo_user:TU_PASSWORD_SEGURO_AQUI@localhost:5432/apolo_billing

# Redis
REDIS_URL=redis://127.0.0.1:6379

# FreeSWITCH (ajustar según tu configuración)
# Formato: host:port:password (separados por coma para múltiples servidores)
FREESWITCH_SERVERS=127.0.0.1:8021:ClueCon

# Logging
RUST_LOG=info,apolo_billing_engine=info
EOF

# Compilar en modo release
cargo build --release

# Verificar binario
ls -la target/release/apolo-billing-engine
```

---

## 8. Instalar Frontend

```bash
# Como usuario apolo
sudo -u apolo bash
cd /opt/ApoloBilling/frontend

# Instalar dependencias
npm install

# Construir para producción
npm run build

# Verificar build
ls -la dist/
```

---

## 9. Crear Usuario Admin

```bash
# Generar hash Argon2 para la contraseña
cd /opt/ApoloBilling/rust-backend
source ~/.cargo/env

# Ejecutar el ejemplo para generar hash
# (Si no existe, crear uno simple o usar herramienta online de Argon2)
cargo run --example gen_hash -p apolo-auth 2>/dev/null || echo "Usar hash pre-generado"

# Hash pre-generado para 'admin123':
# $argon2id$v=19$m=19456,t=2,p=1$YXBvbG9iaWxsaW5n$8qVJ3p1nYzT9v3pK2xL4mHqWcD7FgN9kR6sT0uI2yXw
```

```bash
# Insertar usuario admin en la base de datos
psql -U apolo_user -d apolo_billing -h localhost << 'EOF'
INSERT INTO usuarios (username, password, nombre, apellido, email, role, activo)
VALUES (
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$YXBvbG9iaWxsaW5n$8qVJ3p1nYzT9v3pK2xL4mHqWcD7FgN9kR6sT0uI2yXw',
    'Administrador',
    'Sistema',
    'admin@apolobilling.local',
    'admin',
    true
) ON CONFLICT (username) DO NOTHING;
EOF
```

---

## 10. Configurar Servicios Systemd

### 10.1 Servicio: Rust Backend API

```bash
sudo tee /etc/systemd/system/apolo-backend.service << 'EOF'
[Unit]
Description=Apolo Backend - REST API for billing management
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=apolo
Group=apolo
WorkingDirectory=/opt/ApoloBilling/rust-backend
Environment=RUST_LOG=info,apolo_api=info,apolo_billing=info
EnvironmentFile=/opt/ApoloBilling/rust-backend/.env
ExecStart=/opt/ApoloBilling/rust-backend/target/release/apolo-billing
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/ApoloBilling/rust-backend
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF
```

### 10.2 Servicio: Rust Billing Engine

```bash
sudo tee /etc/systemd/system/apolo-billing-engine.service << 'EOF'
[Unit]
Description=Apolo Billing Engine - Real-time call authorization and billing
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=apolo
Group=apolo
WorkingDirectory=/opt/ApoloBilling/rust-billing-engine
Environment=RUST_LOG=info,apolo_billing_engine=info
EnvironmentFile=/opt/ApoloBilling/rust-billing-engine/.env
ExecStart=/opt/ApoloBilling/rust-billing-engine/target/release/apolo-billing-engine
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/ApoloBilling/rust-billing-engine
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF
```

### 10.3 Servicio: Frontend (Producción con serve)

```bash
# Instalar serve globalmente
sudo npm install -g serve

# Crear servicio
sudo tee /etc/systemd/system/apolo-frontend.service << 'EOF'
[Unit]
Description=Apolo Frontend - React SPA
After=network.target

[Service]
Type=simple
User=apolo
Group=apolo
WorkingDirectory=/opt/ApoloBilling/frontend
ExecStart=/usr/bin/serve -s dist -l 3000
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
```

### 10.4 Habilitar e Iniciar Servicios

```bash
# Recargar systemd
sudo systemctl daemon-reload

# Habilitar servicios para inicio automático
sudo systemctl enable apolo-backend
sudo systemctl enable apolo-billing-engine
sudo systemctl enable apolo-frontend

# Iniciar servicios
sudo systemctl start apolo-backend
sudo systemctl start apolo-billing-engine
sudo systemctl start apolo-frontend

# Verificar estado
sudo systemctl status apolo-backend
sudo systemctl status apolo-billing-engine
sudo systemctl status apolo-frontend
```

---

## 11. Configurar Nginx como Reverse Proxy

### 11.1 Crear Configuración del Sitio

```bash
sudo tee /etc/nginx/sites-available/apolobilling << 'EOF'
# Upstream servers
upstream rust_backend {
    server 127.0.0.1:8000;
    keepalive 32;
}

upstream frontend {
    server 127.0.0.1:3000;
}

# HTTP - Redirect to HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name tu-dominio.com;

    # Para Let's Encrypt
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTPS - Main server
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name tu-dominio.com;

    # SSL certificates (ajustar rutas después de obtener certificados)
    ssl_certificate /etc/letsencrypt/live/tu-dominio.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/tu-dominio.com/privkey.pem;

    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_proxied any;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/json application/xml;

    # Logging
    access_log /var/log/nginx/apolobilling.access.log;
    error_log /var/log/nginx/apolobilling.error.log;

    # Frontend SPA
    location / {
        proxy_pass http://frontend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_redirect off;
    }

    # Backend API
    location /api/ {
        proxy_pass http://rust_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 300s;
        proxy_connect_timeout 75s;
        proxy_send_timeout 300s;

        # Buffer settings
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://rust_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }

    # Health check endpoint
    location /health {
        proxy_pass http://rust_backend/api/v1/health;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
    }
}
EOF
```

### 11.2 Habilitar Sitio

```bash
# Crear enlace simbólico
sudo ln -sf /etc/nginx/sites-available/apolobilling /etc/nginx/sites-enabled/

# Eliminar configuración por defecto (opcional)
sudo rm -f /etc/nginx/sites-enabled/default

# Verificar configuración
sudo nginx -t

# Recargar Nginx
sudo systemctl reload nginx
```

---

## 12. Obtener Certificado SSL con Let's Encrypt

```bash
# Instalar Certbot
sudo apt install -y certbot python3-certbot-nginx

# Crear directorio para challenge
sudo mkdir -p /var/www/certbot

# Obtener certificado (reemplazar con tu dominio)
sudo certbot --nginx -d tu-dominio.com

# Configurar renovación automática
sudo systemctl enable certbot.timer
sudo systemctl start certbot.timer

# Verificar renovación
sudo certbot renew --dry-run
```

---

## 13. Configurar Firewall (UFW)

```bash
# Instalar UFW
sudo apt install -y ufw

# Configurar reglas básicas
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Permitir SSH
sudo ufw allow ssh

# Permitir HTTP y HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Habilitar firewall
sudo ufw enable

# Verificar estado
sudo ufw status verbose
```

---

## 14. Verificación del Deployment

### 14.1 Verificar Servicios

```bash
# Estado de todos los servicios
sudo systemctl status apolo-backend apolo-billing-engine apolo-frontend

# Logs en tiempo real
sudo journalctl -u apolo-backend -f
sudo journalctl -u apolo-billing-engine -f
```

### 14.2 Verificar Endpoints

```bash
# Health check del backend
curl -s http://localhost:8000/api/v1/health | jq .

# Probar login
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' \
  -c /tmp/cookies.txt \
  -v

# Probar endpoint protegido
curl http://localhost:8000/api/v1/accounts -b /tmp/cookies.txt | jq .
```

### 14.3 Verificar Frontend

```bash
# Verificar que el frontend responde
curl -s http://localhost:3000 | head -20
```

---

## 15. Comandos Útiles de Administración

### Logs

```bash
# Ver logs de todos los servicios
sudo journalctl -u apolo-backend -u apolo-billing-engine -u apolo-frontend -f

# Ver logs de un servicio específico
sudo journalctl -u apolo-backend -n 100 --no-pager

# Ver logs desde una fecha
sudo journalctl -u apolo-backend --since "2024-01-01 00:00:00"
```

### Reinicio de Servicios

```bash
# Reiniciar todos los servicios
sudo systemctl restart apolo-backend apolo-billing-engine apolo-frontend

# Reiniciar solo backend
sudo systemctl restart apolo-backend
```

### Actualización del Código

```bash
# Cambiar a usuario apolo
sudo -u apolo bash
cd /opt/ApoloBilling

# Obtener últimos cambios
git pull origin main

# Recompilar backend
cd rust-backend
source ~/.cargo/env
cargo build --release

# Recompilar billing engine
cd ../rust-billing-engine
cargo build --release

# Reconstruir frontend
cd ../frontend
npm install
npm run build

# Salir de usuario apolo
exit

# Reiniciar servicios
sudo systemctl restart apolo-backend apolo-billing-engine apolo-frontend
```

### Base de Datos

```bash
# Conectar a PostgreSQL
psql -U apolo_user -d apolo_billing -h localhost

# Backup de base de datos
pg_dump -U apolo_user -h localhost apolo_billing > backup_$(date +%Y%m%d).sql

# Restaurar backup
psql -U apolo_user -h localhost apolo_billing < backup_20240101.sql
```

---

## 16. Troubleshooting

### Problema: Servicio no inicia

```bash
# Ver logs detallados
sudo journalctl -u apolo-backend -n 50 --no-pager

# Verificar permisos
ls -la /opt/ApoloBilling/rust-backend/target/release/
sudo chown -R apolo:apolo /opt/ApoloBilling
```

### Problema: Error de conexión a PostgreSQL

```bash
# Verificar que PostgreSQL está corriendo
sudo systemctl status postgresql

# Probar conexión manual
psql -U apolo_user -d apolo_billing -h localhost

# Verificar configuración pg_hba.conf
sudo cat /etc/postgresql/15/main/pg_hba.conf | grep apolo
```

### Problema: Error de conexión a Redis

```bash
# Verificar que Redis está corriendo
sudo systemctl status redis-server

# Probar conexión
redis-cli ping
```

### Problema: Frontend no carga

```bash
# Verificar que el build existe
ls -la /opt/ApoloBilling/frontend/dist/

# Verificar servicio
sudo systemctl status apolo-frontend

# Reconstruir si es necesario
cd /opt/ApoloBilling/frontend
npm run build
```

### Problema: Nginx devuelve 502

```bash
# Verificar que los backends están corriendo
curl http://localhost:8000/api/v1/health
curl http://localhost:3000

# Verificar logs de Nginx
sudo tail -f /var/log/nginx/apolobilling.error.log
```

---

## 17. Credenciales por Defecto

| Usuario | Contraseña | Rol | Uso |
|---------|------------|-----|-----|
| admin | admin123 | admin | Acceso total al sistema |

**IMPORTANTE:** Cambiar las contraseñas por defecto en producción.

---

## 18. Checklist de Deployment

- [ ] Sistema operativo Debian 12 instalado
- [ ] PostgreSQL instalado y configurado
- [ ] Redis instalado y corriendo
- [ ] Rust instalado (rustup)
- [ ] Node.js 20 LTS instalado
- [ ] Nginx instalado
- [ ] Usuario `apolo` creado
- [ ] Repositorio clonado
- [ ] Base de datos y schema creados
- [ ] Usuario admin insertado
- [ ] Backend compilado
- [ ] Billing engine compilado
- [ ] Frontend compilado
- [ ] Archivos .env configurados
- [ ] Servicios systemd creados
- [ ] Servicios habilitados e iniciados
- [ ] Nginx configurado como reverse proxy
- [ ] Certificado SSL obtenido
- [ ] Firewall configurado
- [ ] Health check exitoso
- [ ] Login funcional

---

## Quick Start (Resumen Rápido)

Para usuarios que quieren un deployment rápido después de tener las dependencias instaladas:

```bash
# 1. Clonar repositorio
sudo -u apolo git clone https://github.com/jesus-bazan-entel/ApoloBilling.git /opt/ApoloBilling

# 2. Configurar base de datos
sudo -u postgres psql -c "CREATE USER apolo_user WITH PASSWORD 'tu_password';"
sudo -u postgres psql -c "CREATE DATABASE apolo_billing OWNER apolo_user;"
psql -U apolo_user -d apolo_billing -f /opt/ApoloBilling/schema.sql

# 3. Compilar
cd /opt/ApoloBilling/rust-backend && cargo build --release
cd /opt/ApoloBilling/rust-billing-engine && cargo build --release
cd /opt/ApoloBilling/frontend && npm install && npm run build

# 4. Configurar .env en cada componente

# 5. Iniciar servicios
sudo systemctl start apolo-backend apolo-billing-engine apolo-frontend

# 6. Verificar
curl http://localhost:8000/api/v1/health
```
