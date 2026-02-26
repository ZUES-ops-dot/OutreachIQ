'use client';

import { useState, useEffect } from 'react';
import { useSearchParams } from 'next/navigation';
import { Mail, Users, Bell, Shield, Database, Save, RotateCcw, Wand2, Copy } from 'lucide-react';
import { api } from '@/lib/api';

export default function SettingsPage() {
  const searchParams = useSearchParams();
  const tabParam = searchParams.get('tab');
  const [activeSection, setActiveSection] = useState(tabParam || 'email');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (tabParam) {
      setActiveSection(tabParam);
    }
  }, [tabParam]);

  const [settings, setSettings] = useState({
    // Email Settings
    dailyLimit: 50,
    warmupDuration: 30,
    warmupStartVolume: 5,
    warmupIncrement: 2,
    sendDelay: 60,
    bounceThreshold: 5,
    // Lead Settings
    defaultLeadLimit: 100,
    verificationThreshold: 70,
    autoVerify: true,
    duplicateDetection: 'email',
    // Campaign Settings
    followUpDelay: 3,
    maxFollowUps: 3,
    sendWindowStart: '09:00',
    sendWindowEnd: '17:00',
    sendOnWeekends: false,
    timezone: 'America/New_York',
    // Notification Settings
    emailNotifications: true,
    dailyDigest: true,
    replyAlerts: true,
    bounceAlerts: true,
    // API Settings
    apiRateLimit: 100,
    webhookUrl: ''
  });

  const [generatorForm, setGeneratorForm] = useState({
    template: 'cold_outreach',
    firstName: 'Jordan',
    company: 'Acme Ventures',
    signal: 'scaling your outbound team',
    customMessage: 'We help teams like yours keep inbox placement high while automating warmup and compliance.',
    followUpMessage: 'Just wanted to circle back to make sure this stayed on your radar.',
    senderName: 'Alex from OutreachIQ',
    senderTitle: 'Growth Lead'
  });
  const [generatedEmail, setGeneratedEmail] = useState({ subject: '', body: '' });
  const [copiedField, setCopiedField] = useState<'subject' | 'body' | null>(null);

  const handleGenerateEmail = () => {
    const {
      template,
      firstName,
      company,
      signal,
      customMessage,
      followUpMessage,
      senderName,
      senderTitle
    } = generatorForm;

    const safeName = firstName || 'there';
    const safeCompany = company || 'your company';
    let subject = '';
    let body = '';

    if (template === 'follow_up') {
      subject = `Re: Quick question about ${safeCompany}`;
      body = `Hi ${safeName},

Just wanted to follow up on my previous note. Totally understand you're busy, but I think this could be valuable for ${safeCompany}.

${followUpMessage || 'Let me know if you had a chance to review my earlier email.'}

Happy to share more context or a case study if helpful.

Best,
${senderName}
${senderTitle}`;
    } else {
      subject = `Quick question about ${safeCompany}`;
      body = `Hi ${safeName},

I noticed ${safeCompany} is ${signal} and thought to reach out.

${customMessage}

Would you be open to a quick 15-minute chat this week?

Best,
${senderName}
${senderTitle}`;
    }

    setGeneratedEmail({ subject, body });
    setCopiedField(null);
  };

  const handleCopy = async (field: 'subject' | 'body') => {
    try {
      await navigator.clipboard.writeText(generatedEmail[field]);
      setCopiedField(field);
      setTimeout(() => setCopiedField(null), 2000);
    } catch (error) {
      console.error('Failed to copy text', error);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      // Map frontend settings to backend WorkspaceSettings format
      await api.updateFounderSettings({
        auto_pause_enabled: true,
        spam_rate_threshold: settings.bounceThreshold,
        bounce_rate_threshold: settings.bounceThreshold,
        reply_drop_threshold: 20,
        google_daily_limit: settings.dailyLimit,
        outlook_daily_limit: settings.dailyLimit,
        zoho_daily_limit: settings.dailyLimit,
        notification_email: settings.emailNotifications ? null : null,
        slack_webhook_url: settings.webhookUrl || null,
      });
      alert('Settings saved successfully!');
    } catch (error) {
      console.error('Failed to save settings:', error);
      alert('Failed to save settings. Please try again.');
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    if (confirm('Are you sure you want to reset all settings to defaults?')) {
      setSettings({
        dailyLimit: 50,
        warmupDuration: 30,
        warmupStartVolume: 5,
        warmupIncrement: 2,
        sendDelay: 60,
        bounceThreshold: 5,
        defaultLeadLimit: 100,
        verificationThreshold: 70,
        autoVerify: true,
        duplicateDetection: 'email',
        followUpDelay: 3,
        maxFollowUps: 3,
        sendWindowStart: '09:00',
        sendWindowEnd: '17:00',
        sendOnWeekends: false,
        timezone: 'America/New_York',
        emailNotifications: true,
        dailyDigest: true,
        replyAlerts: true,
        bounceAlerts: true,
        apiRateLimit: 100,
        webhookUrl: ''
      });
    }
  };

  const sections = [
    { id: 'email', label: 'Email Settings', icon: Mail },
    { id: 'leads', label: 'Lead Generation', icon: Users },
    { id: 'campaigns', label: 'Campaigns', icon: Database },
    { id: 'notifications', label: 'Notifications', icon: Bell },
    { id: 'api', label: 'API & Integrations', icon: Shield },
    { id: 'generator', label: 'Email Generator', icon: Wand2 }
  ];

  return (
    <div className="min-h-[calc(100vh-4rem)]">
      {/* Page Header */}
      <div className="border-b border-nord-elevated/50 bg-nord-surface/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-xl font-bold text-nord-text">Settings</h1>
              <p className="text-sm text-nord-text-muted">Configure your OutreachIQ preferences</p>
            </div>
            <div className="flex gap-3">
              <button
                onClick={handleReset}
                className="px-4 py-2 border border-nord-elevated text-nord-text-muted rounded-lg hover:bg-nord-elevated/50 transition-all flex items-center gap-2"
              >
                <RotateCcw size={16} />
                Reset
              </button>
              <button
                onClick={handleSave}
                disabled={saving}
                className="px-4 py-2 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 transition-all flex items-center gap-2 disabled:opacity-50 font-medium"
              >
                <Save size={16} />
                {saving ? 'Saving...' : 'Save Changes'}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 py-6">
        <div className="flex gap-6">
          {/* Sidebar */}
          <div className="w-56 flex-shrink-0">
            <nav className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-2">
              {sections.map(section => (
                <button
                  key={section.id}
                  onClick={() => setActiveSection(section.id)}
                  className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-left transition-all text-sm ${
                    activeSection === section.id
                      ? 'bg-nord-frost3/20 text-nord-frost3'
                      : 'text-nord-text-muted hover:text-nord-text hover:bg-nord-elevated/50'
                  }`}
                >
                  <section.icon size={18} />
                  {section.label}
                </button>
              ))}
            </nav>
          </div>

          {/* Content */}
          <div className="flex-1">
          {/* Email Settings */}
          {activeSection === 'email' && (
            <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <h3 className="font-bold text-lg mb-6">Email Settings</h3>
              <div className="grid grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Default Daily Limit</label>
                  <input
                    type="number"
                    value={settings.dailyLimit}
                    onChange={(e) => setSettings({...settings, dailyLimit: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Maximum emails per inbox per day</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Warmup Duration (days)</label>
                  <input
                    type="number"
                    value={settings.warmupDuration}
                    onChange={(e) => setSettings({...settings, warmupDuration: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Days to fully warm up new inboxes</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Warmup Start Volume</label>
                  <input
                    type="number"
                    value={settings.warmupStartVolume}
                    onChange={(e) => setSettings({...settings, warmupStartVolume: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Initial daily emails during warmup</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Warmup Increment</label>
                  <input
                    type="number"
                    value={settings.warmupIncrement}
                    onChange={(e) => setSettings({...settings, warmupIncrement: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Daily volume increase during warmup</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Send Delay (seconds)</label>
                  <input
                    type="number"
                    value={settings.sendDelay}
                    onChange={(e) => setSettings({...settings, sendDelay: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Minimum delay between emails</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Bounce Threshold (%)</label>
                  <input
                    type="number"
                    value={settings.bounceThreshold}
                    onChange={(e) => setSettings({...settings, bounceThreshold: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Pause sending if bounce rate exceeds</p>
                </div>
              </div>
            </div>
          )}

          {/* Lead Generation Settings */}
          {activeSection === 'leads' && (
            <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <h3 className="font-bold text-lg mb-6">Lead Generation Settings</h3>
              <div className="grid grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Default Lead Limit</label>
                  <input
                    type="number"
                    value={settings.defaultLeadLimit}
                    onChange={(e) => setSettings({...settings, defaultLeadLimit: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Max leads per search request</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Verification Confidence Threshold</label>
                  <input
                    type="number"
                    value={settings.verificationThreshold}
                    onChange={(e) => setSettings({...settings, verificationThreshold: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Minimum confidence score for valid leads</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Auto-verify New Leads</label>
                  <select 
                    value={settings.autoVerify ? 'true' : 'false'}
                    onChange={(e) => setSettings({...settings, autoVerify: e.target.value === 'true'})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  >
                    <option value="true">Enabled</option>
                    <option value="false">Disabled</option>
                  </select>
                  <p className="text-xs text-nord-text-muted mt-1">Automatically verify emails on import</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Duplicate Detection</label>
                  <select 
                    value={settings.duplicateDetection}
                    onChange={(e) => setSettings({...settings, duplicateDetection: e.target.value})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  >
                    <option value="email">By Email</option>
                    <option value="domain">By Domain</option>
                    <option value="both">Both</option>
                  </select>
                  <p className="text-xs text-nord-text-muted mt-1">How to detect duplicate leads</p>
                </div>
              </div>
            </div>
          )}

          {/* Campaign Settings */}
          {activeSection === 'campaigns' && (
            <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <h3 className="font-bold text-lg mb-6">Campaign Settings</h3>
              <div className="grid grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Default Follow-up Delay (days)</label>
                  <input
                    type="number"
                    value={settings.followUpDelay}
                    onChange={(e) => setSettings({...settings, followUpDelay: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Days between follow-up emails</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Max Follow-ups</label>
                  <input
                    type="number"
                    value={settings.maxFollowUps}
                    onChange={(e) => setSettings({...settings, maxFollowUps: parseInt(e.target.value)})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Maximum follow-up emails per lead</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Send Window Start</label>
                  <input
                    type="time"
                    value={settings.sendWindowStart}
                    onChange={(e) => setSettings({...settings, sendWindowStart: e.target.value})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Earliest time to send emails</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Send Window End</label>
                  <input
                    type="time"
                    value={settings.sendWindowEnd}
                    onChange={(e) => setSettings({...settings, sendWindowEnd: e.target.value})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Latest time to send emails</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Send on Weekends</label>
                  <select 
                    value={settings.sendOnWeekends ? 'true' : 'false'}
                    onChange={(e) => setSettings({...settings, sendOnWeekends: e.target.value === 'true'})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  >
                    <option value="false">No</option>
                    <option value="true">Yes</option>
                  </select>
                  <p className="text-xs text-nord-text-muted mt-1">Allow sending on Saturday/Sunday</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Timezone</label>
                  <select 
                    value={settings.timezone}
                    onChange={(e) => setSettings({...settings, timezone: e.target.value})}
                    className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  >
                    <option value="America/New_York">Eastern (ET)</option>
                    <option value="America/Chicago">Central (CT)</option>
                    <option value="America/Denver">Mountain (MT)</option>
                    <option value="America/Los_Angeles">Pacific (PT)</option>
                    <option value="UTC">UTC</option>
                    <option value="Europe/London">London (GMT)</option>
                    <option value="Europe/Paris">Paris (CET)</option>
                  </select>
                  <p className="text-xs text-nord-text-muted mt-1">Timezone for send windows</p>
                </div>
              </div>
            </div>
          )}

          {/* Notification Settings */}
          {activeSection === 'notifications' && (
            <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <h3 className="font-bold text-lg mb-6">Notification Settings</h3>
              <div className="space-y-6">
                <div className="flex items-center justify-between py-4 border-b border-nord-elevated/30">
                  <div>
                    <h4 className="font-medium text-nord-text">Email Notifications</h4>
                    <p className="text-sm text-nord-text-muted">Receive notifications via email</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input 
                      type="checkbox" 
                      checked={settings.emailNotifications}
                      onChange={(e) => setSettings({...settings, emailNotifications: e.target.checked})}
                      className="sr-only peer" 
                    />
                    <div className="w-11 h-6 bg-nord-elevated peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-nord-frost3/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-nord-elevated after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-nord-frost3"></div>
                  </label>
                </div>
                <div className="flex items-center justify-between py-4 border-b border-nord-elevated/30">
                  <div>
                    <h4 className="font-medium text-nord-text">Daily Digest</h4>
                    <p className="text-sm text-nord-text-muted">Get a daily summary of campaign performance</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input 
                      type="checkbox" 
                      checked={settings.dailyDigest}
                      onChange={(e) => setSettings({...settings, dailyDigest: e.target.checked})}
                      className="sr-only peer" 
                    />
                    <div className="w-11 h-6 bg-nord-elevated peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-nord-frost3/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-nord-elevated after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-nord-frost3"></div>
                  </label>
                </div>
                <div className="flex items-center justify-between py-4 border-b border-nord-elevated/30">
                  <div>
                    <h4 className="font-medium text-nord-text">Reply Alerts</h4>
                    <p className="text-sm text-nord-text-muted">Get notified when leads reply</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input 
                      type="checkbox" 
                      checked={settings.replyAlerts}
                      onChange={(e) => setSettings({...settings, replyAlerts: e.target.checked})}
                      className="sr-only peer" 
                    />
                    <div className="w-11 h-6 bg-nord-elevated peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-nord-frost3/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-nord-elevated after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-nord-frost3"></div>
                  </label>
                </div>
                <div className="flex items-center justify-between py-4">
                  <div>
                    <h4 className="font-medium text-nord-text">Bounce Alerts</h4>
                    <p className="text-sm text-nord-text-muted">Get notified when bounce rate is high</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input 
                      type="checkbox" 
                      checked={settings.bounceAlerts}
                      onChange={(e) => setSettings({...settings, bounceAlerts: e.target.checked})}
                      className="sr-only peer" 
                    />
                    <div className="w-11 h-6 bg-nord-elevated peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-nord-frost3/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-nord-elevated after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-nord-frost3"></div>
                  </label>
                </div>
              </div>
            </div>
          )}

          {/* API Settings */}
          {activeSection === 'api' && (
            <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <h3 className="font-bold text-lg mb-6">API & Integrations</h3>
              <div className="space-y-6">
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">API Rate Limit (requests/min)</label>
                  <input
                    type="number"
                    value={settings.apiRateLimit}
                    onChange={(e) => setSettings({...settings, apiRateLimit: parseInt(e.target.value)})}
                    className="w-full max-w-md px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Max API requests per minute</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-nord-text-secondary mb-2">Webhook URL</label>
                  <input
                    type="url"
                    value={settings.webhookUrl}
                    onChange={(e) => setSettings({...settings, webhookUrl: e.target.value})}
                    placeholder="https://your-webhook.com/endpoint"
                    className="w-full max-w-md px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                  />
                  <p className="text-xs text-nord-text-muted mt-1">Receive events via webhook</p>
                </div>
                <div className="pt-6 border-t border-nord-elevated/30">
                  <h4 className="font-medium text-nord-text mb-4">API Key</h4>
                  <div className="flex gap-3">
                    <input
                      type="password"
                      value="••••••••••••••••••••••••"
                      readOnly
                      className="flex-1 max-w-md px-4 py-2 border border-nord-elevated rounded-lg bg-nord-bg"
                    />
                    <button className="px-4 py-2 border border-nord-elevated rounded-lg hover:bg-nord-bg transition-all">
                      Copy
                    </button>
                    <button className="px-4 py-2 border border-nord-red rounded-lg hover:bg-nord-red/10 transition-all">
                      Regenerate
                    </button>
                  </div>
                  <p className="text-xs text-nord-text-muted mt-2">Use this key to authenticate API requests</p>
                </div>
              </div>
            </div>
          )}

          {/* Email Generator */}
          {activeSection === 'generator' && (
            <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <div className="flex items-center justify-between mb-6">
                <div>
                  <h3 className="font-bold text-lg">AI Email Generator</h3>
                  <p className="text-sm text-nord-text-muted">
                    Pre-fill a cold or follow-up email using OutreachIQ templates
                  </p>
                </div>
                <button
                  onClick={handleGenerateEmail}
                  className="px-4 py-2 bg-nord-frost3 text-white rounded-lg flex items-center gap-2 hover:bg-nord-frost2 transition-all"
                >
                  <Wand2 size={16} />
                  Generate Email
                </button>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                {/* Form */}
                <div className="space-y-5">
                  <div>
                    <label className="block text-sm font-medium text-nord-text-secondary mb-2">
                      Template
                    </label>
                    <select
                      value={generatorForm.template}
                      onChange={(e) => setGeneratorForm({ ...generatorForm, template: e.target.value })}
                      className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                    >
                      <option value="cold_outreach">Cold Outreach</option>
                      <option value="follow_up">Follow Up</option>
                    </select>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-nord-text-secondary mb-2">First Name</label>
                      <input
                        type="text"
                        value={generatorForm.firstName}
                        onChange={(e) => setGeneratorForm({ ...generatorForm, firstName: e.target.value })}
                        className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-nord-text-secondary mb-2">Company</label>
                      <input
                        type="text"
                        value={generatorForm.company}
                        onChange={(e) => setGeneratorForm({ ...generatorForm, company: e.target.value })}
                        className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                      />
                    </div>
                  </div>

                  {generatorForm.template === 'cold_outreach' && (
                    <>
                      <div>
                        <label className="block text-sm font-medium text-nord-text-secondary mb-2">Signal</label>
                        <input
                          type="text"
                          value={generatorForm.signal}
                          onChange={(e) => setGeneratorForm({ ...generatorForm, signal: e.target.value })}
                          placeholder="e.g. hiring SDRs, launching in EU"
                          className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-nord-text-secondary mb-2">Custom Message</label>
                        <textarea
                          value={generatorForm.customMessage}
                          onChange={(e) => setGeneratorForm({ ...generatorForm, customMessage: e.target.value })}
                          rows={4}
                          className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                        />
                      </div>
                    </>
                  )}

                  {generatorForm.template === 'follow_up' && (
                    <div>
                      <label className="block text-sm font-medium text-nord-text-secondary mb-2">Follow-up angle</label>
                      <textarea
                        value={generatorForm.followUpMessage}
                        onChange={(e) => setGeneratorForm({ ...generatorForm, followUpMessage: e.target.value })}
                        rows={4}
                        className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                      />
                    </div>
                  )}

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-nord-text-secondary mb-2">Sender Name</label>
                      <input
                        type="text"
                        value={generatorForm.senderName}
                        onChange={(e) => setGeneratorForm({ ...generatorForm, senderName: e.target.value })}
                        className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-nord-text-secondary mb-2">Sender Title</label>
                      <input
                        type="text"
                        value={generatorForm.senderTitle}
                        onChange={(e) => setGeneratorForm({ ...generatorForm, senderTitle: e.target.value })}
                        className="w-full px-4 py-2 border border-nord-elevated rounded-lg focus:outline-none focus:ring-2 focus:ring-nord-frost3"
                      />
                    </div>
                  </div>
                </div>

                {/* Preview */}
                <div className="space-y-5">
                  <div>
                    <label className="flex items-center justify-between text-sm font-medium text-nord-text-secondary mb-2">
                      Subject
                      <button
                        onClick={() => handleCopy('subject')}
                        className="text-xs text-nord-text-muted flex items-center gap-1 hover:text-nord-text"
                      >
                        <Copy size={14} />
                        {copiedField === 'subject' ? 'Copied' : 'Copy'}
                      </button>
                    </label>
                    <textarea
                      value={generatedEmail.subject}
                      readOnly
                      rows={2}
                      className="w-full px-4 py-2 border border-nord-elevated rounded-lg bg-nord-bg focus:outline-none"
                    />
                  </div>
                  <div>
                    <label className="flex items-center justify-between text-sm font-medium text-nord-text-secondary mb-2">
                      Body
                      <button
                        onClick={() => handleCopy('body')}
                        className="text-xs text-nord-text-muted flex items-center gap-1 hover:text-nord-text"
                      >
                        <Copy size={14} />
                        {copiedField === 'body' ? 'Copied' : 'Copy'}
                      </button>
                    </label>
                    <textarea
                      value={generatedEmail.body}
                      readOnly
                      rows={14}
                      className="w-full px-4 py-2 border border-nord-elevated rounded-lg bg-nord-bg font-mono text-sm focus:outline-none"
                    />
                  </div>
                  <p className="text-xs text-nord-text-muted">
                    Tip: tweak the copy after generating to reflect the prospect&apos;s tone.
                  </p>
                </div>
              </div>
            </div>
          )}
          </div>
        </div>
      </div>
    </div>
  );
}
