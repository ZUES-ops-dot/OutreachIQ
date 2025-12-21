'use client';

import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { 
  DollarSign, 
  Mail, 
  AlertTriangle, 
  MessageSquare,
  Play,
  Pause,
  Calendar,
  Clock,
  TrendingDown,
  TrendingUp,
  AlertCircle,
  CheckCircle,
  XCircle,
  RefreshCw,
  Plus,
  ChevronDown,
  ChevronUp,
  Upload,
  X,
  HelpCircle
} from 'lucide-react';
import { api, FounderDashboardData, CampaignCard, InboxHealthCard, ReplyCard } from '@/lib/api';

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(value);
}

function formatPercent(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

function formatTimeAgo(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return `${diffDays}d ago`;
}

function getHealthColor(status: string): string {
  switch (status) {
    case 'healthy': return 'text-nord-success bg-nord-success/20';
    case 'warning': return 'text-nord-warning bg-nord-warning/20';
    case 'danger': return 'text-nord-error bg-nord-error/20';
    default: return 'text-nord-text-muted bg-nord-highlight/50';
  }
}

function getHealthIcon(status: string) {
  switch (status) {
    case 'healthy': return <CheckCircle className="w-4 h-4" />;
    case 'warning': return <AlertCircle className="w-4 h-4" />;
    case 'danger': return <XCircle className="w-4 h-4" />;
    default: return <HelpCircle className="w-4 h-4" />;
  }
}

function getIntentColor(intent: string): string {
  switch (intent) {
    case 'interested': return 'border-l-nord-success';
    case 'maybe_later': return 'border-l-nord-warning';
    case 'objection': return 'border-l-nord-frost2';
    case 'negative': return 'border-l-nord-error';
    case 'auto_reply': return 'border-l-nord-highlight';
    default: return 'border-l-nord-highlight';
  }
}

function getIntentEmoji(intent: string): string {
  switch (intent) {
    case 'interested': return '🟢';
    case 'maybe_later': return '🟡';
    case 'objection': return '🔵';
    case 'negative': return '🔴';
    case 'auto_reply': return '⚪';
    default: return '⚪';
  }
}

function getIntentLabel(intent: string): string {
  switch (intent) {
    case 'interested': return 'Interested';
    case 'maybe_later': return 'Maybe Later';
    case 'objection': return 'Question';
    case 'negative': return 'Negative';
    case 'auto_reply': return 'Auto-reply';
    default: return intent;
  }
}

// Demo data for when API is not available (not authenticated or backend down)
function getDemoData(): FounderDashboardData {
  return {
    overview: {
      total_campaigns: 3,
      active_campaigns: 2,
      paused_campaigns: 1,
      total_sent: 1247,
      total_replies: 89,
      total_meetings: 12,
      cost_per_meeting: 127,
      cost_per_meeting_trend: -15,
    },
    campaigns: [
      {
        id: 'demo-1',
        name: 'Q1 SaaS Founders',
        status: 'active',
        health_status: 'healthy',
        auto_paused: false,
        auto_pause_reason: null,
        total_leads: 500,
        sent: 423,
        replied: 34,
        meetings_booked: 8,
        reply_rate: 0.08,
        created_at: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
      },
      {
        id: 'demo-2',
        name: 'Web3 Protocol Teams',
        status: 'active',
        health_status: 'warning',
        auto_paused: false,
        auto_pause_reason: null,
        total_leads: 350,
        sent: 289,
        replied: 18,
        meetings_booked: 3,
        reply_rate: 0.062,
        created_at: new Date(Date.now() - 14 * 24 * 60 * 60 * 1000).toISOString(),
      },
      {
        id: 'demo-3',
        name: 'Agency Outreach',
        status: 'paused',
        health_status: 'danger',
        auto_paused: true,
        auto_pause_reason: 'High spam rate detected (4.2%)',
        total_leads: 200,
        sent: 156,
        replied: 4,
        meetings_booked: 1,
        reply_rate: 0.026,
        created_at: new Date(Date.now() - 21 * 24 * 60 * 60 * 1000).toISOString(),
      },
    ],
    inboxes: [
      {
        id: 'inbox-1',
        email: 'alex@outreach.io',
        provider: 'google',
        health_status: 'healthy',
        health_score: 0.95,
        spam_rate: 0.01,
        reply_rate: 0.08,
        bounce_rate: 0.02,
        daily_limit: 50,
        sent_today: 32,
      },
      {
        id: 'inbox-2',
        email: 'sales@company.com',
        provider: 'outlook',
        health_status: 'warning',
        health_score: 0.72,
        spam_rate: 0.025,
        reply_rate: 0.05,
        bounce_rate: 0.04,
        daily_limit: 40,
        sent_today: 38,
      },
      {
        id: 'inbox-3',
        email: 'reach@startup.io',
        provider: 'google',
        health_status: 'danger',
        health_score: 0.45,
        spam_rate: 0.042,
        reply_rate: 0.02,
        bounce_rate: 0.09,
        daily_limit: 30,
        sent_today: 12,
      },
    ],
    recent_replies: [
      {
        id: 'reply-1',
        from_email: 'john@techstartup.com',
        from_name: 'John Smith',
        subject: 'Re: Quick question about your product',
        body_preview: 'Hey, thanks for reaching out! I\'d love to learn more about what you\'re building. Can we schedule a call next week?',
        intent: 'interested',
        intent_confidence: 0.92,
        campaign_id: 'demo-1',
        campaign_name: 'Q1 SaaS Founders',
        received_at: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
        is_read: false,
        is_actioned: false,
      },
      {
        id: 'reply-2',
        from_email: 'sarah@web3protocol.io',
        from_name: 'Sarah Chen',
        subject: 'Re: Partnership opportunity',
        body_preview: 'Interesting timing - we\'re actually looking at solutions like this. What\'s your pricing model?',
        intent: 'objection',
        intent_confidence: 0.78,
        campaign_id: 'demo-2',
        campaign_name: 'Web3 Protocol Teams',
        received_at: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
        is_read: true,
        is_actioned: false,
      },
      {
        id: 'reply-3',
        from_email: 'mike@agency.co',
        from_name: 'Mike Johnson',
        subject: 'Re: Collaboration idea',
        body_preview: 'Not the right time for us, but reach back out in Q2 when we have more bandwidth.',
        intent: 'maybe_later',
        intent_confidence: 0.85,
        campaign_id: 'demo-3',
        campaign_name: 'Agency Outreach',
        received_at: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
        is_read: true,
        is_actioned: false,
      },
    ],
    unread_count: 5,
    action_required_count: 2,
  };
}

// ============================================================================
// NEW CAMPAIGN MODAL
// ============================================================================

interface NewCampaignModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (name: string, file: File | null) => void;
}

function NewCampaignModal({ isOpen, onClose, onSubmit }: NewCampaignModalProps) {
  const [name, setName] = useState('');
  const [file, setFile] = useState<File | null>(null);
  const [dragOver, setDragOver] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim()) {
      onSubmit(name, file);
      setName('');
      setFile(null);
      onClose();
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    const droppedFile = e.dataTransfer.files[0];
    if (droppedFile?.name.endsWith('.csv')) {
      setFile(droppedFile);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} />
      <div className="relative bg-nord-surface border border-nord-highlight rounded-xl p-6 w-full max-w-md shadow-lg">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-nord-text-muted hover:text-nord-text transition-colors"
        >
          <X size={20} />
        </button>
        
        <h2 className="text-xl font-semibold text-nord-text mb-6">New Campaign</h2>
        
        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <label className="block text-sm font-medium text-nord-text-secondary mb-2">
              Campaign Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Q1 SaaS Founders"
              className="w-full px-4 py-3 bg-nord-bg border border-nord-highlight rounded-lg text-nord-text placeholder-nord-text-muted focus:outline-none focus:ring-2 focus:ring-nord-frost3 focus:border-transparent"
            />
          </div>
          
          <div className="mb-6">
            <label className="block text-sm font-medium text-nord-text-secondary mb-2">
              Upload Leads (CSV)
            </label>
            <div
              onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
              onDragLeave={() => setDragOver(false)}
              onDrop={handleDrop}
              className={`border-2 border-dashed rounded-lg p-6 text-center transition-colors ${
                dragOver ? 'border-nord-frost3 bg-nord-frost3/10' : 'border-nord-highlight hover:border-nord-text-muted'
              }`}
            >
              {file ? (
                <div className="flex items-center justify-center gap-2 text-nord-success">
                  <CheckCircle size={20} />
                  <span>{file.name}</span>
                  <button
                    type="button"
                    onClick={() => setFile(null)}
                    className="text-nord-text-muted hover:text-nord-text ml-2"
                  >
                    <X size={16} />
                  </button>
                </div>
              ) : (
                <>
                  <Upload className="w-8 h-8 text-nord-text-muted mx-auto mb-2" />
                  <p className="text-nord-text-muted text-sm">
                    Drag & drop CSV or{' '}
                    <label className="text-nord-frost3 cursor-pointer hover:text-nord-frost2">
                      browse
                      <input
                        type="file"
                        accept=".csv"
                        className="hidden"
                        onChange={(e) => setFile(e.target.files?.[0] || null)}
                      />
                    </label>
                  </p>
                </>
              )}
            </div>
          </div>
          
          <button
            type="submit"
            disabled={!name.trim()}
            className="w-full py-3 bg-nord-frost3 text-nord-bg font-semibold rounded-lg hover:bg-nord-frost2 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Create Campaign
          </button>
        </form>
      </div>
    </div>
  );
}

// ============================================================================
// OVERVIEW CARDS COMPONENT (DARK THEME)
// ============================================================================

interface OverviewCardsProps {
  data: FounderDashboardData;
}

function OverviewCards({ data }: OverviewCardsProps) {
  const { overview, inboxes } = data;
  
  const healthyCounts = {
    healthy: inboxes.filter(i => i.health_status === 'healthy').length,
    warning: inboxes.filter(i => i.health_status === 'warning').length,
    danger: inboxes.filter(i => i.health_status === 'danger').length,
  };

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
      {/* Cost Per Meeting */}
      <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-5 hover:border-nord-highlight transition-all">
        <div className="flex items-center justify-between mb-3">
          <div className="w-10 h-10 rounded-lg bg-nord-success/20 flex items-center justify-center">
            <DollarSign className="w-5 h-5 text-nord-success" />
          </div>
          {overview.cost_per_meeting_trend !== 0 && (
            <div className={`flex items-center gap-1 text-sm ${overview.cost_per_meeting_trend < 0 ? 'text-nord-success' : 'text-nord-error'}`}>
              {overview.cost_per_meeting_trend < 0 ? <TrendingDown size={14} /> : <TrendingUp size={14} />}
              <span>{Math.abs(overview.cost_per_meeting_trend).toFixed(0)}%</span>
            </div>
          )}
        </div>
        <div className="text-3xl font-bold text-nord-text">{formatCurrency(overview.cost_per_meeting)}</div>
        <div className="text-sm text-nord-text-muted">per meeting</div>
      </div>

      {/* Inboxes */}
      <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-5 hover:border-nord-highlight transition-all">
        <div className="flex items-center justify-between mb-3">
          <div className="w-10 h-10 rounded-lg bg-nord-frost3/20 flex items-center justify-center">
            <Mail className="w-5 h-5 text-nord-frost3" />
          </div>
        </div>
        <div className="text-3xl font-bold text-nord-text">{inboxes.length}</div>
        <div className="text-sm text-nord-text-muted mb-2">inboxes</div>
        <div className="flex items-center gap-2">
          {healthyCounts.healthy > 0 && (
            <span className="text-xs px-2 py-1 rounded-md bg-nord-success/20 text-nord-success">
              {healthyCounts.healthy} 🟢
            </span>
          )}
          {healthyCounts.warning > 0 && (
            <span className="text-xs px-2 py-1 rounded-md bg-nord-warning/20 text-nord-warning">
              {healthyCounts.warning} 🟡
            </span>
          )}
          {healthyCounts.danger > 0 && (
            <span className="text-xs px-2 py-1 rounded-md bg-nord-error/20 text-nord-error">
              {healthyCounts.danger} 🔴
            </span>
          )}
        </div>
      </div>

      {/* Paused Campaigns */}
      <div className={`border rounded-xl p-5 transition-all ${
        overview.paused_campaigns > 0 
          ? 'bg-nord-error/10 border-nord-error/30 hover:border-nord-error/50' 
          : 'bg-nord-surface border-nord-elevated/50 hover:border-nord-highlight'
      }`}>
        <div className="flex items-center justify-between mb-3">
          <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${
            overview.paused_campaigns > 0 ? 'bg-nord-error/20' : 'bg-nord-highlight/50'
          }`}>
            <AlertTriangle className={`w-5 h-5 ${overview.paused_campaigns > 0 ? 'text-nord-error' : 'text-nord-text-muted'}`} />
          </div>
        </div>
        <div className="text-3xl font-bold text-nord-text">{overview.paused_campaigns}</div>
        <div className="text-sm text-nord-text-muted">paused</div>
        {overview.paused_campaigns > 0 && (
          <div className="text-xs text-nord-error mt-1">⚠️ Needs attention</div>
        )}
      </div>

      {/* Replies */}
      <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-5 hover:border-nord-highlight transition-all">
        <div className="flex items-center justify-between mb-3">
          <div className="w-10 h-10 rounded-lg bg-nord-purple/20 flex items-center justify-center">
            <MessageSquare className="w-5 h-5 text-nord-purple" />
          </div>
        </div>
        <div className="text-3xl font-bold text-nord-text">{overview.total_replies}</div>
        <div className="text-sm text-nord-text-muted">replies</div>
        {data.action_required_count > 0 && (
          <div className="text-xs text-nord-success mt-1">{data.action_required_count} need action</div>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// CAMPAIGN CARD COMPONENT (DARK THEME)
// ============================================================================

interface CampaignCardItemProps {
  campaign: CampaignCard;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  isLoading: boolean;
}

function CampaignCardItem({ campaign, onPause, onResume, isLoading }: CampaignCardItemProps) {
  const [expanded, setExpanded] = useState(false);
  const replyRate = campaign.sent > 0 ? (campaign.replied / campaign.sent) * 100 : 0;
  
  return (
    <div className={`border rounded-xl p-5 mb-4 transition-all ${
      campaign.auto_paused 
        ? 'bg-nord-error/10 border-nord-error/30' 
        : 'bg-nord-surface border-nord-elevated/50 hover:border-nord-highlight'
    }`}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-3 mb-2">
            <h3 className="font-semibold text-nord-text text-lg">{campaign.name}</h3>
            <span className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium ${getHealthColor(campaign.health_status)}`}>
              {getHealthIcon(campaign.health_status)}
              <span className="capitalize">{campaign.health_status}</span>
            </span>
          </div>
          
          {campaign.auto_paused && campaign.auto_pause_reason && (
            <div className="flex items-center gap-2 text-sm text-nord-error bg-nord-error/20 px-4 py-2 rounded-lg mb-3">
              <AlertTriangle size={16} />
              <span className="font-semibold">AUTO-PAUSED:</span>
              <span>{campaign.auto_pause_reason}</span>
            </div>
          )}
          
          <div className="flex items-center gap-4 text-sm text-nord-text-muted">
            <span className="text-nord-text font-medium">{campaign.sent.toLocaleString()}</span>
            <span>sent</span>
            <span className="text-nord-highlight">•</span>
            <span className="text-nord-text font-medium">{campaign.replied}</span>
            <span>replies</span>
            <span className="text-nord-highlight">•</span>
            <span className="text-nord-text font-medium">{campaign.meetings_booked}</span>
            <span>meetings</span>
            <span className="text-nord-highlight">•</span>
            <span className={`font-medium ${replyRate > 5 ? 'text-nord-success' : replyRate > 2 ? 'text-nord-warning' : 'text-nord-text-muted'}`}>
              {replyRate.toFixed(1)}%
            </span>
            <span>reply rate</span>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          {campaign.auto_paused || campaign.status === 'paused' ? (
            <button
              onClick={() => onResume(campaign.id)}
              disabled={isLoading}
              className="flex items-center gap-1.5 px-4 py-2 bg-nord-success text-nord-bg rounded-lg hover:opacity-90 transition-all text-sm font-medium disabled:opacity-50"
            >
              <Play size={14} />
              Resume
            </button>
          ) : campaign.status === 'active' ? (
            <button
              onClick={() => onPause(campaign.id)}
              disabled={isLoading}
              className="flex items-center gap-1.5 px-4 py-2 bg-nord-elevated text-nord-text-secondary rounded-lg hover:bg-nord-highlight transition-colors text-sm font-medium disabled:opacity-50"
            >
              <Pause size={14} />
              Pause
            </button>
          ) : null}
          
          <button
            onClick={() => setExpanded(!expanded)}
            className="p-2 text-nord-text-muted hover:text-nord-text transition-colors rounded-lg hover:bg-nord-elevated"
          >
            {expanded ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
          </button>
        </div>
      </div>
      
      {expanded && (
        <div className="mt-4 pt-4 border-t border-nord-elevated">
          <div className="grid grid-cols-4 gap-4 text-sm">
            <div>
              <div className="text-nord-text-muted mb-1">Total Leads</div>
              <div className="font-semibold text-nord-text">{campaign.total_leads.toLocaleString()}</div>
            </div>
            <div>
              <div className="text-nord-text-muted mb-1">Sent</div>
              <div className="font-semibold text-nord-text">{campaign.sent.toLocaleString()}</div>
            </div>
            <div>
              <div className="text-nord-text-muted mb-1">Replies</div>
              <div className="font-semibold text-nord-text">{campaign.replied}</div>
            </div>
            <div>
              <div className="text-nord-text-muted mb-1">Meetings</div>
              <div className="font-semibold text-nord-text">{campaign.meetings_booked}</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ============================================================================
// INBOX HEALTH CARD COMPONENT (DARK THEME)
// ============================================================================

interface InboxHealthCardItemProps {
  inbox: InboxHealthCard;
}

function InboxHealthCardItem({ inbox }: InboxHealthCardItemProps) {
  const statusColors = {
    healthy: 'border-nord-success/30 bg-nord-success/10',
    warning: 'border-nord-warning/30 bg-nord-warning/10',
    danger: 'border-nord-error/30 bg-nord-error/10',
  };
  
  return (
    <div className={`rounded-xl border p-4 transition-all hover:scale-[1.01] ${statusColors[inbox.health_status] || 'border-nord-elevated/50 bg-nord-surface'}`}>
      <div className="flex items-center justify-between mb-3">
        <div className="font-medium text-nord-text truncate">{inbox.email.split('@')[0]}@</div>
        <span className={`flex items-center gap-1.5 px-2 py-1 rounded-md text-xs font-medium ${getHealthColor(inbox.health_status)}`}>
          {getHealthIcon(inbox.health_status)}
          <span className="capitalize">{inbox.health_status}</span>
        </span>
      </div>
      
      <div className="text-sm mb-3">
        {inbox.health_status === 'danger' && inbox.spam_rate > 0.03 && (
          <span className="text-nord-error">{formatPercent(inbox.spam_rate)} spam rate</span>
        )}
        {inbox.health_status === 'danger' && inbox.bounce_rate > 0.08 && (
          <span className="text-nord-error">{formatPercent(inbox.bounce_rate)} bounce rate</span>
        )}
        {inbox.health_status === 'warning' && (
          <span className="text-nord-warning">{formatPercent(inbox.reply_rate)} reply rate</span>
        )}
        {inbox.health_status === 'healthy' && (
          <span className="text-nord-success">{formatPercent(inbox.reply_rate)} reply rate</span>
        )}
      </div>
      
      {/* Progress bar */}
      <div className="mb-3">
        <div className="h-1.5 bg-nord-elevated rounded-full overflow-hidden">
          <div 
            className={`h-full rounded-full transition-all ${
              inbox.health_status === 'danger' ? 'bg-nord-error' :
              inbox.health_status === 'warning' ? 'bg-nord-warning' : 'bg-nord-success'
            }`}
            style={{ width: `${Math.min((inbox.sent_today / inbox.daily_limit) * 100, 100)}%` }}
          />
        </div>
      </div>
      
      <div className="flex items-center justify-between text-xs text-nord-text-muted">
        <span>{inbox.sent_today}/{inbox.daily_limit} today</span>
        <span className="capitalize">{inbox.provider}</span>
      </div>
      
      {inbox.health_status === 'danger' && (
        <button className="w-full mt-3 px-3 py-2 bg-nord-error text-nord-text rounded-lg text-sm font-medium hover:opacity-90 transition-all">
          Fix Now
        </button>
      )}
    </div>
  );
}

// ============================================================================
// REPLY CARD COMPONENT (DARK THEME)
// ============================================================================

interface ReplyCardItemProps {
  reply: ReplyCard;
  onAction: (id: string, action: string) => void;
  isLoading: boolean;
}

function ReplyCardItem({ reply, onAction, isLoading }: ReplyCardItemProps) {
  return (
    <div className={`bg-nord-surface rounded-xl border-l-4 border border-nord-elevated/50 ${getIntentColor(reply.intent)} p-4 mb-3 hover:border-nord-highlight transition-all`}>
      <div className="flex items-start gap-3">
        <div className="text-xl">{getIntentEmoji(reply.intent)}</div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="font-medium text-nord-text truncate">
              {reply.from_name || reply.from_email}
            </span>
            <span className="text-xs text-nord-text-muted">{formatTimeAgo(reply.received_at)}</span>
          </div>
          
          {reply.subject && (
            <div className="text-sm text-nord-text-secondary mb-1 truncate">{reply.subject}</div>
          )}
          
          <div className="text-sm text-nord-text-muted line-clamp-2">{reply.body_preview}</div>
          
          {reply.campaign_name && (
            <div className="text-xs text-nord-text-muted mt-2">
              Campaign: {reply.campaign_name}
            </div>
          )}
        </div>
      </div>
      
      <div className="flex items-center gap-2 mt-3 pt-3 border-t border-nord-elevated">
        {reply.intent === 'interested' && (
          <>
            <button
              onClick={() => onAction(reply.id, 'replied')}
              disabled={isLoading}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-nord-elevated text-nord-text-secondary rounded-lg hover:bg-nord-highlight transition-colors text-sm disabled:opacity-50"
            >
              <MessageSquare size={14} />
              Reply
            </button>
            <button
              onClick={() => onAction(reply.id, 'booked_meeting')}
              disabled={isLoading}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 transition-all text-sm font-medium disabled:opacity-50"
            >
              <Calendar size={14} />
              Book Meeting
            </button>
          </>
        )}
        
        {reply.intent === 'maybe_later' && (
          <button
            onClick={() => onAction(reply.id, 'snoozed')}
            disabled={isLoading}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-nord-warning text-nord-bg rounded-lg hover:opacity-90 transition-all text-sm disabled:opacity-50"
          >
            <Clock size={14} />
            Set Reminder
          </button>
        )}
        
        {reply.intent === 'objection' && (
          <button
            onClick={() => onAction(reply.id, 'replied')}
            disabled={isLoading}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-nord-frost2 text-nord-bg rounded-lg hover:opacity-90 transition-all text-sm disabled:opacity-50"
          >
            <MessageSquare size={14} />
            Answer Question
          </button>
        )}
        
        <button
          onClick={() => onAction(reply.id, 'archived')}
          disabled={isLoading}
          className="flex items-center gap-1.5 px-3 py-1.5 text-nord-text-muted hover:text-nord-text transition-colors text-sm ml-auto disabled:opacity-50"
        >
          Archive
        </button>
      </div>
    </div>
  );
}

// ============================================================================
// MAIN DASHBOARD COMPONENT (DARK THEME - NO SIDEBAR)
// ============================================================================

export default function FounderDashboardPage() {
  const [data, setData] = useState<FounderDashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date());
  const [showNewCampaignModal, setShowNewCampaignModal] = useState(false);

  const loadDashboard = useCallback(async () => {
    try {
      setError(null);
      const dashboardData = await api.getFounderDashboard();
      setData(dashboardData);
      setLastRefresh(new Date());
    } catch (err) {
      console.error('Failed to load dashboard:', err);
      // Use demo data when API fails (not authenticated or backend down)
      setData(getDemoData());
      setLastRefresh(new Date());
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadDashboard();
    
    // Auto-refresh every 30 seconds
    const interval = setInterval(loadDashboard, 30000);
    return () => clearInterval(interval);
  }, [loadDashboard]);

  const handlePauseCampaign = async (campaignId: string) => {
    setActionLoading(true);
    try {
      await api.pauseFounderCampaign(campaignId);
      await loadDashboard();
    } catch (err) {
      console.error('Failed to pause campaign:', err);
    } finally {
      setActionLoading(false);
    }
  };

  const handleResumeCampaign = async (campaignId: string) => {
    setActionLoading(true);
    try {
      await api.resumeFounderCampaign(campaignId);
      await loadDashboard();
    } catch (err) {
      console.error('Failed to resume campaign:', err);
    } finally {
      setActionLoading(false);
    }
  };

  const handleReplyAction = async (replyId: string, action: string) => {
    setActionLoading(true);
    try {
      await api.actionReply(replyId, action);
      
      // If booking a meeting, create the meeting record
      if (action === 'booked_meeting') {
        const reply = data?.recent_replies.find(r => r.id === replyId);
        if (reply) {
          await api.createMeeting({
            campaign_id: reply.campaign_id || undefined,
            title: `Meeting with ${reply.from_name || reply.from_email}`,
            reply_id: replyId,
          });
        }
      }
      
      await loadDashboard();
    } catch (err) {
      console.error('Failed to action reply:', err);
    } finally {
      setActionLoading(false);
    }
  };

  const handleCreateCampaign = async (name: string, file: File | null) => {
    try {
      await api.createCampaign({ name, vertical: 'general' });
      await loadDashboard();
    } catch (err) {
      console.error('Failed to create campaign:', err);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-nord-bg flex items-center justify-center">
        <div className="flex items-center gap-3 text-nord-text-muted">
          <RefreshCw className="w-6 h-6 animate-spin text-nord-frost3" />
          <span className="text-lg">Loading dashboard...</span>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="min-h-screen bg-nord-bg flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 rounded-xl bg-nord-error/20 flex items-center justify-center mx-auto mb-4">
            <AlertCircle className="w-8 h-8 text-nord-error" />
          </div>
          <h2 className="text-xl font-semibold text-nord-text mb-2">Failed to load dashboard</h2>
          <p className="text-nord-text-muted mb-6">{error}</p>
          <button
            onClick={loadDashboard}
            className="px-6 py-3 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 transition-all font-medium"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-nord-bg min-h-[calc(100vh-4rem)]">
      {/* New Campaign Modal */}
      <NewCampaignModal
        isOpen={showNewCampaignModal}
        onClose={() => setShowNewCampaignModal(false)}
        onSubmit={handleCreateCampaign}
      />

      {/* Page Header */}
      <div className="border-b border-nord-elevated/50 bg-nord-surface/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-xl font-bold text-nord-text">Dashboard</h1>
              <p className="text-sm text-nord-text-muted">
                Updated {lastRefresh.toLocaleTimeString()}
              </p>
            </div>
            
            <div className="flex items-center gap-3">
              <button
                onClick={loadDashboard}
                className="p-2 text-nord-text-muted hover:text-nord-text transition-colors rounded-lg hover:bg-nord-elevated"
              >
                <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
              </button>
              <button 
                onClick={() => setShowNewCampaignModal(true)}
                className="flex items-center gap-2 px-4 py-2 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 transition-all font-medium text-sm"
              >
                <Plus size={16} />
                New Campaign
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-6">
        {/* Overview Cards */}
        <OverviewCards data={data} />

        {/* Main Content Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left Column - Campaigns & Replies */}
          <div className="lg:col-span-2 space-y-8">
            {/* Active Campaigns */}
            <div>
              <h2 className="text-lg font-semibold text-nord-text mb-4 flex items-center gap-2">
                <Mail className="w-5 h-5 text-nord-frost3" />
                Campaigns
              </h2>
              {data.campaigns.length === 0 ? (
                <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-8 text-center">
                  <div className="w-12 h-12 rounded-lg bg-nord-elevated flex items-center justify-center mx-auto mb-3">
                    <Mail className="w-6 h-6 text-nord-text-muted" />
                  </div>
                  <p className="text-nord-text-muted mb-4">No campaigns yet</p>
                  <button 
                    onClick={() => setShowNewCampaignModal(true)}
                    className="px-4 py-2 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 transition-colors text-sm font-medium"
                  >
                    Create Your First Campaign
                  </button>
                </div>
              ) : (
                data.campaigns.map((campaign) => (
                  <CampaignCardItem
                    key={campaign.id}
                    campaign={campaign}
                    onPause={handlePauseCampaign}
                    onResume={handleResumeCampaign}
                    isLoading={actionLoading}
                  />
                ))
              )}
            </div>

            {/* Recent Replies */}
            <div>
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-lg font-semibold text-nord-text flex items-center gap-2">
                  <MessageSquare className="w-5 h-5 text-nord-frost3" />
                  Recent Replies
                  {data.action_required_count > 0 && (
                    <span className="ml-2 px-2.5 py-1 bg-nord-success/20 text-nord-success rounded-md text-xs font-medium">
                      {data.action_required_count} need action
                    </span>
                  )}
                </h2>
              </div>
              
              {data.recent_replies.length === 0 ? (
                <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-8 text-center">
                  <div className="w-12 h-12 rounded-lg bg-nord-elevated flex items-center justify-center mx-auto mb-3">
                    <MessageSquare className="w-6 h-6 text-nord-text-muted" />
                  </div>
                  <p className="text-nord-text-muted">No replies yet. They'll appear here when leads respond.</p>
                </div>
              ) : (
                data.recent_replies.slice(0, 5).map((reply) => (
                  <ReplyCardItem
                    key={reply.id}
                    reply={reply}
                    onAction={handleReplyAction}
                    isLoading={actionLoading}
                  />
                ))
              )}
            </div>
          </div>

          {/* Right Column - Inbox Health */}
          <div className="space-y-6">
            <div>
              <h2 className="text-lg font-semibold text-nord-text mb-4 flex items-center gap-2">
                <Mail className="w-5 h-5 text-nord-frost3" />
                Inbox Health
              </h2>
              <div className="space-y-3">
                {data.inboxes.length === 0 ? (
                  <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-8 text-center">
                    <div className="w-12 h-12 rounded-lg bg-nord-elevated flex items-center justify-center mx-auto mb-3">
                      <Mail className="w-6 h-6 text-nord-text-muted" />
                    </div>
                    <p className="text-nord-text-muted">No inboxes configured yet.</p>
                  </div>
                ) : (
                  data.inboxes.map((inbox) => (
                    <InboxHealthCardItem key={inbox.id} inbox={inbox} />
                  ))
                )}
              </div>
            </div>

            {/* Auto-Pause Settings Card */}
            <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-5">
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-medium text-nord-text flex items-center gap-2">
                  <AlertTriangle className="w-4 h-4 text-nord-warning" />
                  Auto-Pause Rules
                </h3>
                <span className="text-xs px-2 py-1 bg-nord-success/20 text-nord-success rounded-md">Active</span>
              </div>
              <div className="space-y-3 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-nord-text-muted">Spam rate</span>
                  <span className="text-nord-text font-medium">&gt; 3%</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-nord-text-muted">Reply drop (48h)</span>
                  <span className="text-nord-text font-medium">&gt; 40%</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-nord-text-muted">Bounce rate</span>
                  <span className="text-nord-text font-medium">&gt; 8%</span>
                </div>
              </div>
              <Link
                href="/dashboard/settings"
                className="block mt-4 text-center text-sm text-nord-frost3 hover:text-nord-frost2 transition-colors"
              >
                Edit Settings →
              </Link>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
