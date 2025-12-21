# OutreachIQ

Signal-Based Lead Generation & Cold Outreach Platform built with Rust backend and Next.js frontend.

## 🚀 Features

- **Multi-Tenant Architecture** - Secure workspace isolation for multiple customers
- **JWT Authentication** - Role-based access control (owner, admin, member, viewer)
- **Real Email Sending** - SMTP integration with encrypted credentials
- **Inbox Warmup** - Gradual volume increase to protect domain reputation
- **Compliance Built-in** - One-click unsubscribe, suppression lists
- **Usage Tracking** - Per-workspace limits and billing integration
- **Stripe Payments** - Subscription management with webhooks

## Core Philosophy

- **Fresh over Size**: Generate leads on-demand, not from stale databases
- **Signal-Driven**: Track buying intent signals (hiring, funding, tech changes)
- **Deliverability First**: Built-in warmup, verification, and reputation management
- **Vertical Focus**: Start with 3 deep verticals (SaaS, Web3, Agency)

## Project Structure

```
outreachiq/
├── backend/                    # Rust backend (Actix-web)
│   ├── src/
│   │   ├── api/               # API routes (auth, leads, campaigns, billing, compliance)
│   │   ├── models/            # Data models
│   │   ├── services/          # Business logic (email sender, warmup, scheduler)
│   │   ├── middleware/        # Auth middleware
│   │   ├── db/                # Database
│   │   └── bin/               # Worker binary
│   └── migrations/            # SQL migrations
├── frontend/                   # Next.js frontend
├── docker-compose.yml          # Full stack deployment
└── .env.example               # Environment variables template
```

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 15+
- Node.js 18+
- Docker (optional)

### Using Docker

```bash
docker-compose up -d
```

This starts:
- PostgreSQL on port 5432
- Backend API on port 8080
- Frontend on port 3000

### Manual Setup

1. **Start PostgreSQL**

```bash
# Create database
createdb outreachiq
```

2. **Backend**

```bash
cd backend
cp .env.example .env
# Edit .env with your database URL

cargo run
```

3. **Frontend**

```bash
cd frontend
npm install
npm run dev
```

## API Endpoints

### Leads

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/leads` | Get all leads |
| GET | `/api/leads/{id}` | Get lead by ID |
| POST | `/api/leads/search` | Generate leads by vertical |
| POST | `/api/leads/verify` | Verify email addresses |
| GET | `/api/leads/signals/{domain}` | Get company signals |
| DELETE | `/api/leads/{id}` | Delete a lead |

### Campaigns

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/campaigns` | Get all campaigns |
| POST | `/api/campaigns` | Create campaign |
| GET | `/api/campaigns/{id}` | Get campaign by ID |
| PUT | `/api/campaigns/{id}` | Update campaign |
| DELETE | `/api/campaigns/{id}` | Delete campaign |
| POST | `/api/campaigns/{id}/start` | Start campaign |
| POST | `/api/campaigns/{id}/pause` | Pause campaign |
| GET | `/api/campaigns/{id}/leads` | Get campaign leads |
| POST | `/api/campaigns/{id}/leads` | Add leads to campaign |

### Analytics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/analytics/overview` | Dashboard overview stats |
| GET | `/api/analytics/campaigns` | Campaign performance |
| GET | `/api/analytics/leads` | Lead analytics |
| GET | `/api/analytics/deliverability` | Deliverability report |

## Example Usage

### Generate Leads

```bash
curl -X POST http://localhost:8080/api/leads/search \
  -H "Content-Type: application/json" \
  -d '{
    "vertical": "saas",
    "role": "CEO",
    "limit": 20
  }'
```

### Verify Emails

```bash
curl -X POST http://localhost:8080/api/leads/verify \
  -H "Content-Type: application/json" \
  -d '["test@example.com", "ceo@startup.io"]'
```

### Create Campaign

```bash
curl -X POST http://localhost:8080/api/campaigns \
  -H "Content-Type: application/json" \
  -d '{
    "name": "SaaS Founders Q1",
    "vertical": "saas"
  }'
```

## Services

### Lead Generator
Generates leads based on vertical (SaaS, Web3, Agency) with realistic mock data. In production, integrates with:
- GitHub API for developer leads
- Job boards for hiring signals
- Social media for engagement signals

### Email Verifier
Multi-step verification:
1. Syntax validation
2. MX record check
3. Disposable email detection
4. Role-based email detection
5. Confidence scoring

### Signal Tracker
Tracks buying intent signals:
- Hiring activity
- Funding events
- Tech stack changes
- Growth indicators

### Deliverability Service
Manages email sending health:
- Warmup scheduling
- Health score calculation
- Domain authentication checks
- Deliverability reporting

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `JWT_SECRET` | Secret for JWT tokens (min 32 chars) | Required |
| `ENCRYPTION_KEY` | AES-256 key for SMTP passwords | Required |
| `ENCRYPTION_KEY_ID` | Key identifier for rotation | `default-key-v1` |
| `FRONTEND_URL` | Frontend URL for CORS | `http://localhost:3000` |
| `APP_URL` | App URL for email links | `http://localhost:3000` |
| `STRIPE_SECRET_KEY` | Stripe API secret key | Optional |
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook signing secret | Optional |
| `RUST_LOG` | Log level | `info` |

## Architecture

### Services

| Service | Port | Description |
|---------|------|-------------|
| `postgres` | 5432 | PostgreSQL database |
| `backend` | 8080 | Rust API server |
| `worker` | - | Background job processor |
| `frontend` | 3000 | Next.js web app |

### Worker Jobs

The worker binary (`outreachiq-worker`) processes:
- **SendEmail** - Campaign email delivery via SMTP
- **VerifyEmail** - Email address verification
- **WarmupEmail** - Inbox warmup emails
- **ProcessCampaign** - Campaign scheduling

### Pricing Tiers

| Tier | Price | Leads/mo | Inboxes | Emails/mo |
|------|-------|----------|---------|-----------|
| Starter | $97 | 1,000 | 1 | 500 |
| Professional | $297 | 10,000 | 5 | 5,000 |
| Business | $997 | 50,000 | 20 | 25,000 |

## API Endpoints

### Authentication
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | Register new user |
| POST | `/api/auth/login` | Login and get JWT |
| GET | `/api/auth/me` | Get current user |
| POST | `/api/auth/refresh` | Refresh JWT token |

### Billing
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/billing/pricing` | Get pricing tiers |
| POST | `/api/billing/checkout` | Create Stripe checkout |
| POST | `/api/billing/portal` | Create billing portal session |
| GET | `/api/billing/subscription` | Get current subscription |
| GET | `/api/billing/usage` | Get usage stats |

### Compliance
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/compliance/unsubscribe` | Handle unsubscribe (public) |
| GET | `/api/compliance/suppression` | Get suppression list |
| POST | `/api/compliance/suppression` | Add to suppression list |
| DELETE | `/api/compliance/suppression/{email}` | Remove from suppression |

## License

MIT
# OutreachIQ 
