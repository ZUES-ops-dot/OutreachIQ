'use client';

import { useState, useEffect } from 'react';
import { Users, Mail, BarChart3, Settings, Shield, AlertTriangle, CheckCircle, Clock, Trash2, Edit, RefreshCw } from 'lucide-react';
import { api } from '@/lib/api';

interface User {
  id: string;
  email: string;
  name: string;
  role: string;
  created_at: string;
  last_login: string | null;
}

interface SystemStats {
  totalUsers: number;
  activeUsers: number;
  totalLeads: number;
  totalCampaigns: number;
  activeCampaigns: number;
  emailsSentToday: number;
  pendingJobs: number;
  failedJobs: number;
}

export default function AdminDashboard() {
  const [users, setUsers] = useState<User[]>([]);
  const [stats, setStats] = useState<SystemStats>({
    totalUsers: 0,
    activeUsers: 0,
    totalLeads: 0,
    totalCampaigns: 0,
    activeCampaigns: 0,
    emailsSentToday: 0,
    pendingJobs: 0,
    failedJobs: 0,
  });
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'overview' | 'users' | 'jobs' | 'settings'>('overview');

  useEffect(() => {
    loadDashboardData();
  }, []);

  const loadDashboardData = async () => {
    setLoading(true);
    try {
      // Load overview stats from API
      const overview = await api.getOverview();
      
      // Load campaigns to count
      const campaigns = await api.getCampaigns();
      const activeCampaigns = campaigns.filter(c => c.status === 'active').length;
      
      setStats({
        totalUsers: 1, // Current user - would need admin API endpoint for full user list
        activeUsers: 1,
        totalLeads: overview.total_leads || 0,
        totalCampaigns: overview.total_campaigns || campaigns.length,
        activeCampaigns: overview.active_campaigns || activeCampaigns,
        emailsSentToday: overview.total_sent || 0,
        pendingJobs: 0,
        failedJobs: 0,
      });

      // For now, show current user - admin API would provide full user list
      const currentUser = await api.getCurrentUser().catch(() => null);
      if (currentUser) {
        setUsers([{
          id: currentUser.id,
          email: currentUser.email,
          name: currentUser.name,
          role: currentUser.role,
          created_at: new Date().toISOString().split('T')[0],
          last_login: new Date().toISOString().split('T')[0]
        }]);
      }
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      setLoading(false);
    }
  };

  const getRoleBadge = (role: string) => {
    if (role === 'admin') {
      return (
        <span className="flex items-center gap-1 px-2 py-1 bg-purple-50 text-purple-700 text-xs font-medium rounded-full">
          <Shield size={12} />
          Admin
        </span>
      );
    }
    return (
      <span className="px-2 py-1 bg-stone-100 text-stone-600 text-xs font-medium rounded-full">
        User
      </span>
    );
  };

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center">
        <div className="text-stone-600">Loading...</div>
      </div>
    );
  }

  return (
    <div className="p-8">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-stone-900">Admin Dashboard</h1>
          <p className="text-stone-600 mt-1">System management and monitoring</p>
        </div>
        <div className="flex items-center gap-2 px-3 py-1.5 bg-purple-50 text-purple-700 rounded-full text-sm font-medium">
          <Shield size={16} />
          Admin Access
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-6 border-b border-stone-200">
        {[
          { id: 'overview', label: 'Overview', icon: BarChart3 },
          { id: 'users', label: 'Users', icon: Users },
          { id: 'jobs', label: 'Background Jobs', icon: Clock },
          { id: 'settings', label: 'Settings', icon: Settings },
        ].map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as typeof activeTab)}
              className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-all ${
                activeTab === tab.id
                  ? 'border-stone-900 text-stone-900'
                  : 'border-transparent text-stone-500 hover:text-stone-700'
              }`}
            >
              <Icon size={18} />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Stats Grid */}
          <div className="grid grid-cols-4 gap-6">
            <div className="bg-white rounded-xl border border-stone-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <Users className="text-blue-600" size={24} />
                <span className="text-sm text-stone-500">Users</span>
              </div>
              <div className="text-3xl font-bold">{stats.totalUsers}</div>
              <div className="text-sm text-stone-600">{stats.activeUsers} active</div>
            </div>

            <div className="bg-white rounded-xl border border-stone-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <Mail className="text-green-600" size={24} />
                <span className="text-sm text-stone-500">Emails Today</span>
              </div>
              <div className="text-3xl font-bold">{stats.emailsSentToday}</div>
              <div className="text-sm text-stone-600">sent successfully</div>
            </div>

            <div className="bg-white rounded-xl border border-stone-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <Clock className="text-yellow-600" size={24} />
                <span className="text-sm text-stone-500">Pending Jobs</span>
              </div>
              <div className="text-3xl font-bold">{stats.pendingJobs}</div>
              <div className="text-sm text-stone-600">in queue</div>
            </div>

            <div className="bg-white rounded-xl border border-stone-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <AlertTriangle className="text-red-600" size={24} />
                <span className="text-sm text-stone-500">Failed Jobs</span>
              </div>
              <div className="text-3xl font-bold">{stats.failedJobs}</div>
              <div className="text-sm text-stone-600">need attention</div>
            </div>
          </div>

          {/* System Health */}
          <div className="bg-white rounded-xl border border-stone-200 p-6">
            <h3 className="font-bold mb-4">System Health</h3>
            <div className="grid grid-cols-3 gap-6">
              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-stone-600">Database</span>
                  <span className="flex items-center gap-1 text-green-600 text-sm">
                    <CheckCircle size={14} />
                    Healthy
                  </span>
                </div>
                <div className="w-full bg-stone-100 rounded-full h-2">
                  <div className="bg-green-500 h-2 rounded-full" style={{ width: '15%' }}></div>
                </div>
                <div className="text-xs text-stone-500 mt-1">15% storage used</div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-stone-600">API Server</span>
                  <span className="flex items-center gap-1 text-green-600 text-sm">
                    <CheckCircle size={14} />
                    Online
                  </span>
                </div>
                <div className="w-full bg-stone-100 rounded-full h-2">
                  <div className="bg-green-500 h-2 rounded-full" style={{ width: '32%' }}></div>
                </div>
                <div className="text-xs text-stone-500 mt-1">32% CPU usage</div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-stone-600">Job Worker</span>
                  <span className="flex items-center gap-1 text-green-600 text-sm">
                    <CheckCircle size={14} />
                    Running
                  </span>
                </div>
                <div className="w-full bg-stone-100 rounded-full h-2">
                  <div className="bg-green-500 h-2 rounded-full" style={{ width: '45%' }}></div>
                </div>
                <div className="text-xs text-stone-500 mt-1">45% memory used</div>
              </div>
            </div>
          </div>

          {/* Recent Activity */}
          <div className="bg-white rounded-xl border border-stone-200 p-6">
            <h3 className="font-bold mb-4">Recent Activity</h3>
            <div className="space-y-3">
              {[
                { action: 'User registered', user: 'emily@agency.co', time: '5 minutes ago' },
                { action: 'Campaign started', user: 'sarah@company.com', time: '15 minutes ago' },
                { action: 'Leads imported', user: 'john@startup.io', time: '1 hour ago' },
                { action: 'Email account added', user: 'admin@outreachiq.io', time: '2 hours ago' },
              ].map((activity, idx) => (
                <div key={idx} className="flex items-center justify-between py-2 border-b border-stone-100 last:border-0">
                  <div>
                    <span className="font-medium text-stone-900">{activity.action}</span>
                    <span className="text-stone-500 ml-2">by {activity.user}</span>
                  </div>
                  <span className="text-sm text-stone-500">{activity.time}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Users Tab */}
      {activeTab === 'users' && (
        <div className="bg-white rounded-xl border border-stone-200">
          <div className="px-6 py-4 border-b border-stone-200 flex items-center justify-between">
            <h3 className="font-bold">User Management</h3>
            <button className="px-4 py-2 bg-stone-900 text-white rounded-lg hover:bg-stone-800 transition-all text-sm">
              Add User
            </button>
          </div>
          <table className="w-full">
            <thead className="bg-stone-50 border-b border-stone-200">
              <tr>
                <th className="px-6 py-3 text-left text-sm font-semibold text-stone-900">User</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-stone-900">Role</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-stone-900">Created</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-stone-900">Last Login</th>
                <th className="px-6 py-3 text-left text-sm font-semibold text-stone-900">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-100">
              {users.map((user) => (
                <tr key={user.id} className="hover:bg-stone-50">
                  <td className="px-6 py-4">
                    <div className="font-medium text-stone-900">{user.name}</div>
                    <div className="text-sm text-stone-500">{user.email}</div>
                  </td>
                  <td className="px-6 py-4">{getRoleBadge(user.role)}</td>
                  <td className="px-6 py-4 text-sm text-stone-600">{user.created_at}</td>
                  <td className="px-6 py-4 text-sm text-stone-600">
                    {user.last_login || <span className="text-stone-400">Never</span>}
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex gap-2">
                      <button className="p-2 hover:bg-stone-100 rounded-lg transition-all">
                        <Edit size={16} className="text-stone-600" />
                      </button>
                      <button className="p-2 hover:bg-red-50 rounded-lg transition-all">
                        <Trash2 size={16} className="text-red-600" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Jobs Tab */}
      {activeTab === 'jobs' && (
        <div className="space-y-6">
          <div className="grid grid-cols-3 gap-6">
            <div className="bg-white rounded-xl border border-stone-200 p-6">
              <div className="text-sm text-stone-600 mb-2">Pending</div>
              <div className="text-3xl font-bold text-yellow-600">{stats.pendingJobs}</div>
            </div>
            <div className="bg-white rounded-xl border border-stone-200 p-6">
              <div className="text-sm text-stone-600 mb-2">Processing</div>
              <div className="text-3xl font-bold text-blue-600">3</div>
            </div>
            <div className="bg-white rounded-xl border border-stone-200 p-6">
              <div className="text-sm text-stone-600 mb-2">Failed</div>
              <div className="text-3xl font-bold text-red-600">{stats.failedJobs}</div>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-stone-200">
            <div className="px-6 py-4 border-b border-stone-200 flex items-center justify-between">
              <h3 className="font-bold">Recent Jobs</h3>
              <button className="px-4 py-2 border border-stone-300 rounded-lg hover:bg-stone-50 transition-all text-sm">
                Retry Failed
              </button>
            </div>
            <div className="divide-y divide-stone-100">
              {[
                { type: 'SendEmail', status: 'completed', time: '2 min ago' },
                { type: 'VerifyEmail', status: 'processing', time: '3 min ago' },
                { type: 'SendEmail', status: 'completed', time: '5 min ago' },
                { type: 'ProcessCampaign', status: 'failed', time: '10 min ago' },
                { type: 'WarmupEmail', status: 'pending', time: '12 min ago' },
              ].map((job, idx) => (
                <div key={idx} className="px-6 py-4 flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <span className="font-medium text-stone-900">{job.type}</span>
                    <span className={`px-2 py-1 text-xs font-medium rounded-full ${
                      job.status === 'completed' ? 'bg-green-50 text-green-700' :
                      job.status === 'processing' ? 'bg-blue-50 text-blue-700' :
                      job.status === 'failed' ? 'bg-red-50 text-red-700' :
                      'bg-yellow-50 text-yellow-700'
                    }`}>
                      {job.status}
                    </span>
                  </div>
                  <span className="text-sm text-stone-500">{job.time}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && (
        <div className="space-y-6">
          <div className="bg-white rounded-xl border border-stone-200 p-6">
            <h3 className="font-bold mb-4">Email Settings</h3>
            <div className="grid grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Default Daily Limit</label>
                <input
                  type="number"
                  defaultValue={50}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Maximum emails per inbox per day</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Warmup Duration (days)</label>
                <input
                  type="number"
                  defaultValue={30}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Days to fully warm up new inboxes</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Warmup Start Volume</label>
                <input
                  type="number"
                  defaultValue={5}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Initial daily emails during warmup</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Warmup Increment</label>
                <input
                  type="number"
                  defaultValue={2}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Daily volume increase during warmup</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Send Delay (seconds)</label>
                <input
                  type="number"
                  defaultValue={60}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Minimum delay between emails</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Bounce Threshold (%)</label>
                <input
                  type="number"
                  defaultValue={5}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Pause sending if bounce rate exceeds</p>
              </div>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-stone-200 p-6">
            <h3 className="font-bold mb-4">Lead Generation Settings</h3>
            <div className="grid grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Default Lead Limit</label>
                <input
                  type="number"
                  defaultValue={100}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Max leads per search request</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Verification Confidence Threshold</label>
                <input
                  type="number"
                  defaultValue={70}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Minimum confidence score for valid leads</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Auto-verify New Leads</label>
                <select className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900">
                  <option value="true">Enabled</option>
                  <option value="false">Disabled</option>
                </select>
                <p className="text-xs text-stone-500 mt-1">Automatically verify emails on import</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Duplicate Detection</label>
                <select className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900">
                  <option value="email">By Email</option>
                  <option value="domain">By Domain</option>
                  <option value="both">Both</option>
                </select>
                <p className="text-xs text-stone-500 mt-1">How to detect duplicate leads</p>
              </div>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-stone-200 p-6">
            <h3 className="font-bold mb-4">Campaign Settings</h3>
            <div className="grid grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Default Follow-up Delay (days)</label>
                <input
                  type="number"
                  defaultValue={3}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Days between follow-up emails</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Max Follow-ups</label>
                <input
                  type="number"
                  defaultValue={3}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Maximum follow-up emails per lead</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Send Window Start</label>
                <input
                  type="time"
                  defaultValue="09:00"
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Earliest time to send emails</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Send Window End</label>
                <input
                  type="time"
                  defaultValue="17:00"
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Latest time to send emails</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Send on Weekends</label>
                <select className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900">
                  <option value="false">No</option>
                  <option value="true">Yes</option>
                </select>
                <p className="text-xs text-stone-500 mt-1">Allow sending on Saturday/Sunday</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Timezone</label>
                <select className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900">
                  <option value="America/New_York">Eastern (ET)</option>
                  <option value="America/Chicago">Central (CT)</option>
                  <option value="America/Denver">Mountain (MT)</option>
                  <option value="America/Los_Angeles">Pacific (PT)</option>
                  <option value="UTC">UTC</option>
                </select>
                <p className="text-xs text-stone-500 mt-1">Timezone for send windows</p>
              </div>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-stone-200 p-6">
            <h3 className="font-bold mb-4">API Configuration</h3>
            <div className="grid grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">API Rate Limit (requests/min)</label>
                <input
                  type="number"
                  defaultValue={100}
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Max API requests per minute</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 mb-2">Webhook URL</label>
                <input
                  type="url"
                  placeholder="https://your-webhook.com/endpoint"
                  className="w-full px-4 py-2 border border-stone-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-900"
                />
                <p className="text-xs text-stone-500 mt-1">Receive events via webhook</p>
              </div>
            </div>
          </div>

          <div className="flex gap-4">
            <button className="px-6 py-3 bg-stone-900 text-white rounded-lg hover:bg-stone-800 transition-all">
              Save Settings
            </button>
            <button className="px-6 py-3 border border-stone-300 rounded-lg hover:bg-stone-50 transition-all">
              Reset to Defaults
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
