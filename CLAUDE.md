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

# DEPLOYMENT

## Quick Start

### 1. Start Rust Backend
```bash
cd /opt/ApoloBilling/rust-backend
cp .env.example .env              # Configure if needed
cargo run --release               # Start server on :8000
# Or in background:
nohup cargo run --release > /tmp/rust-backend.log 2>&1 &
```

### 2. Start Frontend
```bash
cd /opt/ApoloBilling/frontend
npm install
npm run dev                       # Dev server on :3000
# Or for production:
npm run build && npm run preview
```

### 3. Verify System
```bash
# Health check
curl http://localhost:8000/api/v1/health

# Login
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' \
  -c /tmp/cookies.txt

# Test protected endpoint
curl http://localhost:8000/api/v1/accounts -b /tmp/cookies.txt
```

## Database Setup (First Time)

### Create Users Table
```sql
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
```

### Create Admin User
```bash
# Generate Argon2 hash for password
cd /opt/ApoloBilling/rust-backend
cargo run --example gen_hash -p apolo-auth
```

```sql
INSERT INTO usuarios (username, password, nombre, apellido, email, role, activo)
VALUES (
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$...[generated_hash]...',
    'Administrador',
    'Sistema',
    'admin@apolobilling.com',
    'admin',
    true
);
```

## Default Credentials

| User | Password | Role |
|------|----------|------|
| admin | admin123 | admin |

## Important Notes

1. **Users table**: Uses `usuarios` table with `password` column (not `users` or `password_hash`)
2. **Password hashing**: Argon2id via `cargo run --example gen_hash -p apolo-auth`
3. **Authentication**: JWT stored in HTTP-only cookie named `token`
4. **Frontend proxy**: All `/api/*` requests proxied to Rust backend at :8000

## Useful Commands

```bash
# Build backend
cd /opt/ApoloBilling/rust-backend && cargo build --release

# Run tests
cargo test

# View logs
tail -f /tmp/rust-backend.log

# Restart backend
pkill -f apolo-billing
nohup cargo run --release > /tmp/rust-backend.log 2>&1 &

# Build frontend for production
cd /opt/ApoloBilling/frontend && npm run build
```
