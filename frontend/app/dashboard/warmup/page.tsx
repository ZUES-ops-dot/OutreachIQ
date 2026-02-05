'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { Mail, TrendingUp, Shield, AlertTriangle, CheckCircle, Activity, Plus, Settings, Play, Pause, RefreshCw, Link2, Copy, ExternalLink } from 'lucide-react';
import { api, EmailAccount, Campaign } from '@/lib/api';

interface InboxIssue {
  type: 'critical' | 'warning';
  message: string;
}

interface InboxMetrics {
  inboxRate: number;
  spamRate: number;
  bounceRate: number;
  openRate: number;
}

interface Inbox {
  id: string;
  email: string;
  provider: string;
  status: 'active' | 'warming' | 'warning' | 'paused';
  healthScore: number;
  daysActive: number;
  currentVolume: number;
  targetVolume: number;
  warmupProgress: number;
  metrics: InboxMetrics;
  issues: InboxIssue[];
  linkedCampaigns?: string[]; // Campaign IDs this inbox is assigned to
}

export default function WarmupPage() {
  const router = useRouter();
  const [selectedInbox, setSelectedInbox] = useState<string | null>(null);
  const [inboxes, setInboxes] = useState<Inbox[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [newAccount, setNewAccount] = useState({
    email: '',
    provider: 'google',
    smtp_host: '',
    smtp_port: 587,
    smtp_username: '',
    smtp_password: ''
  });
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [showLinkModal, setShowLinkModal] = useState(false);
  const [linkingInboxId, setLinkingInboxId] = useState<string | null>(null);
  const [selectedCampaignIds, setSelectedCampaignIds] = useState<string[]>([]);
  const [copiedEmail, setCopiedEmail] = useState<string | null>(null);

  useEffect(() => {
    loadEmailAccounts();
    loadCampaigns();
  }, []);

  const loadCampaigns = async () => {
    try {
      const campaignList = await api.getCampaigns();
      setCampaigns(campaignList);
    } catch (error) {
      console.error('Failed to load campaigns:', error);
    }
  };

  const handleCopyEmail = async (email: string) => {
    try {
      await navigator.clipboard.writeText(email);
      setCopiedEmail(email);
      setTimeout(() => setCopiedEmail(null), 2000);
    } catch (error) {
      console.error('Failed to copy email:', error);
    }
  };

  const handleOpenLinkModal = (inboxId: string) => {
    setLinkingInboxId(inboxId);
    // Pre-select currently linked campaigns if any
    const inbox = inboxes.find(i => i.id === inboxId);
    setSelectedCampaignIds(inbox?.linkedCampaigns || []);
    setShowLinkModal(true);
  };

  const handleLinkCampaigns = async () => {
    if (!linkingInboxId) return;
    // In production, this would call an API to save the inbox-campaign assignments
    // For now, update local state
    setInboxes(prev => prev.map(inbox => 
      inbox.id === linkingInboxId 
        ? { ...inbox, linkedCampaigns: selectedCampaignIds }
        : inbox
    ));
    setShowLinkModal(false);
    setLinkingInboxId(null);
    setSelectedCampaignIds([]);
  };

  const toggleCampaignSelection = (campaignId: string) => {
    setSelectedCampaignIds(prev => 
      prev.includes(campaignId)
        ? prev.filter(id => id !== campaignId)
        : [...prev, campaignId]
    );
  };

  const loadEmailAccounts = async () => {
    setLoading(true);
    try {
      const accounts = await api.getEmailAccounts();
      const transformedInboxes: Inbox[] = accounts.map(account => ({
        id: account.id,
        email: account.email,
        provider: account.provider,
        status: account.warmup_status === 'active' ? 'warming' : 
                account.warmup_status === 'completed' ? 'active' : 
                account.warmup_status === 'paused' ? 'paused' : 'warning',
        healthScore: account.health_score,
        daysActive: Math.floor((Date.now() - new Date(account.created_at).getTime()) / (1000 * 60 * 60 * 24)),
        currentVolume: account.sent_today,
        targetVolume: account.daily_limit,
        warmupProgress: Math.min(100, Math.round((account.sent_today / account.daily_limit) * 100)),
        metrics: {
          inboxRate: 95,
          spamRate: 2,
          bounceRate: 3,
          openRate: 65
        },
        issues: account.health_score < 80 ? [{ type: 'warning' as const, message: 'Health score below threshold' }] : []
      }));
      setInboxes(transformedInboxes);
      if (transformedInboxes.length > 0 && !selectedInbox) {
        setSelectedInbox(transformedInboxes[0].id);
      }
    } catch (error) {
      console.error('Failed to load email accounts:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleStartWarmup = async (accountId: string) => {
    try {
      await api.startWarmup(accountId);
      loadEmailAccounts();
    } catch (error) {
      console.error('Failed to start warmup:', error);
    }
  };

  const handlePauseWarmup = async (accountId: string) => {
    try {
      await api.pauseWarmup(accountId);
      loadEmailAccounts();
    } catch (error) {
      console.error('Failed to pause warmup:', error);
    }
  };

  const handleAddAccount = async () => {
    try {
      await api.createEmailAccount({
        email: newAccount.email,
        provider: newAccount.provider,
        smtp_host: newAccount.smtp_host,
        smtp_port: newAccount.smtp_port,
        smtp_username: newAccount.smtp_username,
        smtp_password: newAccount.smtp_password
      });
      setShowAddModal(false);
      setNewAccount({ email: '', provider: 'google', smtp_host: '', smtp_port: 587, smtp_username: '', smtp_password: '' });
      loadEmailAccounts();
    } catch (error) {
      console.error('Failed to add email account:', error);
      alert('Failed to add email account');
    }
  };

  // Use API data or empty array
  const displayInboxes = inboxes.length > 0 ? inboxes : [];

  const warmupHistory = [
    { day: 'Day 1', sent: 5, inbox: 5, spam: 0, bounce: 0 },
    { day: 'Day 3', sent: 10, inbox: 10, spam: 0, bounce: 0 },
    { day: 'Day 5', sent: 15, inbox: 15, spam: 0, bounce: 0 },
    { day: 'Day 7', sent: 20, inbox: 20, spam: 0, bounce: 0 },
    { day: 'Day 10', sent: 25, inbox: 24, spam: 1, bounce: 0 },
    { day: 'Day 14', sent: 30, inbox: 29, spam: 1, bounce: 0 },
    { day: 'Day 17', sent: 35, inbox: 34, spam: 1, bounce: 0 },
    { day: 'Day 21', sent: 40, inbox: 39, spam: 1, bounce: 0 },
    { day: 'Day 23', sent: 47, inbox: 46, spam: 0, bounce: 1 }
  ];

  const healthHistory = [
    { date: 'Dec 11', score: 88 },
    { date: 'Dec 12', score: 90 },
    { date: 'Dec 13', score: 91 },
    { date: 'Dec 14', score: 93 },
    { date: 'Dec 15', score: 94 },
    { date: 'Dec 16', score: 95 },
    { date: 'Dec 17', score: 96 }
  ];

  const selectedInboxData = inboxes.find(i => i.id === selectedInbox);

  const getStatusBadge = (status: string) => {
    switch(status) {
      case 'active':
        return (
          <span className="flex items-center gap-1 px-3 py-1 bg-green-50 text-green-700 text-xs font-semibold rounded-full">
            <CheckCircle size={12} />
            Active
          </span>
        );
      case 'warming':
        return (
          <span className="flex items-center gap-1 px-3 py-1 bg-blue-50 text-blue-700 text-xs font-semibold rounded-full">
            <Activity size={12} />
            Warming
          </span>
        );
      case 'warning':
        return (
          <span className="flex items-center gap-1 px-3 py-1 bg-yellow-50 text-yellow-700 text-xs font-semibold rounded-full">
            <AlertTriangle size={12} />
            Warning
          </span>
        );
      case 'paused':
        return (
          <span className="flex items-center gap-1 px-3 py-1 bg-nord-elevated text-nord-text-muted text-xs font-semibold rounded-full">
            <Pause size={12} />
            Paused
          </span>
        );
      default:
        return null;
    }
  };

  const getHealthColor = (score: number) => {
    if (score >= 90) return 'text-green-600';
    if (score >= 75) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getHealthBgColor = (score: number) => {
    if (score >= 90) return 'bg-green-500';
    if (score >= 75) return 'bg-yellow-500';
    return 'bg-red-500';
  };

  return (
    <div className="min-h-[calc(100vh-4rem)]">
      {/* Page Header */}
      <div className="border-b border-nord-elevated/50 bg-nord-surface/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-xl font-bold text-nord-text">Email Warmup</h1>
              <p className="text-sm text-nord-text-muted">Monitor domain health and deliverability</p>
            </div>
            <div className="flex gap-3">
              <button 
                onClick={loadEmailAccounts}
                disabled={loading}
                className="px-4 py-2 border border-nord-elevated text-nord-text-muted rounded-lg hover:bg-nord-elevated/50 transition-all flex items-center gap-2 disabled:opacity-50"
              >
                <RefreshCw size={16} className={loading ? 'animate-spin' : ''} />
                Refresh
              </button>
              <button 
                onClick={() => setShowAddModal(true)}
                className="px-4 py-2 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 transition-all flex items-center gap-2 font-medium"
              >
                <Plus size={16} />
                Add Inbox
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 py-6">

      {/* Add Inbox Modal */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-nord-surface rounded-xl p-6 w-full max-w-md border border-nord-elevated/50">
            <h3 className="text-lg font-bold text-nord-text mb-4">Add Email Account</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-nord-text-secondary mb-1">Email Address</label>
                <input
                  type="email"
                  value={newAccount.email}
                  onChange={(e) => setNewAccount({...newAccount, email: e.target.value})}
                  className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  placeholder="you@company.com"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-nord-text-secondary mb-1">Provider</label>
                <select
                  value={newAccount.provider}
                  onChange={(e) => setNewAccount({...newAccount, provider: e.target.value})}
                  className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                >
                  <option value="google">Google Workspace</option>
                  <option value="microsoft">Microsoft 365</option>
                  <option value="other">Other SMTP</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-nord-text-secondary mb-1">SMTP Host</label>
                <input
                  type="text"
                  value={newAccount.smtp_host}
                  onChange={(e) => setNewAccount({...newAccount, smtp_host: e.target.value})}
                  className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  placeholder="smtp.gmail.com"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-1">SMTP Port</label>
                  <input
                    type="number"
                    value={newAccount.smtp_port}
                    onChange={(e) => setNewAccount({...newAccount, smtp_port: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-1">Username</label>
                  <input
                    type="text"
                    value={newAccount.smtp_username}
                    onChange={(e) => setNewAccount({...newAccount, smtp_username: e.target.value})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-nord-text-secondary mb-1">Password / App Password</label>
                <input
                  type="password"
                  value={newAccount.smtp_password}
                  onChange={(e) => setNewAccount({...newAccount, smtp_password: e.target.value})}
                  className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                />
              </div>
            </div>
            <div className="flex gap-3 mt-6">
              <button
                onClick={() => setShowAddModal(false)}
                className="flex-1 px-4 py-2 border border-nord-elevated rounded-lg hover:bg-nord-elevated/50 transition-all"
              >
                Cancel
              </button>
              <button
                onClick={handleAddAccount}
                className="flex-1 px-4 py-2 bg-nord-frost3 text-white rounded-lg hover:bg-nord-frost2 transition-all"
              >
                Add Account
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Link to Campaigns Modal */}
      {showLinkModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-nord-surface rounded-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-bold mb-2">Link Inbox to Campaigns</h3>
            <p className="text-sm text-nord-text-muted mb-4">
              Select which campaigns this inbox should send emails for. The inbox will be used for initial outreach and follow-ups.
            </p>
            
            {campaigns.length === 0 ? (
              <div className="text-center py-8 text-nord-text-muted">
                <Mail size={32} className="mx-auto mb-2 opacity-50" />
                <p>No campaigns yet</p>
                <button 
                  onClick={() => router.push('/dashboard/campaigns/new')}
                  className="mt-3 text-sm text-nord-text underline"
                >
                  Create your first campaign
                </button>
              </div>
            ) : (
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {campaigns.map(campaign => (
                  <label
                    key={campaign.id}
                    className={`flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-all ${
                      selectedCampaignIds.includes(campaign.id)
                        ? 'border-stone-900 bg-nord-elevated/50'
                        : 'border-nord-elevated/50 hover:border-nord-elevated'
                    }`}
                  >
                    <input
                      type="checkbox"
                      checked={selectedCampaignIds.includes(campaign.id)}
                      onChange={() => toggleCampaignSelection(campaign.id)}
                      className="w-4 h-4 rounded border-nord-elevated text-nord-text focus:ring-nord-frost3"
                    />
                    <div className="flex-1">
                      <div className="font-medium text-sm">{campaign.name}</div>
                      <div className="text-xs text-nord-text-muted">
                        {campaign.status} • {campaign.total_leads} leads • {campaign.sent} sent
                      </div>
                    </div>
                  </label>
                ))}
              </div>
            )}

            <div className="flex gap-3 mt-6">
              <button
                onClick={() => { setShowLinkModal(false); setLinkingInboxId(null); }}
                className="flex-1 px-4 py-2 border border-nord-elevated rounded-lg hover:bg-nord-elevated/50 transition-all"
              >
                Cancel
              </button>
              <button
                onClick={handleLinkCampaigns}
                disabled={campaigns.length === 0}
                className="flex-1 px-4 py-2 bg-nord-frost3 text-white rounded-lg hover:bg-nord-frost2 transition-all disabled:opacity-50"
              >
                Save Links
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Overview Cards */}
      <div className="grid grid-cols-4 gap-6 mb-8">
        <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
          <div className="flex items-center justify-between mb-4">
            <Mail className="text-blue-600" size={24} />
            <span className="text-sm text-nord-text-muted">Total</span>
          </div>
          <div className="text-3xl font-bold mb-1">{inboxes.length}</div>
          <div className="text-sm text-nord-text-muted">Email Accounts</div>
        </div>

        <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
          <div className="flex items-center justify-between mb-4">
            <CheckCircle className="text-green-600" size={24} />
            <span className="text-sm text-nord-text-muted">Healthy</span>
          </div>
          <div className="text-3xl font-bold mb-1">
            {inboxes.filter(i => i.healthScore >= 90).length}
          </div>
          <div className="text-sm text-nord-text-muted">Active & Ready</div>
        </div>

        <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
          <div className="flex items-center justify-between mb-4">
            <Activity className="text-blue-600" size={24} />
            <span className="text-sm text-nord-text-muted">Progress</span>
          </div>
          <div className="text-3xl font-bold mb-1">
            {inboxes.filter(i => i.status === 'warming').length}
          </div>
          <div className="text-sm text-nord-text-muted">Warming Up</div>
        </div>

        <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
          <div className="flex items-center justify-between mb-4">
            <Shield className="text-purple-600" size={24} />
            <span className="text-sm text-nord-text-muted">Average</span>
          </div>
          <div className="text-3xl font-bold mb-1">
            {Math.round(inboxes.reduce((sum, i) => sum + i.healthScore, 0) / inboxes.length)}
          </div>
          <div className="text-sm text-nord-text-muted">Health Score</div>
        </div>
      </div>

      {/* Inbox List */}
      <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 mb-8">
        <div className="px-6 py-4 border-b border-nord-elevated/50">
          <h2 className="text-lg font-bold">Email Accounts</h2>
        </div>
        <div className="divide-y divide-nord-elevated/30">
          {inboxes.map(inbox => (
            <div
              key={inbox.id}
              onClick={() => setSelectedInbox(inbox.id)}
              className={`px-6 py-4 cursor-pointer transition-all ${
                selectedInbox === inbox.id ? 'bg-nord-elevated/50' : 'hover:bg-nord-elevated/50'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <span className="font-semibold text-nord-text">{inbox.email}</span>
                    {getStatusBadge(inbox.status)}
                    <span className="text-xs text-nord-text-muted">{inbox.provider}</span>
                  </div>
                  
                  <div className="flex items-center gap-6 text-sm">
                    <div className="flex items-center gap-2">
                      <span className="text-nord-text-muted">Health:</span>
                      <span className={`font-bold ${getHealthColor(inbox.healthScore)}`}>
                        {inbox.healthScore}/100
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-nord-text-muted">Volume:</span>
                      <span className="font-medium">{inbox.currentVolume}/{inbox.targetVolume} per day</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-nord-text-muted">Progress:</span>
                      <span className="font-medium">{inbox.warmupProgress}%</span>
                    </div>
                  </div>

                  {/* Progress Bar */}
                  <div className="mt-3 w-full bg-nord-elevated rounded-full h-2">
                    <div
                      className={`h-2 rounded-full transition-all ${getHealthBgColor(inbox.healthScore)}`}
                      style={{ width: `${inbox.warmupProgress}%` }}
                    ></div>
                  </div>

                  {/* Linked Campaigns */}
                  {inbox.linkedCampaigns && inbox.linkedCampaigns.length > 0 && (
                    <div className="mt-3 flex items-center gap-2 flex-wrap">
                      <span className="text-xs text-nord-text-muted">Linked to:</span>
                      {inbox.linkedCampaigns.map(campaignId => {
                        const campaign = campaigns.find(c => c.id === campaignId);
                        return campaign ? (
                          <span 
                            key={campaignId}
                            className="px-2 py-0.5 bg-blue-50 text-blue-700 text-xs rounded-full"
                          >
                            {campaign.name}
                          </span>
                        ) : null;
                      })}
                    </div>
                  )}

                  {/* Issues */}
                  {inbox.issues.length > 0 && (
                    <div className="mt-3 space-y-1">
                      {inbox.issues.map((issue, idx) => (
                        <div
                          key={idx}
                          className={`flex items-center gap-2 text-xs ${
                            issue.type === 'critical' ? 'text-red-600' : 'text-yellow-600'
                          }`}
                        >
                          <AlertTriangle size={12} />
                          {issue.message}
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                <div className="flex gap-2">
                  <button 
                    onClick={(e) => { e.stopPropagation(); handleCopyEmail(inbox.email); }}
                    className="p-2 border border-nord-elevated rounded-lg hover:bg-nord-elevated transition-all"
                    title="Copy email address"
                  >
                    {copiedEmail === inbox.email ? <CheckCircle size={16} className="text-green-600" /> : <Copy size={16} />}
                  </button>
                  <button 
                    onClick={(e) => { e.stopPropagation(); handleOpenLinkModal(inbox.id); }}
                    className="p-2 border border-nord-elevated rounded-lg hover:bg-nord-elevated transition-all"
                    title="Link to campaigns"
                  >
                    <Link2 size={16} />
                  </button>
                  {inbox.status === 'paused' ? (
                    <button 
                      onClick={(e) => { e.stopPropagation(); handleStartWarmup(inbox.id); }}
                      className="p-2 border border-nord-elevated rounded-lg hover:bg-nord-elevated transition-all"
                      title="Start warmup"
                    >
                      <Play size={16} />
                    </button>
                  ) : (
                    <button 
                      onClick={(e) => { e.stopPropagation(); handlePauseWarmup(inbox.id); }}
                      className="p-2 border border-nord-elevated rounded-lg hover:bg-nord-elevated transition-all"
                      title="Pause warmup"
                    >
                      <Pause size={16} />
                    </button>
                  )}
                  <button className="p-2 border border-nord-elevated rounded-lg hover:bg-nord-elevated transition-all" title="Settings">
                    <Settings size={16} />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Detailed Analytics for Selected Inbox */}
      {selectedInboxData && (
        <div className="grid grid-cols-3 gap-6 mb-8">
          {/* Health Score Trend */}
          <div className="col-span-2 bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
            <h3 className="font-bold mb-6">Health Score Trend</h3>
            <ResponsiveContainer width="100%" height={250}>
              <LineChart data={healthHistory}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" />
                <XAxis dataKey="date" stroke="#737373" fontSize={12} />
                <YAxis domain={[0, 100]} stroke="#737373" fontSize={12} />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#3b4252',
                    border: '1px solid #4c566a',
                    borderRadius: '8px',
                    color: '#eceff4'
                  }}
                />
                <Line
                  type="monotone"
                  dataKey="score"
                  stroke="#10b981"
                  strokeWidth={3}
                  dot={{ fill: '#10b981', r: 4 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>

          {/* Deliverability Metrics */}
          <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
            <h3 className="font-bold mb-6">Deliverability</h3>
            <div className="space-y-4">
              <div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm text-nord-text-muted">Inbox Rate</span>
                  <span className="text-sm font-bold text-green-600">
                    {selectedInboxData.metrics.inboxRate}%
                  </span>
                </div>
                <div className="w-full bg-nord-elevated rounded-full h-2">
                  <div
                    className="bg-green-500 h-2 rounded-full"
                    style={{ width: `${selectedInboxData.metrics.inboxRate}%` }}
                  ></div>
                </div>
              </div>

              <div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm text-nord-text-muted">Open Rate</span>
                  <span className="text-sm font-bold text-blue-600">
                    {selectedInboxData.metrics.openRate}%
                  </span>
                </div>
                <div className="w-full bg-nord-elevated rounded-full h-2">
                  <div
                    className="bg-blue-500 h-2 rounded-full"
                    style={{ width: `${selectedInboxData.metrics.openRate}%` }}
                  ></div>
                </div>
              </div>

              <div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm text-nord-text-muted">Spam Rate</span>
                  <span className={`text-sm font-bold ${
                    selectedInboxData.metrics.spamRate < 2 ? 'text-green-600' : 'text-red-600'
                  }`}>
                    {selectedInboxData.metrics.spamRate}%
                  </span>
                </div>
                <div className="w-full bg-nord-elevated rounded-full h-2">
                  <div
                    className={`h-2 rounded-full ${
                      selectedInboxData.metrics.spamRate < 2 ? 'bg-green-500' : 'bg-red-500'
                    }`}
                    style={{ width: `${Math.min(selectedInboxData.metrics.spamRate * 10, 100)}%` }}
                  ></div>
                </div>
              </div>

              <div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm text-nord-text-muted">Bounce Rate</span>
                  <span className={`text-sm font-bold ${
                    selectedInboxData.metrics.bounceRate < 2 ? 'text-green-600' : 'text-yellow-600'
                  }`}>
                    {selectedInboxData.metrics.bounceRate}%
                  </span>
                </div>
                <div className="w-full bg-nord-elevated rounded-full h-2">
                  <div
                    className={`h-2 rounded-full ${
                      selectedInboxData.metrics.bounceRate < 2 ? 'bg-green-500' : 'bg-yellow-500'
                    }`}
                    style={{ width: `${Math.min(selectedInboxData.metrics.bounceRate * 10, 100)}%` }}
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Warmup Progress Chart */}
      <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6 mb-8">
        <div className="flex items-center justify-between mb-6">
          <h3 className="font-bold">Warmup Volume Progression</h3>
          <div className="flex gap-4 text-sm">
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 bg-nord-frost3 rounded-full"></div>
              <span className="text-nord-text-muted">Sent</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 bg-green-500 rounded-full"></div>
              <span className="text-nord-text-muted">Inbox</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 bg-red-500 rounded-full"></div>
              <span className="text-nord-text-muted">Spam</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
              <span className="text-nord-text-muted">Bounce</span>
            </div>
          </div>
        </div>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={warmupHistory}>
            <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" />
            <XAxis dataKey="day" stroke="#737373" fontSize={12} />
            <YAxis stroke="#737373" fontSize={12} />
            <Tooltip
              contentStyle={{
                backgroundColor: 'white',
                border: '1px solid #e5e5e5',
                borderRadius: '8px'
              }}
            />
            <Line type="monotone" dataKey="sent" stroke="#1a1a1a" strokeWidth={2} name="Sent" />
            <Line type="monotone" dataKey="inbox" stroke="#10b981" strokeWidth={2} name="Inbox" />
            <Line type="monotone" dataKey="spam" stroke="#ef4444" strokeWidth={2} name="Spam" />
            <Line type="monotone" dataKey="bounce" stroke="#f59e0b" strokeWidth={2} name="Bounce" />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Best Practices */}
      <div className="bg-blue-50 border border-blue-200 rounded-xl p-6">
        <h3 className="font-bold text-blue-900 mb-4 flex items-center gap-2">
          <Shield size={20} />
          Warmup Best Practices
        </h3>
        <div className="grid grid-cols-2 gap-4 text-sm text-blue-800">
          <div>
            <div className="font-semibold mb-1">• Start Slow</div>
            <div className="text-blue-700 ml-4">Begin with 5-10 emails per day</div>
          </div>
          <div>
            <div className="font-semibold mb-1">• Gradual Increase</div>
            <div className="text-blue-700 ml-4">Increase by 10-15% every 3-5 days</div>
          </div>
          <div>
            <div className="font-semibold mb-1">• Monitor Health</div>
            <div className="text-blue-700 ml-4">Keep health score above 90</div>
          </div>
          <div>
            <div className="font-semibold mb-1">• Typical Duration</div>
            <div className="text-blue-700 ml-4">30-45 days to reach full volume</div>
          </div>
        </div>
      </div>
      </div>
    </div>
  );
}
