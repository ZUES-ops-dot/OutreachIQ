'use client';

import { useRouter } from 'next/navigation';
import { Zap, Shield, Target, BarChart3, ArrowRight, Mail, TrendingUp, Users } from 'lucide-react';
import { isAuthenticated } from '@/lib/api';

export default function LandingPage() {
  const router = useRouter();

  const navigateToDashboard = () => {
    if (isAuthenticated()) {
      router.push('/dashboard/founder');
    } else {
      router.push('/login?next=/dashboard/founder');
    }
  };

  const navigateToLeads = () => {
    if (isAuthenticated()) {
      router.push('/dashboard/leads');
    } else {
      router.push('/login?next=/dashboard/leads');
    }
  };

  return (
    <div className="min-h-screen bg-nord-bg">
      {/* Header */}
      <header className="border-b border-nord-elevated/50 bg-nord-surface/80 backdrop-blur-sm sticky top-0 z-50">
        <div className="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-nord-frost3 flex items-center justify-center">
              <Zap className="w-4 h-4 text-nord-bg" />
            </div>
            <span className="text-xl font-bold text-nord-text">OutreachIQ</span>
          </div>
          <button
            onClick={navigateToDashboard}
            className="px-4 py-2 bg-nord-frost3 text-nord-bg rounded-lg text-sm font-medium hover:bg-nord-frost2 transition-all"
          >
            Go to Dashboard
          </button>
        </div>
      </header>

      {/* Hero */}
      <section className="max-w-6xl mx-auto px-6 py-24 text-center">
        <div className="inline-flex items-center gap-2 px-4 py-2 bg-nord-frost3/10 border border-nord-frost3/30 rounded-full text-nord-frost3 text-sm mb-8">
          <TrendingUp size={16} />
          <span>Signal-driven outreach that converts</span>
        </div>
        <h1 className="text-5xl md:text-6xl font-bold text-nord-text mb-6 leading-tight">
          Fresh Leads with<br />
          <span className="text-nord-frost3">Buying Intent Signals</span>
        </h1>
        <p className="text-xl text-nord-text-muted mb-10 max-w-2xl mx-auto leading-relaxed">
          Generate leads on-demand, not from stale databases. Track hiring, funding, 
          and tech changes to reach prospects at the perfect moment.
        </p>
        <div className="flex items-center justify-center gap-4">
          <button
            onClick={navigateToLeads}
            className="px-8 py-4 bg-nord-frost3 text-nord-bg rounded-xl font-medium hover:bg-nord-frost2 transition-all inline-flex items-center gap-2 text-lg"
          >
            Start Generating Leads <ArrowRight size={20} />
          </button>
          <button
            onClick={() => router.push('/signals')}
            className="px-8 py-4 bg-nord-surface border border-nord-elevated text-nord-text rounded-xl font-medium hover:bg-nord-elevated transition-all text-lg"
          >
            View Live Signals
          </button>
        </div>
      </section>

      {/* Features */}
      <section className="max-w-6xl mx-auto px-6 py-16">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-6 hover:border-nord-frost3/30 transition-all">
            <div className="w-12 h-12 bg-nord-frost3/20 rounded-xl flex items-center justify-center mb-4">
              <Zap className="text-nord-frost3" size={24} />
            </div>
            <h3 className="font-semibold text-nord-text mb-2">Fresh Over Size</h3>
            <p className="text-sm text-nord-text-muted leading-relaxed">Generate leads on-demand, never from stale databases</p>
          </div>
          <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-6 hover:border-nord-frost3/30 transition-all">
            <div className="w-12 h-12 bg-nord-success/20 rounded-xl flex items-center justify-center mb-4">
              <Target className="text-nord-success" size={24} />
            </div>
            <h3 className="font-semibold text-nord-text mb-2">Signal-Driven</h3>
            <p className="text-sm text-nord-text-muted leading-relaxed">Track hiring, funding, and tech stack changes</p>
          </div>
          <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-6 hover:border-nord-frost3/30 transition-all">
            <div className="w-12 h-12 bg-nord-warning/20 rounded-xl flex items-center justify-center mb-4">
              <Shield className="text-nord-warning" size={24} />
            </div>
            <h3 className="font-semibold text-nord-text mb-2">Deliverability First</h3>
            <p className="text-sm text-nord-text-muted leading-relaxed">Built-in warmup, verification, and reputation management</p>
          </div>
          <div className="bg-nord-surface border border-nord-elevated/50 rounded-xl p-6 hover:border-nord-frost3/30 transition-all">
            <div className="w-12 h-12 bg-nord-purple/20 rounded-xl flex items-center justify-center mb-4">
              <BarChart3 className="text-nord-purple" size={24} />
            </div>
            <h3 className="font-semibold text-nord-text mb-2">Vertical Focus</h3>
            <p className="text-sm text-nord-text-muted leading-relaxed">Deep expertise in SaaS, Web3, and Agency verticals</p>
          </div>
        </div>
      </section>

      {/* Stats */}
      <section className="border-y border-nord-elevated/50 bg-nord-surface/50">
        <div className="max-w-6xl mx-auto px-6 py-16">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="text-center">
              <div className="text-5xl font-bold text-nord-success mb-2">98%</div>
              <div className="text-nord-text-muted">Inbox Delivery Rate</div>
            </div>
            <div className="text-center">
              <div className="text-5xl font-bold text-nord-frost3 mb-2">10x</div>
              <div className="text-nord-text-muted">Faster Than Competitors</div>
            </div>
            <div className="text-center">
              <div className="text-5xl font-bold text-nord-warning mb-2">85%</div>
              <div className="text-nord-text-muted">Email Verification Rate</div>
            </div>
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section className="max-w-6xl mx-auto px-6 py-20">
        <h2 className="text-3xl font-bold text-nord-text text-center mb-12">How It Works</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div className="text-center">
            <div className="w-16 h-16 bg-nord-frost3 rounded-2xl flex items-center justify-center mx-auto mb-4">
              <Users className="text-nord-bg" size={28} />
            </div>
            <div className="text-sm text-nord-frost3 font-medium mb-2">Step 1</div>
            <h3 className="text-lg font-semibold text-nord-text mb-2">Upload Your ICP</h3>
            <p className="text-nord-text-muted text-sm">Define your ideal customer profile or upload a CSV of target companies</p>
          </div>
          <div className="text-center">
            <div className="w-16 h-16 bg-nord-success rounded-2xl flex items-center justify-center mx-auto mb-4">
              <Target className="text-nord-bg" size={28} />
            </div>
            <div className="text-sm text-nord-success font-medium mb-2">Step 2</div>
            <h3 className="text-lg font-semibold text-nord-text mb-2">We Find Signals</h3>
            <p className="text-nord-text-muted text-sm">Our system tracks hiring, funding, and tech changes in real-time</p>
          </div>
          <div className="text-center">
            <div className="w-16 h-16 bg-nord-warning rounded-2xl flex items-center justify-center mx-auto mb-4">
              <Mail className="text-nord-bg" size={28} />
            </div>
            <div className="text-sm text-nord-warning font-medium mb-2">Step 3</div>
            <h3 className="text-lg font-semibold text-nord-text mb-2">Launch Campaigns</h3>
            <p className="text-nord-text-muted text-sm">Send personalized outreach with built-in deliverability protection</p>
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="max-w-6xl mx-auto px-6 py-16">
        <div className="bg-nord-surface border border-nord-elevated/50 rounded-2xl p-12 text-center">
          <h2 className="text-3xl font-bold text-nord-text mb-4">Ready to find your next customers?</h2>
          <p className="text-nord-text-muted mb-8 max-w-xl mx-auto">
            Stop wasting time on cold leads. Start reaching prospects with real buying intent today.
          </p>
          <button
            onClick={() => router.push('/dashboard/founder')}
            className="px-8 py-4 bg-nord-frost3 text-nord-bg rounded-xl font-medium hover:bg-nord-frost2 transition-all inline-flex items-center gap-2 text-lg"
          >
            Get Started Free <ArrowRight size={20} />
          </button>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-nord-elevated/50 bg-nord-surface/50">
        <div className="max-w-6xl mx-auto px-6 py-8 text-center text-nord-text-muted text-sm">
          © 2024 OutreachIQ. Signal-based lead generation for modern sales teams.
        </div>
      </footer>
    </div>
  );
}
