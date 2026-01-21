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
