# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ApoloBilling is a real-time telecommunications billing platform for FreeSWITCH PBX environments. It handles call authorization, balance reservations, real-time billing, and CDR (Call Detail Records) generation.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      React Frontend (:3000)                      │
└─────────────────────────────────────────────────────────────────┘
              │                    │                    │
              ▼                    ▼                    ▼
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
│ Python FastAPI  │   │  Rust Backend   │   │  Rust Billing   │
│   (:8000)       │   │  (:9001)        │   │  Engine (:9000) │
│ CRUD, Dashboard │   │ CDR queries,    │   │ ESL, real-time  │
│                 │   │ exports, stats  │   │ billing         │
└────────┬────────┘   └────────┬────────┘   └────────┬────────┘
         │                     │                     │
         └─────────────────────┼─────────────────────┘
                               ▼
                    ┌─────────────────────┐
                    │  PostgreSQL + Redis │
                    └─────────────────────┘
```

**Four main components:**
- **frontend/** - React 19 + TypeScript + Vite SPA for dashboard UI
- **backend/** - Python FastAPI REST API for CRUD operations
- **rust-backend/** - High-performance Rust API for CDR queries (millions of records)
- **rust-billing-engine/** - Real-time billing processor via FreeSWITCH ESL

## Common Commands

### Backend (Python/FastAPI)
```bash
cd backend
source venv/bin/activate
pip install -r requirements.txt
python main.py                    # Start server on :8000
# OR: uvicorn main:app --host 0.0.0.0 --port 8000 --reload
python init_db_clean.py           # Initialize/reset database
```

### Frontend (React/TypeScript)
```bash
cd frontend
npm install
npm run dev        # Dev server on :3000 (proxies API to :9000)
npm run build      # Production build (runs tsc then vite build)
npm run lint       # ESLint check
npm run preview    # Preview production build
```

### Rust Backend (High-Volume CDR API)
```bash
cd rust-backend
cp .env.example .env              # Configure database URL
cargo build --release
cargo run                         # Start server on :9001
cargo test                        # Run tests
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

- `backend/main.py` - FastAPI application setup, router registration
- `frontend/src/App.tsx` - React router and main app component
- `rust-backend/src/main.rs` - High-volume CDR API server
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

## Backend API Structure

```
backend/app/
├── api/routers/          # FastAPI routers (accounts, rates, rate_cards, management)
├── models/               # SQLAlchemy models (billing.py, cdr.py, zones.py)
├── schemas/              # Pydantic schemas
├── services/             # Business logic (billing_sync.py, rating.py)
├── db/                   # Database session and dependencies
└── core/                 # Security (JWT, password hashing)
```

## Frontend Structure

```
frontend/src/
├── pages/                # Route components (Dashboard, CDR, Accounts, Rates, etc.)
├── components/           # Reusable UI (Layout, DataTable, StatCard)
├── services/api.ts       # Axios client, all API endpoints
└── lib/utils.ts          # Utility functions
```

## Rust Backend Structure (CDR API)

```
rust-backend/
├── src/main.rs           # Actix-web server setup
└── crates/
    ├── apolo-api/        # HTTP handlers and DTOs
    │   ├── handlers/cdr.rs    # CDR list, export, stats
    │   └── dto/               # Request/response types
    ├── apolo-db/         # PostgreSQL repositories
    ├── apolo-auth/       # JWT + Argon2 authentication
    ├── apolo-cache/      # Redis caching layer
    └── apolo-core/       # Shared models and traits
```

**CDR API Endpoints (Port 9001):**
- `GET /api/v1/cdrs` - List with pagination and filters
- `GET /api/v1/cdrs/{id}` - Get single CDR
- `GET /api/v1/cdrs/export` - Streaming export (CSV/JSON/JSONL)
- `GET /api/v1/cdrs/stats` - Aggregated statistics

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
# Database (shared by all backends)
DATABASE_URL=postgresql://apolo_user:PASSWORD@localhost:5432/apolo_billing
REDIS_URL=redis://localhost:6379

# Rust Backend (CDR API)
RUST_SERVER_PORT=9001
RUST_SERVER_WORKERS=4
CORS_ORIGINS=http://localhost:3000

# Rust Billing Engine
ESL_HOST=127.0.0.1
ESL_PORT=8021

# Python Backend
JWT_SECRET=...
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

# MIGRACIÓN PYTHON → RUST BACKEND (Completada)

## Resumen de la Migración

El backend Python FastAPI (`:8000`) fue migrado completamente a Rust, consolidándolo con el `rust-backend` existente. El Rust backend ahora maneja TODOS los endpoints de la API en el puerto 8000.

### Nueva Arquitectura (Post-Migración)

```
┌─────────────────────────────────────────────────────────────────┐
│                      React Frontend (:3000)                      │
│                    (Proxy → localhost:8000)                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │      Rust Backend (:8000)      │
              │  - Auth (JWT + Argon2)         │
              │  - Accounts CRUD + Topup       │
              │  - Rate Cards CRUD + LPM       │
              │  - Zones/Prefixes/Tariffs      │
              │  - CDRs (queries, export)      │
              │  - Active Calls                │
              │  - Reservations                │
              │  - Dashboard Stats             │
              └───────────────────────────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │     PostgreSQL + Redis         │
              └───────────────────────────────┘
```

---

## Fases de Migración Completadas

### Fase 1: Autenticación
**Archivos creados:**
- `crates/apolo-api/src/dto/auth.rs` - DTOs para login/logout/register
- `crates/apolo-api/src/handlers/auth.rs` - Handlers de autenticación

**Endpoints:**
```
POST /api/v1/auth/login      → Login, retorna JWT en cookie
POST /api/v1/auth/logout     → Limpia cookie
GET  /api/v1/auth/me         → Info usuario actual
POST /api/v1/auth/register   → Crear usuario (admin only)
POST /api/v1/auth/change-password → Cambiar contraseña
```

### Fase 2: Gestión de Cuentas
**Archivos creados:**
- `crates/apolo-api/src/dto/account.rs`
- `crates/apolo-api/src/handlers/account.rs`

**Endpoints:**
```
GET    /api/v1/accounts          → Lista paginada
POST   /api/v1/accounts          → Crear cuenta
GET    /api/v1/accounts/{id}     → Obtener cuenta
PUT    /api/v1/accounts/{id}     → Actualizar cuenta
POST   /api/v1/accounts/{id}/topup → Recargar saldo
```

### Fase 3: Rate Cards CRUD
**Archivos creados:**
- `crates/apolo-api/src/dto/rate_card.rs`
- `crates/apolo-api/src/handlers/rate_card.rs`

**Endpoints:**
```
GET    /api/v1/rate-cards              → Lista con filtros
POST   /api/v1/rate-cards              → Crear tarifa
GET    /api/v1/rate-cards/{id}         → Obtener tarifa
PUT    /api/v1/rate-cards/{id}         → Actualizar tarifa
DELETE /api/v1/rate-cards/{id}         → Eliminar tarifa
POST   /api/v1/rate-cards/bulk         → Importación masiva
GET    /api/v1/rate-cards/search/{phone} → Búsqueda LPM
```

### Fase 4: Rates Legacy
**Archivos creados:**
- `crates/apolo-api/src/handlers/rate.rs`

**Endpoints:**
```
GET    /api/v1/rates        → Alias de rate-cards
POST   /api/v1/rates        → Crear rate
DELETE /api/v1/rates/{id}   → Eliminar rate
```

### Fase 5: Zonas/Prefijos/Tarifas (Management)
**Archivos creados:**
- `crates/apolo-db/src/repositories/zone_repo.rs`
- `crates/apolo-db/src/repositories/prefix_repo.rs`
- `crates/apolo-db/src/repositories/tariff_repo.rs`
- `crates/apolo-api/src/dto/management.rs`
- `crates/apolo-api/src/handlers/management.rs`

**Endpoints:**
```
GET/POST        /api/v1/zonas           → CRUD zonas
PUT/DELETE      /api/v1/zonas/{id}
GET/POST        /api/v1/prefijos        → CRUD prefijos
DELETE          /api/v1/prefijos/{id}
GET/POST        /api/v1/tarifas         → CRUD tarifas
PUT/DELETE      /api/v1/tarifas/{id}
POST            /api/v1/sync-rate-cards → Sincronizar
```

### Fase 6: Active Calls y CDR Creation
**Archivos creados:**
- `crates/apolo-db/src/repositories/active_call_repo.rs`
- `crates/apolo-api/src/dto/active_call.rs`
- `crates/apolo-api/src/handlers/active_call.rs`

**Endpoints:**
```
GET    /api/v1/active-calls           → Lista llamadas activas
POST   /api/v1/active-calls           → Reportar/upsert llamada
DELETE /api/v1/active-calls/{call_id} → Remover llamada
POST   /api/v1/cdrs                   → Crear CDR
```

### Fase 7: Integración Frontend
**Archivos modificados:**

| Archivo | Cambio |
|---------|--------|
| `frontend/vite.config.ts` | Proxy configurado a puerto 8000 |
| `frontend/src/api/client.ts` | Actualizado para endpoints Rust |
| `frontend/src/types/index.ts` | Tipos actualizados |
| `frontend/src/pages/Balance.tsx` | Reescrito con Account/topup |
| `frontend/src/pages/ActiveCalls.tsx` | Corregidos tipos |
| `frontend/src/pages/CDR.tsx` | Corregidos tipos |
| `frontend/src/pages/Dashboard.tsx` | Corregidos tipos |

**Archivos creados en Rust:**
| Archivo | Propósito |
|---------|-----------|
| `rust-backend/.env` | Configuración servidor (puerto 8000) |
| `handlers/dashboard.rs` | Endpoint `/api/v1/stats` |
| `handlers/reservation.rs` | Endpoints `/api/v1/reservations` |

---

## Instrucciones de Despliegue

### 1. Detener Python Backend
```bash
# Encontrar y detener proceso en puerto 8000
PID=$(lsof -t -i :8000)
if [ -n "$PID" ]; then
    kill $PID
fi
```

### 2. Iniciar Rust Backend
```bash
cd /opt/ApoloBilling/rust-backend
cargo run --release
# O en background:
nohup cargo run --release > /tmp/rust-backend.log 2>&1 &
```

### 3. Crear Tabla de Usuarios (Primera vez)
```sql
-- La tabla debe llamarse 'usuarios' con columna 'password'
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

### 4. Crear Usuario Admin
```bash
# Generar hash Argon2 para la contraseña
cd /opt/ApoloBilling/rust-backend
cargo run --example gen_hash -p apolo-auth
# Esto genera el hash para "admin123"
```

```sql
-- Insertar admin (usar el hash generado)
INSERT INTO usuarios (username, password, nombre, apellido, email, role, activo)
VALUES (
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$...[hash generado]...',
    'Administrador',
    'Sistema',
    'admin@apolobilling.com',
    'admin',
    true
);
```

### 5. Iniciar Frontend
```bash
cd /opt/ApoloBilling/frontend
npm run dev
# Acceder a http://localhost:3000
```

---

## Configuración del Rust Backend

### Archivo: `rust-backend/.env`
```bash
# Database
DATABASE_URL=postgresql://apolo_user:PASSWORD@localhost:5432/apolo_billing
DATABASE_MAX_CONNECTIONS=20

# Server - Puerto 8000 (reemplaza Python)
RUST_SERVER_HOST=0.0.0.0
RUST_SERVER_PORT=8000
RUST_SERVER_WORKERS=4

# CORS
CORS_ORIGINS=http://localhost:3000,http://127.0.0.1:3000

# JWT
JWT_SECRET=apolo-billing-secret-key-change-in-production
JWT_EXPIRATION_SECS=1800

# Logging
LOG_LEVEL=info
RUST_LOG=apolo_billing=debug,apolo_api=debug,actix_web=info
```

### Archivo: `frontend/vite.config.ts`
```typescript
export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://127.0.0.1:8000',
        ws: true,
      },
    },
  },
})
```

---

## Credenciales por Defecto

| Usuario | Contraseña | Rol |
|---------|------------|-----|
| admin | admin123 | admin |

---

## Verificación del Sistema

```bash
# 1. Verificar Rust backend
curl http://localhost:8000/api/v1/health
# Respuesta: {"service":"apolo-billing-rust","status":"healthy","version":"1.0.0"}

# 2. Verificar stats (público)
curl http://localhost:8000/api/v1/stats

# 3. Login y obtener cookie
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' \
  -c /tmp/cookies.txt

# 4. Verificar endpoints protegidos
curl http://localhost:8000/api/v1/accounts -b /tmp/cookies.txt
curl http://localhost:8000/api/v1/auth/me -b /tmp/cookies.txt

# 5. Verificar proxy frontend
curl http://localhost:3000/api/v1/health
```

---

## Notas Importantes

1. **Tabla de usuarios**: El Rust backend usa tabla `usuarios` (no `users`) con columna `password` (no `password_hash`)

2. **Hashing de contraseñas**: Usar Argon2id. Generar con:
   ```bash
   cd rust-backend && cargo run --example gen_hash -p apolo-auth
   ```

3. **Autenticación**: JWT en cookie HTTP-only llamada `token`

4. **Frontend proxy**: Todas las peticiones `/api/*` se redirigen automáticamente al backend Rust

5. **Python backend**: Ya no es necesario. Puede eliminarse o mantenerse como backup

---

## Estructura Final de Handlers Rust

```
rust-backend/crates/apolo-api/src/handlers/
├── mod.rs              # Exports y configuración
├── account.rs          # CRUD cuentas + topup
├── active_call.rs      # Llamadas activas + crear CDR
├── auth.rs             # Login/logout/me/register
├── cdr.rs              # CDRs (list, get, export, stats)
├── dashboard.rs        # Stats del dashboard
├── management.rs       # Zonas/Prefijos/Tarifas
├── rate.rs             # Rates legacy
├── rate_card.rs        # Rate cards CRUD + LPM
└── reservation.rs      # Reservaciones de saldo
```

---

## Comandos Útiles Post-Migración

```bash
# Compilar Rust backend
cd /opt/ApoloBilling/rust-backend
cargo build --release

# Ejecutar tests
cargo test

# Ver logs en tiempo real
tail -f /tmp/rust-backend.log

# Reiniciar backend
pkill -f apolo-billing
cd /opt/ApoloBilling/rust-backend && nohup cargo run --release > /tmp/rust-backend.log 2>&1 &

# Compilar frontend
cd /opt/ApoloBilling/frontend
npm run build
```
