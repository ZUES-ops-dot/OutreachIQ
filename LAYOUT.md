# OutreachIQ Codebase Layout

## Overview

OutreachIQ is a signal-based lead generation and outreach platform. It consists of a **Rust backend** (Actix-web + SQLx + Postgres) and a **Next.js frontend** (React + Tailwind CSS).

---

## Project Structure

```
OutreachIQ/
├── backend/                    # Rust API server
│   ├── migrations/             # SQLx database migrations
│   ├── src/
│   │   ├── api/                # HTTP route handlers
│   │   ├── bin/                # Binary entry points
│   │   ├── db/                 # Database utilities
│   │   ├── middleware/         # Auth middleware
│   │   ├── models/             # Data models
│   │   ├── services/           # Business logic
│   │   ├── main.rs             # API server entry
│   │   └── lib.rs              # Library exports
│   ├── Cargo.toml
│   └── .env.example
│
├── frontend/                   # Next.js web app
│   ├── app/                    # App router pages
│   │   ├── dashboard/          # Dashboard pages
│   │   │   ├── founder/        # Main founder dashboard
│   │   │   ├── settings/       # Settings page
│   │   │   └── ...             # Other dashboard pages
│   │   ├── login/              # Auth pages
│   │   ├── register/
│   │   ├── signals/            # Public signals feed
│   │   ├── pricing/            # Pricing page
│   │   └── page.tsx            # Landing page
│   ├── lib/
│   │   └── api.ts              # API client
│   ├── tailwind.config.ts      # Tailwind + Nord theme
│   └── package.json
│
└── LAYOUT.md                   # This file
```

---

## Backend (`/backend`)

### Entry Points

| File | Description |
|------|-------------|
| `src/main.rs` | HTTP API server (port 8080) |
| `src/bin/worker.rs` | Background job runner |

### API Modules (`src/api/`)

| Module | Routes | Description |
|--------|--------|-------------|
| `auth.rs` | `/api/auth/*` | Login, register, JWT tokens |
| `founder_dashboard.rs` | `/api/founder/*` | Main dashboard data, campaigns, inbox health |
| `campaigns.rs` | `/api/campaigns/*` | Campaign CRUD |
| `leads.rs` | `/api/leads/*` | Lead generation, verification |
| `signals.rs` | `/api/signals/*` | Company signal feed |
| `email_accounts.rs` | `/api/email-accounts/*` | Inbox management, warmup |
| `analytics.rs` | `/api/analytics/*` | Stats and metrics |
| `billing.rs` | `/api/billing/*` | Stripe integration |

### Services (`src/services/`)

| Service | Purpose |
|---------|---------|
| `lead_generator.rs` | Generate leads from signals |
| `email_sender.rs` | Send outreach emails |
| `email_verifier.rs` | Verify email addresses |
| `warmup_service.rs` | Email warmup automation |
| `deliverability.rs` | Inbox health monitoring |
| `reply_classifier.rs` | AI reply intent classification (Claude) |
| `auto_pause.rs` | Auto-pause campaigns on bad metrics |
| `github_connector.rs` | GitHub activity signals |
| `wellfound_connector.rs` | Job posting signals |
| `signal_tracker.rs` | Signal aggregation |

### Models (`src/models/`)

| Model | Table |
|-------|-------|
| `user.rs` | `users` |
| `workspace.rs` | `workspaces` |
| `campaign.rs` | `campaigns` |
| `lead.rs` | `leads` |
| `company.rs` | `companies` |
| `signal.rs` | `signals` |

### Database Migrations (`migrations/`)

| Migration | Description |
|-----------|-------------|
| `20240101000000_initial.sql` | Base schema (users, workspaces, campaigns, leads) |
| `20240102000000_multi_tenancy_security.sql` | RLS policies |
| `20240103000000_signals_schema.sql` | Signal tracking tables |
| `20240104000000_founder_dashboard.sql` | Inbox health, reply classification, auto-pause |

---

## Frontend (`/frontend`)

### Pages (`app/`)

| Route | File | Description |
|-------|------|-------------|
| `/` | `page.tsx` | Landing page (Nord theme) |
| `/dashboard` | `dashboard/page.tsx` | Redirects to `/dashboard/founder` |
| `/dashboard/founder` | `dashboard/founder/page.tsx` | **Main dashboard** (campaigns, inbox health, replies) |
| `/dashboard/settings` | `dashboard/settings/page.tsx` | User settings |
| `/signals` | `signals/page.tsx` | Public signal feed |
| `/login` | `login/page.tsx` | Login form |
| `/register` | `register/page.tsx` | Registration form |
| `/pricing` | `pricing/page.tsx` | Pricing tiers |

### API Client (`lib/api.ts`)

Single API client class with methods for all backend endpoints:

```typescript
const api = new ApiClient();

// Auth
api.login({ email, password })
api.register({ email, password, name })

// Founder Dashboard
api.getFounderDashboard()
api.pauseFounderCampaign(id)
api.resumeFounderCampaign(id)
api.getInboxHealth()
api.getReplies()
api.actionReply(id, action)

// Campaigns
api.getCampaigns()
api.createCampaign({ name, vertical })
api.startCampaign(id)

// Leads
api.searchLeads({ vertical, role, signals })
api.verifyLeads(emails)
```

### Styling

**Theme**: Nord-based eye-friendly palette (defined in `tailwind.config.ts`)

| Color | Hex | Usage |
|-------|-----|-------|
| `nord-bg` | `#2e3440` | Page backgrounds |
| `nord-surface` | `#3b4252` | Card backgrounds |
| `nord-elevated` | `#434c5e` | Elevated elements |
| `nord-text` | `#eceff4` | Primary text |
| `nord-text-muted` | `#d8dee9` | Secondary text |
| `nord-frost3` | `#81a1c1` | Primary accent (blue) |
| `nord-success` | `#a3be8c` | Success states (green) |
| `nord-warning` | `#ebcb8b` | Warning states (yellow) |
| `nord-error` | `#bf616a` | Error states (red) |
| `nord-purple` | `#b48ead` | Accent (purple) |

---

## Environment Variables

### Backend (`.env`)

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/outreachiq
JWT_SECRET=your-super-secret-jwt-key-min-32-chars
ENCRYPTION_KEY=base64-encoded-32-byte-key
ANTHROPIC_API_KEY=sk-ant-...  # For reply classification
GITHUB_TOKEN=ghp_...          # For signal ingestion
FRONTEND_URL=http://localhost:3000
```

### Frontend (`.env.local`)

```bash
NEXT_PUBLIC_API_URL=http://localhost:8080/api
```

---

## Running Locally

### 1. Start Postgres (Docker)

```bash
docker run --name outreachiq-postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=outreachiq \
  -p 5432:5432 \
  -d postgres:15
```

### 2. Run Migrations

```bash
cd backend
$env:DATABASE_URL = "postgres://postgres:postgres@localhost:5432/outreachiq"
cargo sqlx migrate run
```

### 3. Start Backend

```bash
cargo run --bin outreachiq-api
# API runs on http://localhost:8080
```

### 4. Start Frontend

```bash
cd frontend
npm install
npm run dev
# App runs on http://localhost:3000
```

---

## Key Features

### Founder Dashboard (`/dashboard/founder`)

- **Overview Cards**: Cost per meeting, inbox count, paused campaigns, replies
- **Campaign Cards**: Health status, auto-pause alerts, pause/resume actions
- **Inbox Health**: Per-inbox spam/bounce/reply rates, daily limits
- **Reply Cards**: Intent classification (interested/objection/maybe_later), quick actions

### Signal System (`/signals`)

- Real-time company signals (hiring, funding, GitHub activity)
- Confidence scoring based on signal strength
- Public feed with email capture CTA

### Auto-Pause

Campaigns automatically pause when:
- Spam rate > 3%
- Bounce rate > 8%
- Reply rate drops > 40% in 48h

---

## Navigation Flow

```
Landing Page (/)
    ├── "Go to Dashboard" → /dashboard/founder
    ├── "Start Generating Leads" → /dashboard/founder
    └── "View Live Signals" → /signals

Founder Dashboard (/dashboard/founder)
    ├── "New Campaign" → Modal (CSV upload)
    ├── "Settings" → /dashboard/settings
    └── Campaign actions (pause/resume)
```

---

## Notes

- **Auth**: JWT-based, stored in localStorage. Dashboard shows demo data when not authenticated.
- **Multi-tenancy**: Workspace-based isolation with RLS policies.
- **Styling**: Nord theme for eye comfort during long sessions.
