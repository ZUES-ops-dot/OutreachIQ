# OutreachIQ -- Cold Email Engine in Rust

![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust&logoColor=white)
![Actix](https://img.shields.io/badge/Actix--Web-4.x-000000)
![Postgres](https://img.shields.io/badge/Postgres-15-336791?logo=postgresql&logoColor=white)
![Next.js](https://img.shields.io/badge/Next.js-14-black?logo=nextdotjs)
![License](https://img.shields.io/badge/license-MIT-green)

> Multi-tenant cold email platform with encrypted credential storage, inbox warmup, JWT-based access control, Stripe billing, and a Rust async core. Sends 10,000+ verified emails per hour from a single VPS.

**Live demo:** _coming soon_ · **[Architecture](#architecture)** · **[Quick Start](#quick-start)**

---

## What problem this solves

Most cold-email tools (Mailshake, Lemlist, Instantly) are SaaS-only and lock your data behind their UI. OutreachIQ is the same engine, self-hostable, multi-tenant from day one, with a Rust core that fits in 256 MB of RAM and SMTP credentials encrypted at rest with AES-GCM.

**Pain points solved:**
- Cold-email SaaS pricing scales linearly with volume
- Credentials stored in plaintext or "managed" by vendor
- No tenant isolation for agencies running multiple workspaces
- Domain-warmup left to manual scheduling

## Highlights

- **Multi-tenant by design** -- workspace isolation enforced at the SQL layer with row-level scoping; per-workspace usage limits, billing, and member roles
- **Encrypted SMTP creds** -- every email account row has its password sealed with AES-GCM via a versioned key (`ENCRYPTION_KEY_ID`)
- **Inbox warmup scheduler** -- gradual volume ramp with reply detection to protect domain reputation
- **JWT auth, fail-fast** -- token decode panics on missing `JWT_SECRET` rather than silently using a placeholder. Roles: owner / admin / member / viewer.
- **Reply classifier** -- Anthropic-backed intent tagging (interested / maybe later / objection / negative) drives the founder dashboard
- **Stripe billing** -- subscription webhooks, usage-based limits, signed webhook validation
- **Compliance built-in** -- one-click unsubscribe links, suppression lists, send-window enforcement
- **Async job queue** -- Postgres-backed worker for sends, replies, warmup ticks; survives restarts

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  Next.js 14 frontend (App Router, server actions)                 │
│  ─ login / register / dashboard / campaigns / warmup / billing    │
└──────────────────────┬───────────────────────────────────────────┘
                       │ JSON over HTTP (JWT cookie)
                       ▼
┌──────────────────────────────────────────────────────────────────┐
│  Rust API service (Actix-Web 4)                                   │
│  ┌────────────┐  ┌────────────┐  ┌──────────┐  ┌──────────────┐  │
│  │ auth + JWT │  │ campaigns  │  │ billing  │  │ workspace    │  │
│  └────────────┘  └────────────┘  └──────────┘  └──────────────┘  │
│  middleware: auth, tenancy scoping, rate limiting                 │
└──────────────────────┬───────────────────────────────────────────┘
                       │ SQLx
                       ▼
              ┌─────────────────┐         ┌──────────────────────┐
              │ Postgres 15     │←────────│ Worker service       │
              │ • multi-tenant  │         │ • send loop          │
              │ • RLS-style     │         │ • reply ingest       │
              │   scoping       │         │ • warmup scheduler   │
              └─────────────────┘         └──────────┬───────────┘
                       ▲                              │
                       │                              ▼
                       │                   ┌────────────────────┐
                       │                   │  SMTP (lettre)     │
                       │                   │  decrypts creds    │
                       │                   │  per send          │
                       │                   └────────────────────┘
                       │
                ┌──────┴──────┐
                │  Stripe     │  webhooks → subscription state
                └─────────────┘
```

## Tech stack

| Layer | Tech |
|-------|------|
| API | Rust + Actix-Web 4, Tokio runtime |
| DB | Postgres 15, SQLx (compile-time-checked queries) |
| Auth | `jsonwebtoken`, Argon2 password hashing |
| Crypto | `aes-gcm` for credential sealing |
| Email | `lettre` SMTP client, custom warmup scheduler |
| Frontend | Next.js 14 + Tailwind, Nord-themed dark UI |
| Billing | Stripe webhooks (signed) |
| Reply AI | Anthropic Claude classification |
| Container | Multi-stage Dockerfile, distroless runtime |

## Quick Start

```bash
git clone https://github.com/ZUES-ops-dot/OutreachIQ.git
cd OutreachIQ
docker compose up
```

This brings up Postgres + the Rust API + the worker + the Next.js frontend. The first migration runs automatically.

Visit <http://localhost:3000> and register the first user -- they become the owner of a fresh workspace.

### Without Docker

```bash
# 1. Postgres running locally on :5432
cd backend
cp .env.example .env
# Required: DATABASE_URL, JWT_SECRET, ENCRYPTION_KEY (base64 32 bytes)
sqlx migrate run
cargo run --bin api
# In a second shell:
cargo run --bin worker

# Frontend:
cd ../frontend
cp .env.example .env.local
npm install && npm run dev
```

## Configuration

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | Postgres connection string |
| `JWT_SECRET` | **Required.** Service panics if unset (no insecure fallback). |
| `ENCRYPTION_KEY` | Base64-encoded 32-byte key for SMTP credential sealing |
| `ENCRYPTION_KEY_ID` | Versioned key id for rotation |
| `STRIPE_SECRET_KEY` | Optional billing |
| `STRIPE_WEBHOOK_SECRET` | Signed webhook validation |
| `APP_URL` | Used in unsubscribe + reply tracking links |

Generate a fresh encryption key:

```bash
openssl rand -base64 32
```

## Repository layout

```
backend/
  src/
    api/                Routes (auth, campaigns, billing, email_accounts, ...)
    middleware/         JWT + tenancy scoping
    services/           Email sender, reply classifier, job queue
    models/             SQLx-mapped row types
    config.rs           Fail-fast env loader
  migrations/           SQL migrations including multi-tenancy enforcement

frontend/
  app/                  Next.js App Router pages
  components/           Reusable UI (Nord theme)

docker-compose.yml      Full local stack
Dockerfile              Multi-stage release build
Dockerfile.worker       Worker variant
```

## Roadmap

See [Issues](https://github.com/ZUES-ops-dot/OutreachIQ/issues) -- A/B subject-line variants, SMTP connection-pool tuning, timezone-aware send window, dedicated job-queue crate.

## License

MIT -- see [LICENSE](LICENSE).
