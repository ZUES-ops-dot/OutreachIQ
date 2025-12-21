-- ============================================================================
-- CRITICAL FIX #1: Add Multi-Tenancy (Non-Negotiable for SaaS)
-- ============================================================================

-- Create workspaces table
CREATE TABLE workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    plan_tier VARCHAR(50) DEFAULT 'starter', -- starter, professional, business
    monthly_lead_limit INTEGER DEFAULT 1000,
    monthly_email_limit INTEGER DEFAULT 500,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Alter users table to add missing fields (users table already exists from initial migration)
ALTER TABLE users ADD COLUMN IF NOT EXISTS first_name VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_name VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified BOOLEAN DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login TIMESTAMP WITH TIME ZONE;

-- Create workspace_members (RBAC)
CREATE TABLE workspace_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member', -- owner, admin, member, viewer
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(workspace_id, user_id)
);

-- Add workspace_id to ALL existing tables
ALTER TABLE leads ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
ALTER TABLE campaigns ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
ALTER TABLE email_accounts ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
ALTER TABLE jobs ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;

-- Create indices for performance
CREATE INDEX idx_leads_workspace ON leads(workspace_id);
CREATE INDEX idx_campaigns_workspace ON campaigns(workspace_id);
CREATE INDEX idx_email_accounts_workspace ON email_accounts(workspace_id);
CREATE INDEX idx_workspace_members_user ON workspace_members(user_id);
CREATE INDEX idx_workspace_members_workspace ON workspace_members(workspace_id);

-- ============================================================================
-- CRITICAL FIX #2: Secure SMTP Credentials (Encrypt Passwords)
-- ============================================================================

-- Add encrypted_password field, deprecate plaintext
ALTER TABLE email_accounts ADD COLUMN smtp_password_encrypted BYTEA;
ALTER TABLE email_accounts ADD COLUMN encryption_key_id VARCHAR(100);

-- Make provider/smtp fields NOT NULL with defaults first
UPDATE email_accounts SET provider = 'unknown' WHERE provider IS NULL;
UPDATE email_accounts SET smtp_host = 'localhost' WHERE smtp_host IS NULL;
UPDATE email_accounts SET smtp_port = 587 WHERE smtp_port IS NULL;

ALTER TABLE email_accounts ALTER COLUMN provider SET NOT NULL;
ALTER TABLE email_accounts ALTER COLUMN smtp_host SET NOT NULL;
ALTER TABLE email_accounts ALTER COLUMN smtp_port SET NOT NULL;

-- ============================================================================
-- CRITICAL FIX #3: Fix Job Queue Retry Logic
-- ============================================================================

-- Add proper retry tracking
ALTER TABLE jobs ADD COLUMN retry_count INTEGER DEFAULT 0;
ALTER TABLE jobs ADD COLUMN max_retries INTEGER DEFAULT 3;
ALTER TABLE jobs ADD COLUMN next_retry_at TIMESTAMP WITH TIME ZONE;

-- Update job status to VARCHAR for flexibility
ALTER TABLE jobs ALTER COLUMN status TYPE VARCHAR(50);
-- Valid statuses: 'pending', 'processing', 'completed', 'failed', 'scheduled'

-- Create index for efficient job selection
CREATE INDEX idx_jobs_pending ON jobs(status, next_retry_at) WHERE status IN ('pending', 'scheduled');
CREATE INDEX idx_jobs_workspace ON jobs(workspace_id);

-- ============================================================================
-- CRITICAL FIX #4: Add Campaign-Lead Tracking (Proper Analytics)
-- ============================================================================

ALTER TABLE campaign_leads ADD COLUMN open_count INTEGER DEFAULT 0;
ALTER TABLE campaign_leads ADD COLUMN click_count INTEGER DEFAULT 0;
ALTER TABLE campaign_leads ADD COLUMN bounce_type VARCHAR(50); -- hard, soft, complaint
ALTER TABLE campaign_leads ADD COLUMN unsubscribed_at TIMESTAMP WITH TIME ZONE;

-- ============================================================================
-- CRITICAL FIX #5: Add Unsubscribe & Compliance
-- ============================================================================

-- Global suppression list
CREATE TABLE suppression_list (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    reason VARCHAR(50) NOT NULL, -- unsubscribed, bounced, complained, manual
    source VARCHAR(100), -- campaign_id, import, etc
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(workspace_id, email)
);

CREATE INDEX idx_suppression_email ON suppression_list(workspace_id, email);

-- ============================================================================
-- CRITICAL FIX #6: Add Proper Usage Tracking (For Billing)
-- ============================================================================

CREATE TABLE usage_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    metric_type VARCHAR(50) NOT NULL, -- leads_generated, emails_sent, verifications
    count INTEGER NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(workspace_id, metric_type, period_start)
);

CREATE INDEX idx_usage_workspace_period ON usage_metrics(workspace_id, period_start);

-- ============================================================================
-- CRITICAL FIX #7: Add API Rate Limiting Tracking
-- ============================================================================

CREATE TABLE rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    endpoint VARCHAR(255) NOT NULL,
    requests_count INTEGER DEFAULT 0,
    window_start TIMESTAMP WITH TIME ZONE NOT NULL,
    window_end TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_rate_limits_workspace_window ON rate_limits(workspace_id, window_start);

-- ============================================================================
-- Sample Data for Testing
-- ============================================================================

-- Create a test workspace
INSERT INTO workspaces (id, name, slug, plan_tier, monthly_lead_limit, monthly_email_limit)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Test Company',
    'test-company',
    'professional',
    10000,
    5000
);

-- Update existing test user with new fields
UPDATE users 
SET first_name = 'Test', 
    last_name = 'User', 
    email_verified = TRUE 
WHERE email = 'test@outreachiq.io';

-- Link user to workspace as owner (if user exists)
INSERT INTO workspace_members (workspace_id, user_id, role)
SELECT 
    '00000000-0000-0000-0000-000000000001',
    id,
    'owner'
FROM users 
WHERE email = 'test@outreachiq.io'
ON CONFLICT (workspace_id, user_id) DO NOTHING;

-- ============================================================================
-- Migration Rollback (if needed)
-- ============================================================================

/*
-- To rollback these changes (use with caution):

DROP TABLE IF EXISTS rate_limits;
DROP TABLE IF EXISTS usage_metrics;
DROP TABLE IF EXISTS suppression_list;
DROP TABLE IF EXISTS workspace_members;
DROP TABLE IF EXISTS workspaces;

ALTER TABLE leads DROP COLUMN IF EXISTS workspace_id;
ALTER TABLE campaigns DROP COLUMN IF EXISTS workspace_id;
ALTER TABLE email_accounts DROP COLUMN IF EXISTS workspace_id;
ALTER TABLE jobs DROP COLUMN IF EXISTS workspace_id;
ALTER TABLE email_accounts DROP COLUMN IF EXISTS smtp_password_encrypted;
ALTER TABLE email_accounts DROP COLUMN IF EXISTS encryption_key_id;
ALTER TABLE jobs DROP COLUMN IF EXISTS retry_count;
ALTER TABLE jobs DROP COLUMN IF EXISTS max_retries;
ALTER TABLE jobs DROP COLUMN IF EXISTS next_retry_at;
ALTER TABLE campaign_leads DROP COLUMN IF EXISTS open_count;
ALTER TABLE campaign_leads DROP COLUMN IF EXISTS click_count;
ALTER TABLE campaign_leads DROP COLUMN IF EXISTS bounce_type;
ALTER TABLE campaign_leads DROP COLUMN IF EXISTS unsubscribed_at;
ALTER TABLE users DROP COLUMN IF EXISTS first_name;
ALTER TABLE users DROP COLUMN IF EXISTS last_name;
ALTER TABLE users DROP COLUMN IF EXISTS email_verified;
ALTER TABLE users DROP COLUMN IF EXISTS last_login;
*/
