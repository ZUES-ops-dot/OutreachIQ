# OutreachIQ Launch Guide

## Prerequisites
- Docker & Docker Compose installed
- Node.js 18+ installed
- PostgreSQL client tools (optional, for manual DB inspection)

## Step 1: Generate Environment Variables

### Backend (.env)
```bash
cd backend
cp .env.example .env
```

Edit `backend/.env` and set:
```
DATABASE_URL=postgres://postgres:postgres@postgres:5432/outreachiq
JWT_SECRET=your-super-secret-jwt-key-change-in-production-min-32-chars
ENCRYPTION_KEY=$(openssl rand -base64 32)
ENCRYPTION_KEY_ID=default-key-v1
FRONTEND_URL=http://localhost:3000
APP_URL=http://localhost:3000
RUST_LOG=info
```

### Frontend (.env.local)
```bash
cd frontend
cat > .env.local << EOF
NEXT_PUBLIC_API_URL=http://localhost:8080/api
EOF
```

## Step 2: Start Services with Docker Compose

From the root directory:
```bash
docker-compose up -d
```

This starts:
- **PostgreSQL** (port 5432) - Database
- **Backend API** (port 8080) - Rust/Actix-web server
- **Worker** - Background job processor
- **Frontend** (port 3000) - Next.js app

Check status:
```bash
docker-compose ps
```

## Step 3: Verify Database Migrations

The backend automatically runs migrations on startup. Check logs:
```bash
docker-compose logs backend | grep -i migrat
```

Expected output:
```
backend | Running migrations...
backend | Applied migration: add_multi_tenancy
```

## Step 4: Access the Application

- **Frontend**: http://localhost:3000
- **API**: http://localhost:8080/api
- **Health Check**: http://localhost:8080/health

## Step 5: Create Your First Account

1. Go to http://localhost:3000/register
2. Sign up with email & password
3. You'll be redirected to /dashboard
4. Your workspace is automatically created

## Step 6: Add Email Account for Sending

1. Go to Dashboard → Email Accounts
2. Click "Add Email Account"
3. Fill in SMTP credentials:
   - Email: your-sending-email@domain.com
   - Provider: Gmail, SendGrid, etc.
   - SMTP Host: smtp.gmail.com (or your provider)
   - SMTP Port: 587
   - Username: your-email@domain.com
   - Password: your-app-password

The password is encrypted before storage.

## Step 7: Create Your First Campaign

1. Go to Dashboard → Campaigns
2. Click "New Campaign"
3. Add leads (manually or via CSV)
4. Configure email template
5. Set daily sending limits
6. Click "Start Campaign"

The worker will begin sending emails respecting:
- Daily limits per inbox
- Inbox warmup schedule
- Suppression list
- Health scores

## Troubleshooting

### Backend won't start
```bash
docker-compose logs backend
```

Common issues:
- Database not ready: Wait 10 seconds and retry
- Port 8080 in use: Change port in docker-compose.yml
- Migration failed: Check DATABASE_URL

### Frontend won't load
```bash
docker-compose logs frontend
```

Check:
- NEXT_PUBLIC_API_URL is set correctly
- Backend is running on port 8080

### Worker not processing jobs
```bash
docker-compose logs worker
```

Check:
- Database connection working
- Jobs table exists (run migrations)
- ENCRYPTION_KEY is set

### Email not sending
1. Check email account credentials in dashboard
2. Verify SMTP settings are correct
3. Check worker logs for errors
4. Verify inbox health score > 0

## Stopping Services

```bash
docker-compose down
```

To also remove volumes (database):
```bash
docker-compose down -v
```

## Development Mode (Without Docker)

### Backend
```bash
cd backend
cargo run
```

### Frontend
```bash
cd frontend
npm install
npm run dev
```

### Worker
```bash
cd backend
cargo run --bin outreachiq-worker
```

## Production Deployment

1. Set strong `JWT_SECRET` (32+ random chars)
2. Set `ENCRYPTION_KEY` (use `openssl rand -base64 32`)
3. Use production database (not localhost)
4. Set `FRONTEND_URL` and `APP_URL` to your domain
5. Enable HTTPS/SSL
6. Configure Stripe keys for billing
7. Set up proper logging & monitoring

## API Documentation

### Authentication
All requests (except /auth and /billing/pricing) require:
```
Authorization: Bearer <jwt_token>
```

### Key Endpoints

**Auth:**
- POST `/api/auth/register` - Create account
- POST `/api/auth/login` - Get JWT token
- GET `/api/auth/me` - Get current user

**Campaigns:**
- GET `/api/campaigns` - List campaigns
- POST `/api/campaigns` - Create campaign
- POST `/api/campaigns/{id}/start` - Start sending
- POST `/api/campaigns/{id}/pause` - Pause sending

**Email Accounts:**
- GET `/api/email-accounts` - List inboxes
- POST `/api/email-accounts` - Add inbox
- POST `/api/email-accounts/{id}/warmup/start` - Start warmup

**Billing:**
- GET `/api/billing/pricing` - Get pricing tiers
- POST `/api/billing/checkout` - Create Stripe checkout
- GET `/api/billing/usage` - Get usage stats

**Compliance:**
- GET `/api/compliance/unsubscribe?token=...` - One-click unsubscribe
- GET `/api/compliance/suppression` - View suppression list
- POST `/api/compliance/suppression` - Add to suppression

## Support

For issues:
1. Check logs: `docker-compose logs <service>`
2. Verify environment variables
3. Ensure ports 3000, 8080, 5432 are available
4. Check database connectivity
