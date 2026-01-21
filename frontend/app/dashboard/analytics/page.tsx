'use client';

import { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';
import { TrendingUp, Mail, MousePointer, MessageSquare, AlertTriangle, BarChart3, RefreshCw } from 'lucide-react';
import { api, LeadAnalytics, DeliverabilityMetrics, Campaign } from '@/lib/api';

interface DailyStats {
  date: string;
  sent: number;
  opened: number;
  clicked: number;
  replied: number;
}

interface CampaignPerformance {
  id: string;
  name: string;
  sent: number;
  opened: number;
  replied: number;
  rate: number;
}

export default function AnalyticsPage() {
  const [leadAnalytics, setLeadAnalytics] = useState<LeadAnalytics | null>(null);
  const [deliverability, setDeliverability] = useState<DeliverabilityMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState('7d');
  const [selectedCampaign, setSelectedCampaign] = useState('all');
  const [campaigns, setCampaigns] = useState<Array<{id: string; name: string}>>([{ id: 'all', name: 'All Campaigns' }]);
  const [dailyData, setDailyData] = useState<DailyStats[]>([]);
  const [campaignPerformance, setCampaignPerformance] = useState<CampaignPerformance[]>([]);
  const [overviewData, setOverviewData] = useState<any>(null);

  // Calculate overview stats from real data
  const overviewStats = [
    { 
      label: 'Inbox Rate',
      value: `${Math.round((deliverability?.inbox_rate || 0) * 100)}%`,
      change: deliverability ? '+0%' : '--',
      trend: 'up',
      icon: Mail,
      bgColor: 'bg-blue-50',
      iconColor: 'text-blue-600'
    },
    { 
      label: 'Open Rate',
      value: overviewData ? `${Math.round((overviewData.total_opened / Math.max(overviewData.total_sent, 1)) * 100)}%` : '0%',
      change: '--',
      trend: 'up',
      icon: TrendingUp,
      bgColor: 'bg-green-50',
      iconColor: 'text-green-600'
    },
    { 
      label: 'Click Rate',
      value: overviewData ? `${Math.round((overviewData.total_clicked / Math.max(overviewData.total_sent, 1)) * 100)}%` : '0%',
      change: '--',
      trend: 'up',
      icon: MousePointer,
      bgColor: 'bg-purple-50',
      iconColor: 'text-purple-600'
    },
    { 
      label: 'Reply Rate',
      value: overviewData ? `${Math.round((overviewData.total_replied / Math.max(overviewData.total_sent, 1)) * 100)}%` : '0%',
      change: '--',
      trend: 'up',
      icon: MessageSquare,
      bgColor: 'bg-orange-50',
      iconColor: 'text-orange-600'
    }
  ];

  const verificationStatus = [
    { name: 'Valid', value: leadAnalytics?.verification_breakdown?.valid || 0, color: '#10b981' },
    { name: 'Risky', value: leadAnalytics?.verification_breakdown?.risky || 0, color: '#f59e0b' },
    { name: 'Invalid', value: leadAnalytics?.verification_breakdown?.invalid || 0, color: '#ef4444' },
    { name: 'Pending', value: leadAnalytics?.verification_breakdown?.pending || 0, color: '#6b7280' }
  ];

  const deliverabilityScore = [
    { metric: 'Inbox Rate', score: Math.round((deliverability?.inbox_rate || 0) * 100), color: '#10b981' },
    { metric: 'Bounce Rate', score: Math.round((deliverability?.bounce_rate || 0) * 100), color: '#ef4444' },
    { metric: 'Spam Rate', score: Math.round((deliverability?.spam_rate || 0) * 100 * 10) / 10, color: '#f59e0b' },
    { metric: 'Domain Health', score: Math.round(deliverability?.average_health_score || 0), color: '#3b82f6' }
  ];

  const recentReplies: Array<{name: string; company: string; email: string; reply: string; time: string; sentiment: string}> = [];

  useEffect(() => {
    loadAnalytics();
  }, []);

  const loadAnalytics = async () => {
    setLoading(true);
    try {
      // Load all analytics data from API
      const [leads, deliver, campaignList, overview] = await Promise.all([
        api.getLeadAnalytics(),
        api.getDeliverabilityMetrics(),
        api.getCampaigns(),
        api.getAnalyticsOverview()
      ]);
      
      setLeadAnalytics(leads);
      setDeliverability(deliver);
      setOverviewData(overview);
      
      // Set campaigns dropdown
      setCampaigns([
        { id: 'all', name: 'All Campaigns' },
        ...campaignList.map(c => ({ id: c.id, name: c.name }))
      ]);
      
      // Transform campaigns to performance data using correct property names
      const perfData: CampaignPerformance[] = campaignList.map(c => ({
        id: c.id,
        name: c.name,
        sent: c.sent || 0,
        opened: c.opened || 0,
        replied: c.replied || 0,
        rate: c.sent > 0 ? Math.round((c.replied / c.sent) * 100 * 10) / 10 : 0
      }));
      setCampaignPerformance(perfData);
    } catch (error) {
      console.error('Failed to load analytics:', error);
    } finally {
      setLoading(false);
    }
  };

  const getSentimentColor = (sentiment: string) => {
    switch(sentiment) {
      case 'positive': return 'bg-green-100 text-green-700';
      case 'negative': return 'bg-red-100 text-red-700';
      default: return 'bg-nord-elevated text-nord-text-muted';
    }
  };

  return (
    <div className="min-h-[calc(100vh-4rem)]">
      {/* Page Header */}
      <div className="border-b border-nord-elevated/50 bg-nord-surface/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-xl font-bold text-nord-text">Analytics</h1>
              <p className="text-sm text-nord-text-muted">Real-time campaign performance</p>
            </div>
            <div className="flex gap-3">
              <select
                value={selectedCampaign}
                onChange={(e) => setSelectedCampaign(e.target.value)}
                className="px-4 py-2 border border-nord-elevated rounded-lg bg-nord-surface text-nord-text focus:outline-none focus:ring-2 focus:ring-nord-frost3"
              >
                {campaigns.map(c => (
                  <option key={c.id} value={c.id}>{c.name}</option>
                ))}
              </select>
              <select
                value={timeRange}
                onChange={(e) => setTimeRange(e.target.value)}
                className="px-4 py-2 border border-nord-elevated rounded-lg bg-nord-surface text-nord-text focus:outline-none focus:ring-2 focus:ring-nord-frost3"
              >
                <option value="7d">Last 7 days</option>
                <option value="30d">Last 30 days</option>
                <option value="90d">Last 90 days</option>
              </select>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 py-6">
      {loading ? (
        <div className="text-center py-12 text-nord-text-muted">Loading analytics...</div>
      ) : (
        <>
          {/* Overview Stats */}
          <div className="grid grid-cols-4 gap-6 mb-8">
            {overviewStats.map((stat, idx) => {
              const Icon = stat.icon;
              return (
                <div key={idx} className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
                  <div className="flex items-center justify-between mb-4">
                    <div className={`w-12 h-12 rounded-lg ${stat.bgColor} flex items-center justify-center`}>
                      <Icon size={24} className={stat.iconColor} />
                    </div>
                    <span className={`text-sm font-medium ${
                      stat.trend === 'up' ? 'text-green-600' : 'text-red-600'
                    }`}>
                      {stat.change}
                    </span>
                  </div>
                  <div className="text-3xl font-bold text-nord-text mb-1">{stat.value}</div>
                  <div className="text-sm text-nord-text-muted">{stat.label}</div>
                </div>
              );
            })}
          </div>

          {/* Performance Chart */}
          <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6 mb-8">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-lg font-bold text-nord-text">Daily Performance</h2>
              <div className="flex gap-4 text-sm">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-nord-frost3 rounded-full"></div>
                  <span className="text-nord-text-muted">Sent</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-blue-500 rounded-full"></div>
                  <span className="text-nord-text-muted">Opened</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-purple-500 rounded-full"></div>
                  <span className="text-nord-text-muted">Clicked</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                  <span className="text-nord-text-muted">Replied</span>
                </div>
              </div>
            </div>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={dailyData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#4c566a" />
                <XAxis dataKey="date" stroke="#d8dee9" fontSize={12} />
                <YAxis stroke="#d8dee9" fontSize={12} />
                <Tooltip 
                  contentStyle={{ 
                    backgroundColor: '#3b4252',
                    border: '1px solid #4c566a',
                    borderRadius: '8px',
                    color: '#eceff4'
                  }}
                />
                <Line type="monotone" dataKey="sent" stroke="#88c0d0" strokeWidth={2} dot={false} />
                <Line type="monotone" dataKey="opened" stroke="#3b82f6" strokeWidth={2} dot={false} />
                <Line type="monotone" dataKey="clicked" stroke="#8b5cf6" strokeWidth={2} dot={false} />
                <Line type="monotone" dataKey="replied" stroke="#10b981" strokeWidth={2} dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </div>

          <div className="grid grid-cols-3 gap-6 mb-8">
            {/* Campaign Performance */}
            <div className="col-span-2 bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <h2 className="text-lg font-bold text-nord-text mb-6">Campaign Performance</h2>
              <div className="space-y-4">
                {campaignPerformance.map((campaign, idx) => (
                  <div key={idx} className="pb-4 border-b border-nord-elevated/30 last:border-0">
                    <div className="flex items-center justify-between mb-2">
                      <span className="font-semibold text-sm text-nord-text">{campaign.name}</span>
                      <span className="text-sm text-nord-text-muted">{campaign.rate}% reply rate</span>
                    </div>
                    <div className="flex gap-4 text-sm">
                      <span className="text-nord-text-muted">
                        <span className="font-medium text-nord-text">{campaign.sent.toLocaleString()}</span> sent
                      </span>
                      <span className="text-nord-text-muted">
                        <span className="font-medium text-nord-text">{campaign.opened.toLocaleString()}</span> opened
                      </span>
                      <span className="text-nord-text-muted">
                        <span className="font-medium text-nord-text">{campaign.replied}</span> replied
                      </span>
                    </div>
                    <div className="mt-2 w-full bg-nord-elevated rounded-full h-2">
                      <div 
                        className="bg-nord-frost3 h-2 rounded-full"
                        style={{ width: `${(campaign.replied / campaign.sent) * 100}%` }}
                      ></div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Verification Status */}
            <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
              <h2 className="text-lg font-bold text-nord-text mb-6">Lead Verification</h2>
              <ResponsiveContainer width="100%" height={200}>
                <PieChart>
                  <Pie
                    data={verificationStatus}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={80}
                    paddingAngle={2}
                    dataKey="value"
                  >
                    {verificationStatus.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                </PieChart>
              </ResponsiveContainer>
              <div className="space-y-3 mt-4">
                {verificationStatus.map((status, idx) => (
                  <div key={idx} className="flex items-center justify-between text-sm">
                    <div className="flex items-center gap-2">
                      <div 
                        className="w-3 h-3 rounded-full"
                        style={{ backgroundColor: status.color }}
                      ></div>
                      <span className="text-nord-text-muted">{status.name}</span>
                    </div>
                    <span className="font-semibold text-nord-text">{status.value.toLocaleString()}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Deliverability Health */}
          <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6 mb-8">
            <div className="flex items-center gap-2 mb-6">
              <h2 className="text-lg font-bold text-nord-text">Deliverability Health</h2>
              <span className="px-3 py-1 bg-nord-success/20 text-nord-success text-xs font-semibold rounded-full">
                Excellent
              </span>
            </div>
            <div className="grid grid-cols-4 gap-6">
              {deliverabilityScore.map((item, idx) => (
                <div key={idx}>
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm font-medium text-nord-text-muted">{item.metric}</span>
                    <span className="text-lg font-bold text-nord-text">{item.score}%</span>
                  </div>
                  <div className="w-full bg-nord-elevated rounded-full h-2">
                    <div 
                      className="h-2 rounded-full transition-all"
                      style={{ 
                        width: `${Math.min(item.score, 100)}%`,
                        backgroundColor: item.color
                      }}
                    ></div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Recommendations */}
          {deliverability && deliverability.inbox_rate < 0.95 && (
            <div className="bg-blue-50 rounded-xl border border-blue-200 p-6 mb-8">
              <h3 className="font-semibold text-blue-900 mb-3 flex items-center gap-2">
                <BarChart3 size={18} />
                Recommendations
              </h3>
              <ul className="space-y-2">
                <li className="text-sm text-blue-800 flex items-start gap-2">
                  <span className="text-blue-500 mt-1">•</span>
                  Consider warming up your inboxes to improve deliverability
                </li>
                <li className="text-sm text-blue-800 flex items-start gap-2">
                  <span className="text-blue-500 mt-1">•</span>
                  Review your email content for spam triggers
                </li>
              </ul>
            </div>
          )}

          {/* Recent Replies */}
          <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6">
            <h2 className="text-lg font-bold text-nord-text mb-6">Recent Replies</h2>
            <div className="space-y-4">
              {recentReplies.map((reply, idx) => (
                <div key={idx} className="pb-4 border-b border-nord-elevated/30 last:border-0">
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-1">
                        <span className="font-semibold text-nord-text">{reply.name}</span>
                        <span className="text-sm text-nord-text-muted">{reply.company}</span>
                        <span className={`px-2 py-0.5 text-xs font-medium rounded-full ${getSentimentColor(reply.sentiment)}`}>
                          {reply.sentiment}
                        </span>
                      </div>
                      <div className="text-sm text-nord-text-muted mb-2">{reply.email}</div>
                      <div className="text-sm bg-nord-elevated/50 rounded-lg p-3 text-nord-text">
                        &ldquo;{reply.reply}&rdquo;
                      </div>
                    </div>
                    <span className="text-xs text-nord-text-muted ml-4">{reply.time}</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </>
      )}
      </div>
    </div>
  );
}
