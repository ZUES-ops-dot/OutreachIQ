'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { Plus, Play, Pause, Trash2, Mail, Users, TrendingUp } from 'lucide-react';
import { api, Campaign } from '@/lib/api';

export default function CampaignsPage() {
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadCampaigns();
  }, []);

  const loadCampaigns = async () => {
    try {
      const data = await api.getCampaigns();
      setCampaigns(data);
    } catch (error) {
      console.error('Failed to load campaigns:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleStart = async (id: string) => {
    try {
      await api.startCampaign(id);
      loadCampaigns();
    } catch (error) {
      console.error('Failed to start campaign:', error);
    }
  };

  const handlePause = async (id: string) => {
    try {
      await api.pauseCampaign(id);
      loadCampaigns();
    } catch (error) {
      console.error('Failed to pause campaign:', error);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this campaign?')) return;
    try {
      await api.deleteCampaign(id);
      loadCampaigns();
    } catch (error) {
      console.error('Failed to delete campaign:', error);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-nord-success/20 text-nord-success';
      case 'paused':
        return 'bg-nord-warning/20 text-nord-warning';
      case 'completed':
        return 'bg-nord-frost3/20 text-nord-frost3';
      default:
        return 'bg-nord-elevated text-nord-text-muted';
    }
  };

  return (
    <div className="min-h-[calc(100vh-4rem)]">
      {/* Page Header */}
      <div className="border-b border-nord-elevated/50 bg-nord-surface/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-xl font-bold text-nord-text">Campaigns</h1>
              <p className="text-sm text-nord-text-muted">Manage your outreach campaigns</p>
            </div>
            <Link
              href="/dashboard/campaigns/new"
              className="bg-nord-frost3 text-nord-bg px-4 py-2 rounded-lg hover:bg-nord-frost2 transition-all flex items-center gap-2 font-medium"
            >
              <Plus size={20} />
              New Campaign
            </Link>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 py-6">

      {/* Campaign Cards */}
      <div className="grid gap-6">
        {loading ? (
          <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-12 text-center text-nord-text-muted">
            Loading campaigns...
          </div>
        ) : campaigns.length === 0 ? (
          <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-12 text-center">
            <Mail size={48} className="mx-auto mb-4 text-nord-text-muted" />
            <h3 className="text-lg font-semibold text-nord-text mb-2">No campaigns yet</h3>
            <p className="text-nord-text-muted mb-4">Create your first campaign to start reaching out to leads.</p>
            <Link
              href="/dashboard/campaigns/new"
              className="inline-flex items-center gap-2 bg-nord-frost3 text-nord-bg px-4 py-2 rounded-lg hover:bg-nord-frost2 transition-all font-medium"
            >
              <Plus size={20} />
              Create Campaign
            </Link>
          </div>
        ) : (
          campaigns.map((campaign) => (
            <div key={campaign.id} className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <div className="flex items-start justify-between mb-4">
                <div>
                  <div className="flex items-center gap-3 mb-1">
                    <h3 className="text-lg font-semibold text-nord-text">{campaign.name}</h3>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(campaign.status)}`}>
                      {campaign.status}
                    </span>
                  </div>
                  <p className="text-sm text-nord-text-muted">
                    {campaign.vertical.toUpperCase()} • Created {new Date(campaign.created_at).toLocaleDateString()}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  {campaign.status === 'draft' || campaign.status === 'paused' ? (
                    <button
                      onClick={() => handleStart(campaign.id)}
                      className="p-2 text-green-600 hover:bg-green-50 rounded-lg transition-all"
                      title="Start Campaign"
                    >
                      <Play size={20} />
                    </button>
                  ) : campaign.status === 'active' ? (
                    <button
                      onClick={() => handlePause(campaign.id)}
                      className="p-2 text-yellow-600 hover:bg-yellow-50 rounded-lg transition-all"
                      title="Pause Campaign"
                    >
                      <Pause size={20} />
                    </button>
                  ) : null}
                  <button
                    onClick={() => handleDelete(campaign.id)}
                    className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-all"
                    title="Delete Campaign"
                  >
                    <Trash2 size={20} />
                  </button>
                </div>
              </div>

              {/* Stats */}
              <div className="grid grid-cols-5 gap-4">
                <div className="bg-nord-elevated/50 rounded-lg p-3">
                  <div className="flex items-center gap-2 text-nord-text-muted mb-1">
                    <Users size={14} />
                    <span className="text-xs">Leads</span>
                  </div>
                  <div className="text-xl font-bold text-nord-text">{campaign.total_leads}</div>
                </div>
                <div className="bg-nord-elevated/50 rounded-lg p-3">
                  <div className="flex items-center gap-2 text-nord-text-muted mb-1">
                    <Mail size={14} />
                    <span className="text-xs">Sent</span>
                  </div>
                  <div className="text-xl font-bold text-nord-text">{campaign.sent}</div>
                </div>
                <div className="bg-nord-elevated/50 rounded-lg p-3">
                  <div className="flex items-center gap-2 text-nord-text-muted mb-1">
                    <TrendingUp size={14} />
                    <span className="text-xs">Opened</span>
                  </div>
                  <div className="text-xl font-bold text-nord-text">{campaign.opened}</div>
                </div>
                <div className="bg-nord-elevated/50 rounded-lg p-3">
                  <div className="text-xs text-nord-text-muted mb-1">Clicked</div>
                  <div className="text-xl font-bold text-nord-text">{campaign.clicked}</div>
                </div>
                <div className="bg-nord-elevated/50 rounded-lg p-3">
                  <div className="text-xs text-nord-text-muted mb-1">Replied</div>
                  <div className="text-xl font-bold text-nord-text">{campaign.replied}</div>
                </div>
              </div>

              {/* Progress Bar */}
              {campaign.total_leads > 0 && (
                <div className="mt-4">
                  <div className="flex justify-between text-xs text-nord-text-muted mb-1">
                    <span>Progress</span>
                    <span>{Math.round((campaign.sent / campaign.total_leads) * 100)}%</span>
                  </div>
                  <div className="h-2 bg-nord-elevated rounded-full overflow-hidden">
                    <div
                      className="h-full bg-nord-frost3 rounded-full transition-all"
                      style={{ width: `${(campaign.sent / campaign.total_leads) * 100}%` }}
                    />
                  </div>
                </div>
              )}
            </div>
          ))
        )}
      </div>
      </div>
    </div>
  );
}
