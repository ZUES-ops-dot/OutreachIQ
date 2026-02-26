-- ============================================================================
-- Fix leads unique constraint for multi-tenancy
-- The email column should be unique per workspace, not globally
-- ============================================================================

-- Drop the global unique constraint on email
ALTER TABLE leads DROP CONSTRAINT IF EXISTS leads_email_key;

-- Add a composite unique constraint on (workspace_id, email)
-- This allows the same email to exist in different workspaces
CREATE UNIQUE INDEX IF NOT EXISTS idx_leads_workspace_email 
ON leads(workspace_id, email);

-- Note: This migration assumes workspace_id is already added to leads table
-- from migration 20240102000000_multi_tenancy_security.sql
