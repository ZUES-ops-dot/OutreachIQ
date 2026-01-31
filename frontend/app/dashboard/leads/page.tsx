'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { Search, Filter, Download, Plus, CheckCircle, XCircle, AlertCircle, Clock, TrendingUp, Briefcase, DollarSign, Code, RefreshCw } from 'lucide-react';
import { api, Lead } from '@/lib/api';

interface EnhancedLead {
  id: string;
  firstName: string;
  lastName: string;
  email: string;
  company: string;
  title: string;
  location: string;
  vertical: string;
  verificationStatus: string;
  confidence: number;
  signals: {
    hiring: { active: boolean; roles: string[] };
    funding: { raised: boolean; round?: string; amount?: string; date?: string };
    techStack: string[];
    growth: { score: number; indicators: string[] };
  };
  linkedin: string;
  lastContacted: string | null;
}

export default function LeadsPage() {
  const router = useRouter();
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedVertical, setSelectedVertical] = useState('all');
  const [selectedSignals, setSelectedSignals] = useState<string[]>([]);
  const [selectedLeads, setSelectedLeads] = useState<string[]>([]);
  const [viewMode, setViewMode] = useState<'table' | 'cards'>('table');
  const [isSearching, setIsSearching] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [showAddToCampaign, setShowAddToCampaign] = useState(false);
  const [campaigns, setCampaigns] = useState<Array<{id: string; name: string}>>([]);

  const [leads, setLeads] = useState<EnhancedLead[]>([]);

  const signalFilters = [
    { id: 'hiring', label: 'Recently Hiring', icon: Briefcase },
    { id: 'funding', label: 'Raised Funding', icon: DollarSign },
    { id: 'techStack', label: 'Tech Stack Match', icon: Code },
    { id: 'growth', label: 'High Growth', icon: TrendingUp }
  ];

  const verticals = [
    { id: 'all', name: 'All Industries' },
    { id: 'saas', name: 'SaaS / Software' },
    { id: 'web3', name: 'Web3 / Crypto' },
    { id: 'agency', name: 'Agency / Marketing' },
    { id: 'ecommerce', name: 'E-commerce / Retail' },
    { id: 'fintech', name: 'Fintech / Finance' },
    { id: 'healthcare', name: 'Healthcare / Medical' },
    { id: 'education', name: 'Education / EdTech' },
    { id: 'real_estate', name: 'Real Estate' },
    { id: 'consulting', name: 'Consulting' },
    { id: 'manufacturing', name: 'Manufacturing' },
    { id: 'media', name: 'Media / Entertainment' },
    { id: 'logistics', name: 'Logistics / Supply Chain' }
  ];

  useEffect(() => {
    loadLeadsFromAPI();
    loadCampaigns();
  }, []);

  const loadLeadsFromAPI = async () => {
    setIsLoading(true);
    try {
      const apiLeads = await api.getLeads();
      // Transform API leads to EnhancedLead format
      const enhancedLeads: EnhancedLead[] = apiLeads.map(lead => ({
        id: lead.id,
        firstName: lead.first_name || '',
        lastName: lead.last_name || '',
        email: lead.email,
        company: lead.company || '',
        title: lead.title || '',
        location: '',
        vertical: 'saas',
        verificationStatus: lead.verification_status,
        confidence: lead.confidence_score,
        signals: {
          hiring: { active: lead.signals?.recent_hiring || false, roles: [] },
          funding: { raised: !!lead.signals?.funding_event, round: lead.signals?.funding_event || undefined },
          techStack: lead.signals?.tech_stack || [],
          growth: { score: 80, indicators: lead.signals?.growth_indicators || [] }
        },
        linkedin: lead.linkedin_url || '',
        lastContacted: null
      }));
      setLeads(enhancedLeads);
    } catch (error) {
      console.error('Failed to load leads:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const loadCampaigns = async () => {
    try {
      const campaignList = await api.getCampaigns();
      setCampaigns(campaignList.map(c => ({ id: c.id, name: c.name })));
    } catch (error) {
      console.error('Failed to load campaigns:', error);
    }
  };

  const handleGenerateLeads = async () => {
    setIsSearching(true);
    try {
      const newLeads = await api.searchLeads({
        vertical: selectedVertical === 'all' ? 'saas' : selectedVertical,
        limit: 50
      });
      const enhancedLeads: EnhancedLead[] = newLeads.map(lead => ({
        id: lead.id,
        firstName: lead.first_name || '',
        lastName: lead.last_name || '',
        email: lead.email,
        company: lead.company || '',
        title: lead.title || '',
        location: '',
        vertical: selectedVertical === 'all' ? 'saas' : selectedVertical,
        verificationStatus: lead.verification_status,
        confidence: lead.confidence_score,
        signals: {
          hiring: { active: lead.signals?.recent_hiring || false, roles: [] },
          funding: { raised: !!lead.signals?.funding_event, round: lead.signals?.funding_event || undefined },
          techStack: lead.signals?.tech_stack || [],
          growth: { score: 80, indicators: lead.signals?.growth_indicators || [] }
        },
        linkedin: lead.linkedin_url || '',
        lastContacted: null
      }));
      setLeads(prev => [...prev, ...enhancedLeads]);
    } catch (error) {
      console.error('Failed to generate leads:', error);
    } finally {
      setIsSearching(false);
    }
  };

  const handleExport = () => {
    const selectedData = leads.filter(l => selectedLeads.includes(l.id));
    const csv = [
      ['First Name', 'Last Name', 'Email', 'Company', 'Title', 'Status', 'Confidence'].join(','),
      ...selectedData.map(l => [l.firstName, l.lastName, l.email, l.company, l.title, l.verificationStatus, l.confidence].join(','))
    ].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `leads-export-${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
  };

  const handleAddToCampaign = async (campaignId: string) => {
    try {
      await api.addLeadsToCampaign(campaignId, selectedLeads);
      setShowAddToCampaign(false);
      setSelectedLeads([]);
      alert(`Added ${selectedLeads.length} leads to campaign`);
    } catch (error) {
      console.error('Failed to add leads to campaign:', error);
      alert('Failed to add leads to campaign');
    }
  };

  const handleDeleteLead = async (leadId: string) => {
    if (!confirm('Are you sure you want to delete this lead?')) return;
    try {
      await api.deleteLead(leadId);
      setLeads(prev => prev.filter(l => l.id !== leadId));
    } catch (error) {
      console.error('Failed to delete lead:', error);
    }
  };

  const toggleSignal = (signalId: string) => {
    setSelectedSignals(prev =>
      prev.includes(signalId)
        ? prev.filter(s => s !== signalId)
        : [...prev, signalId]
    );
  };

  const toggleLead = (leadId: string) => {
    setSelectedLeads(prev =>
      prev.includes(leadId)
        ? prev.filter(id => id !== leadId)
        : [...prev, leadId]
    );
  };

  const filteredLeads = leads.filter(lead => {
    // Vertical filter
    if (selectedVertical !== 'all' && lead.vertical !== selectedVertical) return false;

    // Search query
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      const searchableText = `${lead.firstName} ${lead.lastName} ${lead.email} ${lead.company} ${lead.title}`.toLowerCase();
      if (!searchableText.includes(query)) return false;
    }

    // Signal filters
    if (selectedSignals.length > 0) {
      if (selectedSignals.includes('hiring') && !lead.signals.hiring.active) return false;
      if (selectedSignals.includes('funding') && !lead.signals.funding.raised) return false;
      if (selectedSignals.includes('growth') && lead.signals.growth.score < 80) return false;
    }

    return true;
  });

  const selectAllLeads = () => {
    if (selectedLeads.length === filteredLeads.length) {
      setSelectedLeads([]);
    } else {
      setSelectedLeads(filteredLeads.map(l => l.id));
    }
  };

  const getVerificationBadge = (status: string, confidence: number) => {
    switch(status) {
      case 'valid':
        return (
          <span className="flex items-center gap-1 px-2 py-1 bg-green-50 text-green-700 text-xs font-medium rounded-full">
            <CheckCircle size={12} />
            Valid ({confidence}%)
          </span>
        );
      case 'risky':
        return (
          <span className="flex items-center gap-1 px-2 py-1 bg-yellow-50 text-yellow-700 text-xs font-medium rounded-full">
            <AlertCircle size={12} />
            Risky ({confidence}%)
          </span>
        );
      case 'invalid':
        return (
          <span className="flex items-center gap-1 px-2 py-1 bg-red-50 text-red-700 text-xs font-medium rounded-full">
            <XCircle size={12} />
            Invalid
          </span>
        );
      default:
        return (
          <span className="flex items-center gap-1 px-2 py-1 bg-nord-elevated text-nord-text-muted text-xs font-medium rounded-full">
            <Clock size={12} />
            Pending
          </span>
        );
    }
  };

  const handleSearch = () => {
    setIsSearching(true);
    setTimeout(() => setIsSearching(false), 1000);
  };

  return (
    <div className="min-h-[calc(100vh-4rem)]">
      {/* Page Header */}
      <div className="border-b border-nord-elevated/50 bg-nord-surface/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-xl font-bold text-nord-text">Lead Discovery</h1>
              <p className="text-sm text-nord-text-muted">Find prospects with buying signals</p>
            </div>
        <div className="flex gap-3">
              <button 
                onClick={loadLeadsFromAPI}
                disabled={isLoading}
                className="px-4 py-2 border border-nord-elevated text-nord-text-muted rounded-lg hover:bg-nord-elevated/50 transition-all flex items-center gap-2 disabled:opacity-50"
              >
                <RefreshCw size={16} className={isLoading ? 'animate-spin' : ''} />
                Refresh
              </button>
              <button 
                onClick={handleExport}
                disabled={selectedLeads.length === 0}
                className="px-4 py-2 border border-nord-elevated text-nord-text-muted rounded-lg hover:bg-nord-elevated/50 transition-all flex items-center gap-2 disabled:opacity-50"
              >
                <Download size={16} />
                Export ({selectedLeads.length})
              </button>
              <div className="relative">
                <button 
                  onClick={() => setShowAddToCampaign(!showAddToCampaign)}
                  disabled={selectedLeads.length === 0}
                  className="px-4 py-2 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 transition-all flex items-center gap-2 disabled:opacity-50 font-medium"
                >
                  <Plus size={16} />
                  Add to Campaign
                </button>
                {showAddToCampaign && campaigns.length > 0 && (
                  <div className="absolute right-0 top-full mt-2 w-64 bg-nord-surface rounded-lg shadow-lg border border-nord-elevated/50 z-10">
                    <div className="p-2">
                      <p className="text-xs text-nord-text-muted px-2 py-1">Select a campaign:</p>
                      {campaigns.map(campaign => (
                        <button
                          key={campaign.id}
                          onClick={() => handleAddToCampaign(campaign.id)}
                          className="w-full text-left px-3 py-2 text-sm text-nord-text hover:bg-nord-elevated/50 rounded"
                        >
                          {campaign.name}
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 py-6">
        {/* Search Bar */}
        <div className="flex gap-3 mb-6">
          <div className="flex-1 relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-nord-text-muted" size={20} />
            <input
              type="text"
              placeholder="Search by name, company, email, or title..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleGenerateLeads()}
              className="w-full pl-12 pr-4 py-3 border border-nord-elevated rounded-lg bg-nord-surface focus:outline-none focus:ring-2 focus:ring-nord-frost3 text-nord-text placeholder:text-nord-text-muted"
            />
          </div>
          <button
            onClick={handleGenerateLeads}
            disabled={isSearching}
            className="px-6 py-3 bg-nord-frost3 text-nord-bg rounded-lg hover:bg-nord-frost2 transition-all disabled:opacity-50 font-medium"
          >
            {isSearching ? 'Generating...' : 'Generate Leads'}
          </button>
        </div>

      <div className="flex gap-6">
        {/* Filters Sidebar */}
        <div className="w-64 flex-shrink-0">
          <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 p-6 sticky top-6">
            <div className="flex items-center gap-2 mb-4">
              <Filter size={18} />
              <h3 className="font-bold">Filters</h3>
            </div>

            {/* Vertical Filter */}
            <div className="mb-6">
              <label className="block text-sm font-semibold mb-3 text-nord-text-secondary">Vertical</label>
              <div className="space-y-2">
                {verticals.map(v => (
                  <button
                    key={v.id}
                    onClick={() => setSelectedVertical(v.id)}
                    className={`w-full px-3 py-2 text-left text-sm rounded-lg transition-all ${
                      selectedVertical === v.id
                        ? 'bg-nord-frost3 text-white'
                        : 'bg-nord-elevated/50 text-nord-text-secondary hover:bg-nord-elevated'
                    }`}
                  >
                    {v.name}
                  </button>
                ))}
              </div>
            </div>

            {/* Signal Filters */}
            <div>
              <label className="block text-sm font-semibold mb-3 text-nord-text-secondary">Buying Signals</label>
              <div className="space-y-2">
                {signalFilters.map(signal => {
                  const Icon = signal.icon;
                  return (
                    <button
                      key={signal.id}
                      onClick={() => toggleSignal(signal.id)}
                      className={`w-full px-3 py-2 text-left text-sm rounded-lg transition-all flex items-center gap-2 ${
                        selectedSignals.includes(signal.id)
                          ? 'bg-nord-frost3 text-white'
                          : 'bg-nord-elevated/50 text-nord-text-secondary hover:bg-nord-elevated'
                      }`}
                    >
                      <Icon size={14} />
                      {signal.label}
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Results Count */}
            <div className="mt-6 pt-6 border-t border-nord-elevated/50">
              <div className="text-sm text-nord-text-muted">
                <span className="font-bold text-nord-text">{filteredLeads.length}</span> leads found
              </div>
            </div>
          </div>
        </div>

        {/* Results */}
        <div className="flex-1">
          <div className="bg-nord-surface rounded-xl border border-nord-elevated/50 overflow-hidden">
            {/* Table Header */}
            <div className="px-6 py-4 border-b border-nord-elevated/50 flex items-center justify-between">
              <div className="flex items-center gap-4">
                <input
                  type="checkbox"
                  checked={selectedLeads.length === filteredLeads.length && filteredLeads.length > 0}
                  onChange={selectAllLeads}
                  className="w-4 h-4 rounded border-nord-elevated"
                />
                <span className="text-sm font-medium text-nord-text-muted">
                  {selectedLeads.length} selected
                </span>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => setViewMode('table')}
                  className={`px-3 py-1 text-sm rounded ${
                    viewMode === 'table' ? 'bg-nord-frost3 text-white' : 'text-nord-text-muted hover:bg-nord-elevated/50'
                  }`}
                >
                  Table
                </button>
                <button
                  onClick={() => setViewMode('cards')}
                  className={`px-3 py-1 text-sm rounded ${
                    viewMode === 'cards' ? 'bg-nord-frost3 text-white' : 'text-nord-text-muted hover:bg-nord-elevated/50'
                  }`}
                >
                  Cards
                </button>
              </div>
            </div>

            {/* Leads List */}
            <div className="divide-y divide-nord-elevated/30">
              {filteredLeads.map(lead => (
                <div
                  key={lead.id}
                  className="px-6 py-4 hover:bg-nord-elevated/50 transition-all cursor-pointer"
                >
                  <div className="flex items-start gap-4">
                    <input
                      type="checkbox"
                      checked={selectedLeads.includes(lead.id)}
                      onChange={() => toggleLead(lead.id)}
                      className="w-4 h-4 rounded border-nord-elevated mt-1"
                    />
                    
                    <div className="flex-1">
                      <div className="flex items-start justify-between mb-2">
                        <div>
                          <div className="flex items-center gap-3">
                            <h3 className="font-semibold text-nord-text">
                              {lead.firstName} {lead.lastName}
                            </h3>
                            {getVerificationBadge(lead.verificationStatus, lead.confidence)}
                          </div>
                          <p className="text-sm text-nord-text-muted mt-1">
                            {lead.title} at {lead.company}
                          </p>
                          <p className="text-sm text-nord-text-muted">{lead.email}</p>
                        </div>
                        <span className="text-xs text-nord-text-muted">{lead.location}</span>
                      </div>

                      {/* Signals */}
                      <div className="flex flex-wrap gap-2 mt-3">
                        {lead.signals.hiring.active && (
                          <div className="px-3 py-1 bg-blue-50 text-blue-700 text-xs rounded-full flex items-center gap-1">
                            <Briefcase size={12} />
                            Hiring {lead.signals.hiring.roles.length} roles
                          </div>
                        )}
                        {lead.signals.funding.raised && (
                          <div className="px-3 py-1 bg-green-50 text-green-700 text-xs rounded-full flex items-center gap-1">
                            <DollarSign size={12} />
                            {lead.signals.funding.round} - {lead.signals.funding.amount}
                          </div>
                        )}
                        {lead.signals.growth.score >= 80 && (
                          <div className="px-3 py-1 bg-purple-50 text-purple-700 text-xs rounded-full flex items-center gap-1">
                            <TrendingUp size={12} />
                            {lead.signals.growth.score}% growth score
                          </div>
                        )}
                        {lead.signals.techStack.length > 0 && (
                          <div className="px-3 py-1 bg-orange-50 text-orange-700 text-xs rounded-full flex items-center gap-1">
                            <Code size={12} />
                            {lead.signals.techStack.slice(0, 2).join(', ')}
                            {lead.signals.techStack.length > 2 && ` +${lead.signals.techStack.length - 2}`}
                          </div>
                        )}
                      </div>

                      {/* Growth Indicators */}
                      {lead.signals.growth.indicators.length > 0 && (
                        <div className="mt-3 text-xs text-nord-text-muted">
                          <span className="font-medium">Growth indicators:</span>{' '}
                          {lead.signals.growth.indicators.join(' • ')}
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {filteredLeads.length === 0 && (
              <div className="px-6 py-12 text-center">
                <Search size={48} className="mx-auto mb-4 text-nord-text-muted" />
                <p className="text-nord-text-muted mb-2">No leads found</p>
                <p className="text-sm text-nord-text-muted">Try adjusting your filters or search query</p>
              </div>
            )}
          </div>
        </div>
      </div>
      </div>
    </div>
  );
}
