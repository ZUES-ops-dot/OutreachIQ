'use client';

import { useState, useEffect } from 'react';
import { 
  Briefcase, 
  GitBranch, 
  DollarSign, 
  TrendingUp,
  ExternalLink,
  Filter,
  RefreshCw,
  Building2,
  Clock,
  Zap
} from 'lucide-react';

interface Signal {
  id: string;
  signal_type: string;
  source: string;
  title: string;
  description: string | null;
  source_url: string | null;
  confidence_score: number;
  detected_at: string;
  signal_date: string | null;
  company_id: string;
  company_name: string;
  company_domain: string;
  company_logo: string | null;
  industry: string | null;
}

interface SignalStats {
  total_signals: number;
  companies_with_signals: number;
  hiring_signals: number;
  github_signals: number;
  signals_last_24h: number;
}

const SIGNAL_TYPE_CONFIG: Record<string, { icon: React.ElementType; color: string; label: string }> = {
  hiring: { icon: Briefcase, color: 'bg-blue-500', label: 'Hiring' },
  github_activity: { icon: GitBranch, color: 'bg-purple-500', label: 'GitHub Activity' },
  funding: { icon: DollarSign, color: 'bg-green-500', label: 'Funding' },
  tech_adoption: { icon: Zap, color: 'bg-yellow-500', label: 'Tech Adoption' },
  expansion: { icon: TrendingUp, color: 'bg-orange-500', label: 'Expansion' },
  product_launch: { icon: TrendingUp, color: 'bg-pink-500', label: 'Product Launch' },
};

const SOURCE_LABELS: Record<string, string> = {
  wellfound: 'Wellfound',
  github: 'GitHub',
  rss_feed: 'RSS Feed',
  twitter: 'Twitter',
  manual: 'Manual',
};

const DEMO_SIGNALS: Signal[] = [
  {
    id: 'demo-1',
    signal_type: 'hiring',
    source: 'wellfound',
    title: 'Scaling sales team for Series B launch',
    description: 'Adding 12 outbound reps across North America.',
    source_url: null,
    confidence_score: 0.86,
    detected_at: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
    signal_date: null,
    company_id: 'demo-co-1',
    company_name: 'ScaleStack',
    company_domain: 'scalestack.io',
    company_logo: null,
    industry: 'SaaS',
  },
  {
    id: 'demo-2',
    signal_type: 'funding',
    source: 'rss_feed',
    title: 'Announced $45M Series A',
    description: 'Expansion into EU with new go-to-market leader.',
    source_url: null,
    confidence_score: 0.91,
    detected_at: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    signal_date: null,
    company_id: 'demo-co-2',
    company_name: 'FreightFlow',
    company_domain: 'freightflow.ai',
    company_logo: null,
    industry: 'Logistics',
  },
  {
    id: 'demo-3',
    signal_type: 'github_activity',
    source: 'github',
    title: '5 repos shipped major releases this week',
    description: 'Active open-source roadmap and community hiring.',
    source_url: null,
    confidence_score: 0.78,
    detected_at: new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString(),
    signal_date: null,
    company_id: 'demo-co-3',
    company_name: 'LedgerLabs',
    company_domain: 'ledgerlabs.xyz',
    company_logo: null,
    industry: 'Web3',
  },
  {
    id: 'demo-4',
    signal_type: 'tech_adoption',
    source: 'twitter',
    title: 'Migrating outbound stack to custom AI copilot',
    description: 'Looking for partners experienced in compliant outreach.',
    source_url: null,
    confidence_score: 0.74,
    detected_at: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
    signal_date: null,
    company_id: 'demo-co-4',
    company_name: 'ClinicOS',
    company_domain: 'clinicos.health',
    company_logo: null,
    industry: 'Healthcare',
  },
];

const DEMO_STATS: SignalStats = {
  total_signals: DEMO_SIGNALS.length,
  companies_with_signals: new Set(DEMO_SIGNALS.map((s) => s.company_id)).size,
  hiring_signals: DEMO_SIGNALS.filter((s) => s.signal_type === 'hiring').length,
  github_signals: DEMO_SIGNALS.filter((s) => s.signal_type === 'github_activity').length,
  signals_last_24h: DEMO_SIGNALS.length,
};

function getDemoSignals(filter: string): Signal[] {
  if (filter === 'all') return DEMO_SIGNALS;
  return DEMO_SIGNALS.filter((signal) => signal.signal_type === filter);
}

function formatTimeAgo(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

function ConfidenceBadge({ score }: { score: number }) {
  const percentage = Math.round(score * 100);
  let colorClass = 'bg-nord-elevated text-nord-text-muted';
  
  if (percentage >= 80) colorClass = 'bg-nord-success/20 text-nord-success';
  else if (percentage >= 60) colorClass = 'bg-nord-warning/20 text-nord-warning';
  else if (percentage >= 40) colorClass = 'bg-orange-100 text-orange-700';
  
  return (
    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${colorClass}`}>
      {percentage}% confidence
    </span>
  );
}

function SignalCard({ signal }: { signal: Signal }) {
  const config = SIGNAL_TYPE_CONFIG[signal.signal_type] || SIGNAL_TYPE_CONFIG.hiring;
  const Icon = config.icon;

  return (
    <div className="bg-nord-surface rounded-lg border border-nord-elevated/50 p-4 hover:border-nord-elevated transition-all">
      <div className="flex items-start gap-3">
        <div className={`${config.color} p-2 rounded-lg`}>
          <Icon className="w-5 h-5 text-white" />
        </div>
        
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="font-semibold text-nord-text truncate">
              {signal.company_name}
            </span>
            <span className="text-xs text-nord-text-muted bg-nord-elevated px-2 py-0.5 rounded">
              {config.label}
            </span>
          </div>
          
          <h3 className="text-sm text-nord-text mb-2 line-clamp-2">
            {signal.title}
          </h3>
          
          {signal.description && (
            <p className="text-xs text-nord-text-muted mb-2 line-clamp-2">
              {signal.description}
            </p>
          )}
          
          <div className="flex items-center gap-3 text-xs text-nord-text-muted">
            <span className="flex items-center gap-1">
              <Clock className="w-3 h-3" />
              {formatTimeAgo(signal.detected_at)}
            </span>
            <span>via {SOURCE_LABELS[signal.source] || signal.source}</span>
            <ConfidenceBadge score={signal.confidence_score} />
          </div>
        </div>
        
        {signal.source_url && (
          <a 
            href={signal.source_url} 
            target="_blank" 
            rel="noopener noreferrer"
            className="text-nord-text-muted hover:text-nord-text"
          >
            <ExternalLink className="w-4 h-4" />
          </a>
        )}
      </div>
    </div>
  );
}

function StatsCard({ label, value, icon: Icon }: { label: string; value: number; icon: React.ElementType }) {
  return (
    <div className="bg-nord-surface rounded-lg border border-nord-elevated/50 p-4">
      <div className="flex items-center gap-3">
        <div className="bg-nord-elevated p-2 rounded-lg">
          <Icon className="w-5 h-5 text-nord-frost3" />
        </div>
        <div>
          <p className="text-2xl font-bold text-nord-text">{value}</p>
          <p className="text-sm text-nord-text-muted">{label}</p>
        </div>
      </div>
    </div>
  );
}

export default function SignalsPage() {
  const [signals, setSignals] = useState<Signal[]>([]);
  const [stats, setStats] = useState<SignalStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<string>('all');
  const [error, setError] = useState<string | null>(null);

  const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api';

  const fetchSignals = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const params = new URLSearchParams({ limit: '50' });
      if (filter !== 'all') {
        params.append('signal_type', filter);
      }
      
      const [signalsRes, statsRes] = await Promise.all([
        fetch(`${API_BASE}/signals/feed?${params}`),
        fetch(`${API_BASE}/signals/stats`),
      ]);

      if (!signalsRes.ok) throw new Error('Failed to fetch signals');
      if (!statsRes.ok) throw new Error('Failed to fetch stats');

      const signalsData = await signalsRes.json();
      const statsData = await statsRes.json();

      setSignals(signalsData.signals || []);
      setStats(statsData);
    } catch (err) {
      setSignals(getDemoSignals(filter));
      setStats(DEMO_STATS);
      setError(err instanceof Error ? err.message : 'Failed to load live signals. Showing demo data.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSignals();
  }, [filter]);

  return (
    <div className="min-h-screen bg-nord-bg">
      {/* Header */}
      <header className="bg-nord-surface border-b border-nord-elevated/50">
        <div className="max-w-6xl mx-auto px-4 py-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-nord-text">
                Buying-Intent Signal Feed
              </h1>
              <p className="text-nord-text-muted mt-1">
                Real-time hiring, funding, and tech-change alerts across industries
              </p>
            </div>
            <button
              onClick={fetchSignals}
              disabled={loading}
              className="flex items-center gap-2 px-4 py-2 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 disabled:opacity-50 font-medium"
            >
              <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </button>
          </div>
        </div>
      </header>

      <main className="max-w-6xl mx-auto px-4 py-8">
        {/* Stats */}
        {stats && (
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
            <StatsCard label="Total Signals" value={stats.total_signals} icon={Zap} />
            <StatsCard label="Companies" value={stats.companies_with_signals} icon={Building2} />
            <StatsCard label="Hiring Signals" value={stats.hiring_signals} icon={Briefcase} />
            <StatsCard label="Last 24h" value={stats.signals_last_24h} icon={Clock} />
          </div>
        )}

        {/* Filters */}
        <div className="flex items-center gap-2 mb-6">
          <Filter className="w-4 h-4 text-nord-text-muted" />
          <span className="text-sm text-nord-text-muted">Filter:</span>
          {['all', 'hiring', 'github_activity', 'funding'].map((type) => (
            <button
              key={type}
              onClick={() => setFilter(type)}
              className={`px-3 py-1 rounded-full text-sm ${
                filter === type
                  ? 'bg-nord-frost3 text-nord-bg'
                  : 'bg-nord-elevated text-nord-text-muted hover:bg-nord-elevated/80'
              }`}
            >
              {type === 'all' ? 'All' : SIGNAL_TYPE_CONFIG[type]?.label || type}
            </button>
          ))}
        </div>

        {/* Error State */}
        {error && (
          <div className="bg-nord-red/10 border border-nord-red/30 rounded-lg p-4 mb-6">
            <p className="text-nord-red">{error}</p>
            <button 
              onClick={fetchSignals}
              className="text-nord-red underline mt-2"
            >
              Try again
            </button>
          </div>
        )}

        {/* Loading State */}
        {loading && (
          <div className="flex items-center justify-center py-12">
            <RefreshCw className="w-8 h-8 text-nord-text-muted animate-spin" />
          </div>
        )}

        {/* Empty State */}
        {!loading && !error && signals.length === 0 && (
          <div className="text-center py-12">
            <Zap className="w-12 h-12 text-nord-text-muted mx-auto mb-4" />
            <h3 className="text-lg font-medium text-nord-text mb-2">No signals yet</h3>
            <p className="text-nord-text-muted mb-4">
              Signals will appear here once ingestion runs.
            </p>
            <p className="text-sm text-nord-text-muted">
              Trigger ingestion via POST /api/signals/ingest
            </p>
          </div>
        )}

        {/* Signal List */}
        {!loading && signals.length > 0 && (
          <div className="space-y-3">
            {signals.map((signal) => (
              <SignalCard key={signal.id} signal={signal} />
            ))}
          </div>
        )}

        {/* Email Capture CTA */}
        <div className="mt-12 bg-gradient-to-r from-nord-frost3 to-nord-frost2 rounded-xl p-8 text-center text-nord-bg">
          <h2 className="text-2xl font-bold mb-2">
            Get signals delivered to your inbox
          </h2>
          <p className="text-nord-bg/80 mb-6">
            Be the first to know when Web3 companies show buying intent
          </p>
          <form className="flex max-w-md mx-auto gap-2">
            <input
              type="email"
              placeholder="Enter your email"
              className="flex-1 px-4 py-3 rounded-lg text-nord-text bg-nord-surface placeholder-nord-text-muted"
            />
            <button
              type="submit"
              className="px-6 py-3 bg-nord-bg text-nord-frost3 font-semibold rounded-lg hover:bg-nord-surface"
            >
              Subscribe
            </button>
          </form>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-nord-elevated/50 mt-12 py-8">
        <div className="max-w-6xl mx-auto px-4 text-center text-nord-text-muted text-sm">
          <p>Signals are sourced from GitHub and Wellfound. Updated every 6 hours.</p>
          <p className="mt-2">
            Built by <a href="#" className="text-nord-text hover:underline">OutreachIQ</a>
          </p>
        </div>
      </footer>
    </div>
  );
}
