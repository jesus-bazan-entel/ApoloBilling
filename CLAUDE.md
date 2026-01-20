# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ApoloBilling is a real-time telecommunications billing platform for FreeSWITCH PBX environments. It handles call authorization, balance reservations, real-time billing, and CDR (Call Detail Records) generation.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────────┐
│  React Frontend │────▶│  FastAPI Backend │────▶│  PostgreSQL + Redis │
│   (Port 3000)   │     │   (Port 8000)    │     └─────────────────────┘
└─────────────────┘     └────────┬─────────┘
                                 │
                        ┌────────▼─────────┐     ┌─────────────────┐
                        │  Rust Billing    │────▶│   FreeSWITCH    │
                        │  Engine (:9000)  │◀────│   (ESL Events)  │
                        └──────────────────┘     └─────────────────┘
```

**Three main components:**
- **frontend/** - React 19 + TypeScript + Vite SPA for dashboard UI
- **backend/** - Python FastAPI REST API for CRUD operations
- **rust-billing-engine/** - High-performance Rust processor for real-time billing via ESL

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
DATABASE_URL=postgresql://apolo_user:PASSWORD@localhost:5432/apolo_billing
REDIS_URL=redis://localhost:6379
ESL_HOST=127.0.0.1
ESL_PORT=8021
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
