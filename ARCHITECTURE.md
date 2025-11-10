<!--
  OutreachIQ – Full-stack architecture overview
  Generated on 2025-12-18
-->

# OutreachIQ Architecture Guide

This document explains how every major part of the OutreachIQ codebase fits together, covering both the Rust backend and the Next.js frontend. Use it as the definitive reference when onboarding new contributors or reviewing the system design end‑to‑end.

---

## 1. Repository Layout

```
outreachiq/
├── backend/                    # Rust/Actix backend
│   ├── src/
│   │   ├── api/                # HTTP route handlers grouped by domain
│   │   ├── models/             # SQLx data models & DTOs
│   │   ├── services/           # Business logic & integrations
│   │   ├── middleware/         # Auth/JWT utilities
│   │   ├── db/                 # Connection helpers & migrations
│   │   ├── bin/                # Background worker entry point
│   │   └── main.rs             # API server bootstrap
│   ├── migrations/             # SQL migrations (sqlx)
│   └── Cargo.*                 # Rust project metadata
├── frontend/                   # Next.js 14 App Router project
│   ├── app/                    # Route segments (landing, dashboard, signals, auth, etc.)
│   ├── components/             # Shared UI (if any)
│   ├── lib/api.ts              # Typed API client with auth helpers
│   └── tailwind.config.*       # Nord-themed design tokens
├── docker-compose.yml          # Local stack (Postgres + backend + worker + frontend)
├── README.md / LAUNCH_GUIDE.md # Default documentation
└── ARCHITECTURE.md             # This document
```

Environment templates live in `.env.example` (root/backend) and `.env.local` (frontend). Docker Compose wires everything together for local development.

---

## 2. Backend (Rust / Actix-web)

### 2.1 Entry Point (`src/main.rs`)

1. Loads `.env` via `dotenvy`.
2. Creates a PostgreSQL pool (`sqlx::postgres::PgPoolOptions`).
3. Runs all migrations (`sqlx::migrate!("./migrations")`).
4. Configures CORS using `FRONTEND_URL`.
5. Registers all REST scopes under `/api` (auth, leads, campaigns, analytics, email accounts, compliance, billing, signals, founder dashboard).
6. Wraps middleware:
   - Logger (`actix_middleware::Logger`)
   - Custom JWT middleware (`middleware::auth::AuthMiddleware`) for protected endpoints.
7. Launches an `HttpServer` on `0.0.0.0:8080` and exposes `/health` for probes.

### 2.2 API Layer (`src/api`)

Each module exposes a `configure` fn that mounts routes on the Actix scope:

| Module | Highlights |
| ------ | ---------- |
| `auth.rs` | Registers/logs in users (`/auth/register`, `/auth/login`, `/auth/me`, `/auth/refresh`). Uses Argon2 for password hashing, issues JWTs with configurable secret. |
| `leads.rs` | Workspace-scoped lead management. Provides CRUD, lead generation (`POST /leads/search` calling `LeadGenerator`), email verification, signal lookups, and deletion. Enforces monthly usage caps via SQL queries. |
| `campaigns.rs` | Campaign CRUD, start/pause actions, attaching leads, retrieving campaign metadata/statistics. |
| `analytics.rs` | Aggregated stats (overview, campaign performance, lead analytics, deliverability). |
| `email_accounts.rs` | Manages sending inboxes, SMTP credentials (encrypted), warmup start/pause, health stats. |
| `compliance.rs` | Suppression list, unsubscribe endpoints, token handling for public opt-outs. |
| `billing.rs` | Pricing tiers, checkout/portal sessions (Stripe), usage reporting. |
| `signals.rs` | Public signal feed & stats (hiring, funding, GitHub), ingestion endpoints. |
| `founder_dashboard.rs` | Aggregated dashboard data: overview metrics, inbox health cards, reply inbox, cost per meeting, meetings scheduler, auto-pause events. Powers the `/dashboard/founder` view. |

All handlers accept a shared `PgPool` via `web::Data<PgPool>` and return typed JSON responses (`serde`).

### 2.3 Models (`src/models`)

Defines the SQLx structs & DTOs for each domain (users, leads, campaigns, signals, companies, billing entities, etc.). Key traits:

- `sqlx::FromRow`: enables direct mapping from DB rows.
- `serde::{Serialize, Deserialize}`: used for JSON payloads.
- Conversion helpers (e.g., `impl From<User> for UserResponse`) keep API responses clean.

### 2.4 Services (`src/services`)

Encapsulate business logic and 3rd-party integrations:

- `lead_generator.rs`: Generates leads per industry (SaaS, Web3, Agency, Fintech, etc.) with mock data or external connectors. Called by `/leads/search`.
- `email_verifier.rs`: Multi-stage verification (MX lookup, disposable detection, scoring).
- `warmup_service.rs`: Inbox warmup scheduler, health tracking.
- `email_sender.rs`: Sends campaign emails via SMTP, handles unsubscribe tokens.
- `campaign_scheduler.rs`: Queues campaign jobs, enforces per-inbox limits.
- `deliverability.rs`: Health score updates, spam/bounce tracking.
- `job_queue.rs`: Simple async job system for worker processing.
- `encryption.rs`: AES-256 utilities for SMTP credentials (requires `ENCRYPTION_KEY`/`ENCRYPTION_KEY_ID`).
- `github_connector.rs` / `wellfound_connector.rs`: Scrape/ingest external signals (GitHub activity, job postings).
- `signal_tracker.rs`: Orchestrates signal ingestion across companies.
- `auto_pause.rs`: Monitors campaigns, auto-pauses when reputation drops.

Each service is reusable by API handlers and the worker binary.

### 2.5 Middleware (`src/middleware/auth.rs`)

Implements:

- `AuthMiddleware`: Validates `Authorization: Bearer <jwt>` headers, attaches claims to the request context.
- Helper functions (`extract_claims`, `parse_workspace_id`, `require_role`, etc.) used across APIs for RBAC.

### 2.6 Database Layer (`src/db`)

- Connection utilities and shared queries.
- Migrations describe tables for users, workspaces, leads, campaigns, signals, usage metrics, suppression lists, etc. (`sqlx migrate run` keeps schema in sync.)

### 2.7 Worker Binary (`src/bin/worker.rs`)

Compiled via `cargo run --bin outreachiq-worker`. Processes queued jobs for:

- Campaign sending
- Warmup emails
- Verification batches
- Auto-pause health checks

Uses the same services/models as the API but is optimized for background execution.

---

## 3. Frontend (Next.js 14 + Tailwind/Nord)

### 3.1 Application Structure (`frontend/app`)

Uses the App Router with server components. Key routes:

| Route | Description |
| ----- | ----------- |
| `/` | Marketing landing page with CTA buttons wired to `/dashboard/founder` (auth-gated) and `/signals`. |
| `/login`, `/register` | Auth forms calling `api.login` / `api.register`. Tokens stored in `localStorage`. |
| `/dashboard/layout.tsx` | Provides the global Nord-themed dashboard shell (top nav, gradients). |
| `/dashboard/founder` | Founder HQ: overview cards, campaign cards, inbox health, replies, modals. Falls back to demo data when API fails. |
| `/dashboard/leads` | Lead generation, filters, CSV export, add-to-campaign modals. Uses `api.getLeads`, `api.searchLeads`, etc. |
| `/dashboard/settings` | Multi-section settings (email, leads, campaigns, notifications, API, email generator). `handleSave` calls `api.updateFounderSettings`. |
| `/dashboard/campaigns` | Campaign list cards, status badges, actions, progress stats. |
| `/dashboard/warmup` | Inbox warmup control center (charts via Recharts, modals, linking to campaigns). |
| `/signals` | Public signal feed (buying-intent alerts). Fetches `/api/signals/feed`/`stats`, falls back to multi-industry demo data when offline. |

Nord design tokens are shared via Tailwind classes (`bg-nord-surface`, `text-nord-text`, etc.).

### 3.2 API Client (`frontend/lib/api.ts`)

Typed `ApiClient` abstraction:

- `request<T>(endpoint, options, requiresAuth)` centralizes fetch logic, JSON parsing, and 401 handling.
- `setAuthData` / `getStoredToken` / `clearAuthData` manage auth state in `localStorage`.
- Exposes domain-specific helpers (leads, campaigns, analytics, warmup, founder dashboard, billing, signals, etc.).
- Ensures requests automatically include `Authorization: Bearer <token>` when authenticated.

### 3.3 Auth Flow

1. User registers/logs in via `/login` or `/register`.
2. Response from `/api/auth/login|register` includes `token` + `user`.
3. Token/user stored in localStorage; Next.js pages read via `isAuthenticated`.
4. Protected dashboard routes assume the token exists; API calls redirect to `/login` on 401.

### 3.4 UI Patterns

- Heavy use of Nord color palette, glassmorphism backgrounds, gradients.
- Recharts for charts (warmup health).
- Lucide-react icons for consistent visual language.
- Components are mostly page-local; modals (e.g., `NewCampaignModal`) live alongside pages for simplicity.

### 3.5 Demo / Offline Behavior

Several dashboard pages include mock data fallbacks so the UI remains functional when the backend is offline:

- `/dashboard/founder`: `getDemoData()`
- `/dashboard/leads`: adds demo leads when API fails (TODO: optional)
- `/signals`: now falls back to `DEMO_SIGNALS` + `DEMO_STATS` if fetch throws.

This dual-mode approach makes demos instant while still hitting the live API after login.

---

## 4. Data & Control Flow

1. **Auth**: Users register via `/api/auth/register`, login via `/api/auth/login`, receive JWT stored client-side. JWT secret configured via `JWT_SECRET`.
2. **Lead Generation**: Frontend calls `api.searchLeads`. Backend checks monthly limits, invokes `LeadGenerator`, verifies emails, persists to `leads` table, returns enriched leads.
3. **Campaign Ops**: Campaign APIs manage lifecycle; worker dequeues sending jobs, using `email_sender` + SMTP creds stored securely.
4. **Signals**: Public feed at `/api/signals/feed`. `SignalTracker` ingests from GitHub/Wellfound connectors; data stored in `signals` table. Frontend displays aggregated stats and cards.
5. **Warmup & Deliverability**: `warmup_service` gradually increases volume, `deliverability` recalculates health. Founder dashboard displays status via `founder_dashboard` API.
6. **Billing**: `billing.rs` wraps Stripe checkout/portal endpoints; usage tracked in `usage_metrics`.

---

## 5. Environment & Deployment

### Mandatory Environment Vars

| Variable | Purpose |
| -------- | ------- |
| `DATABASE_URL` | Postgres DSN used by both API and worker. |
| `JWT_SECRET` | Symmetric signing key for tokens (>=32 chars). |
| `ENCRYPTION_KEY` / `ENCRYPTION_KEY_ID` | AES key for SMTP credential encryption. |
| `FRONTEND_URL`, `APP_URL` | CORS + link generation. |
| `NEXT_PUBLIC_API_URL` (frontend) | Base URL for API client (e.g., `http://localhost:8080/api`). |

Optional: `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `GITHUB_TOKEN`, etc.

### Running Locally

```
# Backend
cd backend
sqlx migrate run
cargo run --bin outreachiq-api

# Frontend
cd frontend
npm install
npm run dev

# (Optional) Worker
cargo run --bin outreachiq-worker
```

Or `docker-compose up -d` to start Postgres + backend + worker + frontend together.

---

## 6. Notable Design Choices

- **Actix + SQLx**: Async-first backend with compile-time SQL checking.
- **Modular APIs**: Each domain (leads, campaigns, signals, etc.) has its own module for clarity.
- **Service Layer**: Business logic extracted from HTTP handlers for reuse (API + worker).
- **Nord UI System**: Tailwind utility classes implement a cohesive dark-themed experience across pages.
- **Demo Fallbacks**: Key dashboards render demo content if the API is unreachable, improving UX for demos/sandboxes.
- **JWT Everywhere**: Same token works for both API and worker-triggered webhooks.
- **Background Processing**: Resource-intensive operations (sending, warmup, classification) delegated to the worker binary.

---

## 7. Suggested Next Steps

1. **Central Documentation**: Keep this file updated alongside code changes.
2. **API Schema Docs**: Consider generating OpenAPI specs for the backend modules.
3. **Component Library**: Factor recurring Nord components (cards, tables, modals) into a shared folder for reuse.
4. **Observability**: Add structured logging/metrics dashboards for worker + API health.

---

This overview now captures how every moving part of OutreachIQ interacts. For deeper dives, inspect the referenced files (e.g., `backend/src/api/*`, `frontend/lib/api.ts`, each `app/dashboard/*/page.tsx`). Update this document whenever significant architectural changes land to keep the bird’s-eye view accurate.
