-- ============================================================================
-- FOUNDER DASHBOARD SCHEMA
-- Inbox Health, Auto-Pause, Reply Classification, Cost Tracking
-- ============================================================================

-- Reply intent classification enum
DO $$ BEGIN
    CREATE TYPE reply_intent AS ENUM ('interested', 'maybe_later', 'objection', 'negative', 'auto_reply');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Inbox health status enum
DO $$ BEGIN
    CREATE TYPE inbox_health_status AS ENUM ('healthy', 'warning', 'danger');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Auto-pause reason enum
DO $$ BEGIN
    CREATE TYPE pause_reason AS ENUM ('spam_rate', 'reply_drop', 'bounce_rate', 'manual', 'other');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ============================================================================
-- INBOX HEALTH TRACKING
-- ============================================================================

-- Inbox health metrics (daily snapshots)
CREATE TABLE IF NOT EXISTS inbox_health_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_account_id UUID NOT NULL REFERENCES email_accounts(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL,
    
    -- Core metrics
    spam_rate FLOAT DEFAULT 0.0,           -- % of emails landing in spam
    reply_rate FLOAT DEFAULT 0.0,          -- 7-day rolling average
    bounce_rate FLOAT DEFAULT 0.0,         -- Total bounce rate
    hard_bounce_rate FLOAT DEFAULT 0.0,    -- Permanent failures
    soft_bounce_rate FLOAT DEFAULT 0.0,    -- Temporary failures
    
    -- Calculated health
    health_status VARCHAR(20) DEFAULT 'healthy',  -- healthy, warning, danger
    health_score FLOAT DEFAULT 100.0,      -- 0-100 score
    
    -- Volume metrics
    emails_sent INTEGER DEFAULT 0,
    emails_delivered INTEGER DEFAULT 0,
    emails_opened INTEGER DEFAULT 0,
    emails_replied INTEGER DEFAULT 0,
    
    -- Timestamps
    measured_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inbox_health_account ON inbox_health_metrics(email_account_id);
CREATE INDEX IF NOT EXISTS idx_inbox_health_workspace ON inbox_health_metrics(workspace_id);
CREATE INDEX IF NOT EXISTS idx_inbox_health_measured ON inbox_health_metrics(measured_at DESC);

-- ============================================================================
-- EMAIL REPLIES WITH INTENT CLASSIFICATION
-- ============================================================================

CREATE TABLE IF NOT EXISTS email_replies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    campaign_id UUID REFERENCES campaigns(id) ON DELETE SET NULL,
    lead_id UUID REFERENCES leads(id) ON DELETE SET NULL,
    email_account_id UUID REFERENCES email_accounts(id) ON DELETE SET NULL,
    
    -- Reply content
    from_email VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    subject TEXT,
    body_text TEXT,
    body_html TEXT,
    
    -- AI Classification
    intent VARCHAR(20) DEFAULT 'auto_reply',  -- interested, maybe_later, objection, negative, auto_reply
    intent_confidence FLOAT DEFAULT 0.0,       -- 0-1 confidence score
    classified_at TIMESTAMP WITH TIME ZONE,
    
    -- Status
    is_read BOOLEAN DEFAULT FALSE,
    is_actioned BOOLEAN DEFAULT FALSE,
    action_taken VARCHAR(50),                  -- replied, booked_meeting, snoozed, archived
    action_at TIMESTAMP WITH TIME ZONE,
    
    -- Metadata
    message_id VARCHAR(255),
    in_reply_to VARCHAR(255),
    received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_replies_workspace ON email_replies(workspace_id);
CREATE INDEX IF NOT EXISTS idx_replies_campaign ON email_replies(campaign_id);
CREATE INDEX IF NOT EXISTS idx_replies_intent ON email_replies(intent);
CREATE INDEX IF NOT EXISTS idx_replies_received ON email_replies(received_at DESC);
CREATE INDEX IF NOT EXISTS idx_replies_unread ON email_replies(workspace_id, is_read) WHERE is_read = FALSE;

-- ============================================================================
-- AUTO-PAUSE EVENTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS auto_pause_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    campaign_id UUID REFERENCES campaigns(id) ON DELETE CASCADE,
    email_account_id UUID REFERENCES email_accounts(id) ON DELETE CASCADE,
    
    -- Pause details
    pause_reason VARCHAR(50) NOT NULL,         -- spam_rate, reply_drop, bounce_rate, manual
    pause_reason_detail TEXT,                  -- Human readable: "Spam rate spiked to 5.2%"
    
    -- Metrics at time of pause
    spam_rate_at_pause FLOAT,
    reply_rate_at_pause FLOAT,
    bounce_rate_at_pause FLOAT,
    reply_rate_change FLOAT,                   -- % change that triggered pause
    
    -- Resolution
    is_resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMP WITH TIME ZONE,
    resolved_by UUID,                          -- User who resolved
    resolution_action VARCHAR(50),             -- resumed, fixed, archived
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pause_events_workspace ON auto_pause_events(workspace_id);
CREATE INDEX IF NOT EXISTS idx_pause_events_campaign ON auto_pause_events(campaign_id);
CREATE INDEX IF NOT EXISTS idx_pause_events_unresolved ON auto_pause_events(workspace_id, is_resolved) WHERE is_resolved = FALSE;

-- ============================================================================
-- MEETINGS TRACKING
-- ============================================================================

CREATE TABLE IF NOT EXISTS meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    campaign_id UUID REFERENCES campaigns(id) ON DELETE SET NULL,
    lead_id UUID REFERENCES leads(id) ON DELETE SET NULL,
    reply_id UUID REFERENCES email_replies(id) ON DELETE SET NULL,
    
    -- Meeting details
    title VARCHAR(255),
    scheduled_at TIMESTAMP WITH TIME ZONE,
    duration_minutes INTEGER DEFAULT 30,
    meeting_link TEXT,
    
    -- Status
    status VARCHAR(20) DEFAULT 'scheduled',    -- scheduled, completed, cancelled, no_show
    outcome VARCHAR(50),                       -- qualified, not_qualified, follow_up, closed_won, closed_lost
    notes TEXT,
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_meetings_workspace ON meetings(workspace_id);
CREATE INDEX IF NOT EXISTS idx_meetings_campaign ON meetings(campaign_id);
CREATE INDEX IF NOT EXISTS idx_meetings_scheduled ON meetings(scheduled_at);

-- ============================================================================
-- CAMPAIGN COSTS TRACKING
-- ============================================================================

CREATE TABLE IF NOT EXISTS campaign_costs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    campaign_id UUID REFERENCES campaigns(id) ON DELETE CASCADE,
    
    -- Cost breakdown
    domain_cost DECIMAL(10,2) DEFAULT 0.00,       -- Amortized domain cost
    inbox_cost DECIMAL(10,2) DEFAULT 0.00,        -- ESP/inbox cost
    lead_cost DECIMAL(10,2) DEFAULT 0.00,         -- Lead acquisition cost
    tool_cost DECIMAL(10,2) DEFAULT 0.00,         -- Tool/software cost
    other_cost DECIMAL(10,2) DEFAULT 0.00,        -- Miscellaneous
    
    -- Period
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    
    -- Calculated metrics (updated by background job)
    total_cost DECIMAL(10,2) GENERATED ALWAYS AS (domain_cost + inbox_cost + lead_cost + tool_cost + other_cost) STORED,
    meetings_booked INTEGER DEFAULT 0,
    cost_per_meeting DECIMAL(10,2) DEFAULT 0.00,
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_campaign_costs_workspace ON campaign_costs(workspace_id);
CREATE INDEX IF NOT EXISTS idx_campaign_costs_campaign ON campaign_costs(campaign_id);
CREATE INDEX IF NOT EXISTS idx_campaign_costs_period ON campaign_costs(period_start, period_end);

-- ============================================================================
-- WORKSPACE SETTINGS FOR AUTO-PAUSE THRESHOLDS
-- ============================================================================

CREATE TABLE IF NOT EXISTS workspace_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL UNIQUE,
    
    -- Auto-pause thresholds
    auto_pause_enabled BOOLEAN DEFAULT TRUE,
    spam_rate_threshold FLOAT DEFAULT 0.03,        -- 3% default
    reply_drop_threshold FLOAT DEFAULT 0.40,       -- 40% drop in 48h
    bounce_rate_threshold FLOAT DEFAULT 0.08,      -- 8% default
    
    -- Provider-aware throttling
    google_daily_limit INTEGER DEFAULT 500,
    outlook_daily_limit INTEGER DEFAULT 300,
    zoho_daily_limit INTEGER DEFAULT 200,
    default_daily_limit INTEGER DEFAULT 100,
    
    -- Notifications
    notify_on_pause BOOLEAN DEFAULT TRUE,
    notify_on_warning BOOLEAN DEFAULT TRUE,
    notification_email VARCHAR(255),
    slack_webhook_url TEXT,
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workspace_settings ON workspace_settings(workspace_id);

-- ============================================================================
-- ADD COLUMNS TO EXISTING TABLES
-- ============================================================================

-- Add auto_paused and pause tracking to campaigns
ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS auto_paused BOOLEAN DEFAULT FALSE;
ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS auto_pause_reason TEXT;
ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS paused_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS meetings_booked INTEGER DEFAULT 0;

-- Add provider detection to email_accounts
ALTER TABLE email_accounts ADD COLUMN IF NOT EXISTS detected_provider VARCHAR(50);
ALTER TABLE email_accounts ADD COLUMN IF NOT EXISTS provider_daily_limit INTEGER;
ALTER TABLE email_accounts ADD COLUMN IF NOT EXISTS last_health_check TIMESTAMP WITH TIME ZONE;
ALTER TABLE email_accounts ADD COLUMN IF NOT EXISTS spam_rate FLOAT DEFAULT 0.0;
ALTER TABLE email_accounts ADD COLUMN IF NOT EXISTS reply_rate FLOAT DEFAULT 0.0;
ALTER TABLE email_accounts ADD COLUMN IF NOT EXISTS bounce_rate FLOAT DEFAULT 0.0;

-- Add reply tracking to campaign_leads
ALTER TABLE campaign_leads ADD COLUMN IF NOT EXISTS reply_intent VARCHAR(20);
ALTER TABLE campaign_leads ADD COLUMN IF NOT EXISTS reply_id UUID;

-- ============================================================================
-- VIEWS FOR DASHBOARD
-- ============================================================================

-- Founder dashboard overview view
CREATE OR REPLACE VIEW founder_dashboard_overview AS
SELECT 
    c.workspace_id,
    COUNT(DISTINCT c.id) as total_campaigns,
    COUNT(DISTINCT c.id) FILTER (WHERE c.status = 'active') as active_campaigns,
    COUNT(DISTINCT c.id) FILTER (WHERE c.auto_paused = TRUE) as paused_campaigns,
    COALESCE(SUM(c.sent), 0) as total_sent,
    COALESCE(SUM(c.replied), 0) as total_replies,
    COALESCE(SUM(c.meetings_booked), 0) as total_meetings,
    CASE 
        WHEN SUM(c.sent) > 0 THEN ROUND((SUM(c.replied)::NUMERIC / SUM(c.sent)::NUMERIC) * 100, 2)
        ELSE 0 
    END as overall_reply_rate
FROM campaigns c
GROUP BY c.workspace_id;

-- Inbox health summary view
CREATE OR REPLACE VIEW inbox_health_summary AS
SELECT 
    ea.workspace_id,
    ea.id as email_account_id,
    ea.email,
    ea.provider,
    ea.health_score,
    ea.spam_rate,
    ea.reply_rate,
    ea.bounce_rate,
    ea.daily_limit,
    ea.sent_today,
    CASE 
        WHEN ea.spam_rate > 0.03 OR ea.bounce_rate > 0.08 THEN 'danger'
        WHEN ea.spam_rate > 0.02 OR ea.bounce_rate > 0.05 OR ea.reply_rate < 0.02 THEN 'warning'
        ELSE 'healthy'
    END as health_status
FROM email_accounts ea;

-- Recent replies needing action view
CREATE OR REPLACE VIEW replies_needing_action AS
SELECT 
    er.*,
    c.name as campaign_name,
    l.first_name as lead_first_name,
    l.last_name as lead_last_name,
    l.company as lead_company
FROM email_replies er
LEFT JOIN campaigns c ON er.campaign_id = c.id
LEFT JOIN leads l ON er.lead_id = l.id
WHERE er.is_actioned = FALSE
ORDER BY 
    CASE er.intent 
        WHEN 'interested' THEN 1 
        WHEN 'objection' THEN 2
        WHEN 'maybe_later' THEN 3
        ELSE 4 
    END,
    er.received_at DESC;

-- Cost per meeting summary view
CREATE OR REPLACE VIEW cost_per_meeting_summary AS
SELECT 
    cc.workspace_id,
    cc.campaign_id,
    c.name as campaign_name,
    cc.total_cost,
    cc.meetings_booked,
    CASE 
        WHEN cc.meetings_booked > 0 THEN ROUND(cc.total_cost / cc.meetings_booked, 2)
        ELSE 0 
    END as cost_per_meeting,
    cc.period_start,
    cc.period_end
FROM campaign_costs cc
JOIN campaigns c ON cc.campaign_id = c.id;

-- ============================================================================
-- FUNCTIONS FOR HEALTH CALCULATION
-- ============================================================================

-- Function to calculate inbox health score
CREATE OR REPLACE FUNCTION calculate_inbox_health_score(
    p_spam_rate FLOAT,
    p_bounce_rate FLOAT,
    p_reply_rate FLOAT
) RETURNS FLOAT AS $$
DECLARE
    score FLOAT := 100.0;
BEGIN
    -- Deduct for spam rate (max -40 points)
    score := score - (p_spam_rate * 1000);  -- 1% spam = -10 points
    
    -- Deduct for bounce rate (max -30 points)
    score := score - (p_bounce_rate * 300);  -- 1% bounce = -3 points
    
    -- Bonus for good reply rate (max +10 points)
    IF p_reply_rate > 0.05 THEN
        score := score + 10;
    ELSIF p_reply_rate > 0.03 THEN
        score := score + 5;
    END IF;
    
    -- Clamp between 0 and 100
    RETURN GREATEST(0, LEAST(100, score));
END;
$$ LANGUAGE plpgsql;

-- Function to check if campaign should be auto-paused
CREATE OR REPLACE FUNCTION should_auto_pause(
    p_workspace_id UUID,
    p_spam_rate FLOAT,
    p_reply_rate FLOAT,
    p_previous_reply_rate FLOAT,
    p_bounce_rate FLOAT
) RETURNS TABLE(should_pause BOOLEAN, reason VARCHAR, detail TEXT) AS $$
DECLARE
    settings RECORD;
    reply_drop FLOAT;
BEGIN
    -- Get workspace settings
    SELECT * INTO settings FROM workspace_settings WHERE workspace_id = p_workspace_id;
    
    -- Use defaults if no settings
    IF settings IS NULL THEN
        settings.spam_rate_threshold := 0.03;
        settings.reply_drop_threshold := 0.40;
        settings.bounce_rate_threshold := 0.08;
    END IF;
    
    -- Check spam rate
    IF p_spam_rate > settings.spam_rate_threshold THEN
        RETURN QUERY SELECT TRUE, 'spam_rate'::VARCHAR, 
            format('Spam rate spiked to %.1f%%', p_spam_rate * 100);
        RETURN;
    END IF;
    
    -- Check reply rate drop
    IF p_previous_reply_rate > 0 THEN
        reply_drop := (p_previous_reply_rate - p_reply_rate) / p_previous_reply_rate;
        IF reply_drop > settings.reply_drop_threshold THEN
            RETURN QUERY SELECT TRUE, 'reply_drop'::VARCHAR,
                format('Reply rate dropped %.0f%% in 48 hours', reply_drop * 100);
            RETURN;
        END IF;
    END IF;
    
    -- Check bounce rate
    IF p_bounce_rate > settings.bounce_rate_threshold THEN
        RETURN QUERY SELECT TRUE, 'bounce_rate'::VARCHAR,
            format('Bounce rate reached %.1f%%', p_bounce_rate * 100);
        RETURN;
    END IF;
    
    -- No pause needed
    RETURN QUERY SELECT FALSE, NULL::VARCHAR, NULL::TEXT;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- INSERT DEFAULT WORKSPACE SETTINGS TRIGGER
-- ============================================================================

CREATE OR REPLACE FUNCTION create_default_workspace_settings()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO workspace_settings (workspace_id)
    VALUES (NEW.id)
    ON CONFLICT (workspace_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Note: This trigger should be added to workspaces table if it exists
-- DROP TRIGGER IF EXISTS trigger_create_workspace_settings ON workspaces;
-- CREATE TRIGGER trigger_create_workspace_settings
--     AFTER INSERT ON workspaces
--     FOR EACH ROW
--     EXECUTE FUNCTION create_default_workspace_settings();
