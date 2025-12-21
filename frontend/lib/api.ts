const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api';

// ============================================================================
// AUTH TYPES
// ============================================================================

export interface User {
  id: string;
  email: string;
  name: string;
  role: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface LoginParams {
  email: string;
  password: string;
}

export interface RegisterParams {
  email: string;
  password: string;
  name: string;
}

// ============================================================================
// BILLING TYPES
// ============================================================================

export interface PricingTier {
  id: string;
  name: string;
  price_monthly: number;
  price_yearly: number;
  leads_per_month: number;
  inboxes: number;
  emails_per_month: number;
  features: string[];
}

export interface UsageSummary {
  leads_used: number;
  leads_limit: number;
  emails_sent: number;
  emails_limit: number;
  period_start: string;
  period_end: string;
}

export interface Subscription {
  plan_tier: string;
  monthly_lead_limit: number;
  monthly_email_limit: number;
  stripe_subscription_id: string | null;
}

// ============================================================================
// TYPE DEFINITIONS - Matching Rust Backend Models
// ============================================================================

export interface LeadSignals {
  recent_hiring: boolean;
  funding_event: string | null;
  tech_stack: string[];
  company_size: string | null;
  growth_indicators: string[];
}

export interface Lead {
  id: string;
  email: string;
  first_name: string | null;
  last_name: string | null;
  company: string | null;
  title: string | null;
  linkedin_url: string | null;
  verification_status: 'pending' | 'valid' | 'invalid' | 'risky';
  confidence_score: number;
  signals: LeadSignals;
  created_at: string;
  verified_at: string | null;
}

export interface Campaign {
  id: string;
  name: string;
  vertical: string;
  status: 'draft' | 'active' | 'paused' | 'completed';
  total_leads: number;
  sent: number;
  opened: number;
  clicked: number;
  replied: number;
  created_at: string;
  started_at: string | null;
}

export interface EmailAccount {
  id: string;
  email: string;
  provider: string;
  smtp_host: string;
  smtp_port: number;
  warmup_status: string;
  daily_limit: number;
  sent_today: number;
  health_score: number;
  created_at: string;
}

export interface LeadSearchParams {
  vertical: string;
  role?: string;
  company_size?: string;
  signals?: string[];
  limit?: number;
}

export interface CreateCampaignParams {
  name: string;
  vertical: string;
  email_subject?: string;
  email_body?: string;
  daily_limit?: number;
}

export interface CreateEmailAccountParams {
  email: string;
  provider: string;
  smtp_host: string;
  smtp_port: number;
  smtp_username: string;
  smtp_password: string;
}

export interface OverviewStats {
  total_sent: number;
  total_opened: number;
  total_clicked: number;
  total_replied: number;
  open_rate: number;
  click_rate: number;
  reply_rate: number;
}

export interface CampaignAnalytics {
  summary: {
    total_leads: number;
    sent: number;
    opened: number;
    clicked: number;
    replied: number;
    bounces: number;
    open_rate: number;
    click_rate: number;
    reply_rate: number;
  };
  daily_stats: Array<{
    date: string;
    sent: number;
    opened: number;
    replied: number;
  }>;
  recent_replies: Array<{
    lead_email: string;
    reply_body: string;
    replied_at: string;
  }>;
}

export interface LeadAnalytics {
  total_leads: number;
  verified_leads: number;
  verification_breakdown: {
    valid: number;
    invalid: number;
    risky: number;
    pending: number;
  };
}

export interface DeliverabilityMetrics {
  inbox_rate: number;
  bounce_rate: number;
  spam_rate: number;
  average_health_score: number;
}

export interface WarmupStats {
  health_score: number;
  daily_volume: number;
  inbox_rate: number;
  spam_rate: number;
  bounce_rate: number;
  warmup_progress: number;
}

// ============================================================================
// FOUNDER DASHBOARD TYPES
// ============================================================================

export interface DashboardOverview {
  total_campaigns: number;
  active_campaigns: number;
  paused_campaigns: number;
  total_sent: number;
  total_replies: number;
  total_meetings: number;
  cost_per_meeting: number;
  cost_per_meeting_trend: number;
}

export interface InboxHealthCard {
  id: string;
  email: string;
  provider: string;
  health_status: 'healthy' | 'warning' | 'danger';
  health_score: number;
  spam_rate: number;
  reply_rate: number;
  bounce_rate: number;
  daily_limit: number;
  sent_today: number;
}

export interface CampaignCard {
  id: string;
  name: string;
  status: string;
  health_status: 'healthy' | 'warning' | 'danger';
  auto_paused: boolean;
  auto_pause_reason: string | null;
  total_leads: number;
  sent: number;
  replied: number;
  meetings_booked: number;
  reply_rate: number;
  created_at: string;
}

export interface ReplyCard {
  id: string;
  from_email: string;
  from_name: string | null;
  subject: string | null;
  body_preview: string;
  intent: 'interested' | 'maybe_later' | 'objection' | 'negative' | 'auto_reply';
  intent_confidence: number;
  campaign_id: string | null;
  campaign_name: string | null;
  received_at: string;
  is_read: boolean;
  is_actioned: boolean;
}

export interface FounderDashboardData {
  overview: DashboardOverview;
  campaigns: CampaignCard[];
  inboxes: InboxHealthCard[];
  recent_replies: ReplyCard[];
  unread_count: number;
  action_required_count: number;
}

export interface AutoPauseEvent {
  id: string;
  campaign_id: string | null;
  campaign_name: string | null;
  pause_reason: string;
  pause_reason_detail: string | null;
  created_at: string;
  is_resolved: boolean;
}

export interface CostPerMeetingStats {
  current_period: number;
  previous_period: number;
  trend_percentage: number;
  total_cost: number;
  total_meetings: number;
  by_campaign: CampaignCostSummary[];
}

export interface CampaignCostSummary {
  campaign_id: string;
  campaign_name: string;
  total_cost: number;
  meetings_booked: number;
  cost_per_meeting: number;
}

export interface WorkspaceSettings {
  auto_pause_enabled: boolean;
  spam_rate_threshold: number;
  reply_drop_threshold: number;
  bounce_rate_threshold: number;
  google_daily_limit: number;
  outlook_daily_limit: number;
  zoho_daily_limit: number;
  notification_email: string | null;
  slack_webhook_url: string | null;
}

export interface Meeting {
  id: string;
  campaign_id: string | null;
  campaign_name: string | null;
  lead_id: string | null;
  lead_email: string | null;
  title: string | null;
  scheduled_at: string | null;
  status: string;
  outcome: string | null;
}

export interface CompanySignals {
  company_name: string;
  domain: string;
  hiring_signal: boolean;
  funding_signal: string | null;
  tech_stack: string[];
  growth_score: number;
  employee_count: string | null;
  industry: string | null;
}

// ============================================================================
// AUTH TOKEN MANAGEMENT
// ============================================================================

const TOKEN_KEY = 'outreachiq_token';
const USER_KEY = 'outreachiq_user';

export function getStoredToken(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem(TOKEN_KEY);
}

export function getStoredUser(): User | null {
  if (typeof window === 'undefined') return null;
  const user = localStorage.getItem(USER_KEY);
  return user ? JSON.parse(user) : null;
}

export function setAuthData(token: string, user: User): void {
  if (typeof window === 'undefined') return;
  localStorage.setItem(TOKEN_KEY, token);
  localStorage.setItem(USER_KEY, JSON.stringify(user));
}

export function clearAuthData(): void {
  if (typeof window === 'undefined') return;
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
}

export function isAuthenticated(): boolean {
  return !!getStoredToken();
}

// ============================================================================
// API CLIENT CLASS
// ============================================================================

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private getAuthHeaders(): Record<string, string> {
    const token = getStoredToken();
    return token ? { Authorization: `Bearer ${token}` } : {};
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
    requiresAuth: boolean = true
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(requiresAuth ? this.getAuthHeaders() : {}),
      ...(options.headers as Record<string, string> || {}),
    };

    const config: RequestInit = {
      ...options,
      headers,
    };

    try {
      const response = await fetch(url, config);
      
      // Handle 401 Unauthorized - clear auth and redirect
      if (response.status === 401) {
        clearAuthData();
        if (typeof window !== 'undefined' && !endpoint.startsWith('/auth/')) {
          window.location.href = '/login';
        }
        throw new Error('Authentication required');
      }

      if (!response.ok) {
        const error = await response.json().catch(() => ({}));
        throw new Error(error.error || error.message || `HTTP error! status: ${response.status}`);
      }

      if (response.status === 204) {
        return {} as T;
      }

      return await response.json();
    } catch (error) {
      console.error('API request failed:', error);
      throw error;
    }
  }

  // ============================================================================
  // AUTH ENDPOINTS
  // ============================================================================

  async login(params: LoginParams): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify(params),
    }, false);
    setAuthData(response.token, response.user);
    return response;
  }

  async register(params: RegisterParams): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify(params),
    }, false);
    setAuthData(response.token, response.user);
    return response;
  }

  async getCurrentUser(): Promise<User> {
    return this.request<User>('/auth/me');
  }

  async refreshToken(): Promise<{ token: string }> {
    const response = await this.request<{ token: string }>('/auth/refresh', {
      method: 'POST',
    });
    const user = getStoredUser();
    if (user) {
      setAuthData(response.token, user);
    }
    return response;
  }

  logout(): void {
    clearAuthData();
    if (typeof window !== 'undefined') {
      window.location.href = '/login';
    }
  }

  // ============================================================================
  // BILLING ENDPOINTS
  // ============================================================================

  async getPricing(): Promise<PricingTier[]> {
    return this.request<PricingTier[]>('/billing/pricing', {}, false);
  }

  async createCheckout(tierId: string, billingCycle: 'monthly' | 'yearly'): Promise<{ checkout_url: string; session_id: string }> {
    return this.request('/billing/checkout', {
      method: 'POST',
      body: JSON.stringify({ tier_id: tierId, billing_cycle: billingCycle }),
    });
  }

  async createPortalSession(): Promise<{ url: string }> {
    return this.request('/billing/portal', { method: 'POST' });
  }

  async getSubscription(): Promise<Subscription> {
    return this.request<Subscription>('/billing/subscription');
  }

  async getUsage(): Promise<UsageSummary> {
    return this.request<UsageSummary>('/billing/usage');
  }

  // ============================================================================
  // LEADS ENDPOINTS
  // ============================================================================

  async getLeads(params?: { limit?: number; offset?: number; vertical?: string }): Promise<Lead[]> {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.vertical) queryParams.append('vertical', params.vertical);
    const query = queryParams.toString();
    return this.request<Lead[]>(`/leads${query ? `?${query}` : ''}`);
  }

  async getLeadById(id: string): Promise<Lead> {
    return this.request<Lead>(`/leads/${id}`);
  }

  async searchLeads(query: LeadSearchParams): Promise<Lead[]> {
    return this.request<Lead[]>('/leads/search', {
      method: 'POST',
      body: JSON.stringify(query),
    });
  }

  async verifyLeads(emails: string[]): Promise<Array<{ email: string; status: string; confidence: number }>> {
    return this.request('/leads/verify', {
      method: 'POST',
      body: JSON.stringify(emails),
    });
  }

  async getSignals(domain: string): Promise<CompanySignals> {
    return this.request<CompanySignals>(`/leads/signals/${domain}`);
  }

  async deleteLead(id: string): Promise<void> {
    return this.request(`/leads/${id}`, { method: 'DELETE' });
  }

  // ============================================================================
  // CAMPAIGNS ENDPOINTS
  // ============================================================================

  async getCampaigns(): Promise<Campaign[]> {
    return this.request<Campaign[]>('/campaigns');
  }

  async getCampaignById(id: string): Promise<Campaign> {
    return this.request<Campaign>(`/campaigns/${id}`);
  }

  async createCampaign(campaign: CreateCampaignParams): Promise<Campaign> {
    return this.request<Campaign>('/campaigns', {
      method: 'POST',
      body: JSON.stringify(campaign),
    });
  }

  async updateCampaign(id: string, updates: Partial<Campaign>): Promise<Campaign> {
    return this.request<Campaign>(`/campaigns/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
  }

  async deleteCampaign(id: string): Promise<void> {
    return this.request(`/campaigns/${id}`, { method: 'DELETE' });
  }

  async startCampaign(id: string): Promise<Campaign> {
    return this.request<Campaign>(`/campaigns/${id}/start`, { method: 'POST' });
  }

  async pauseCampaign(id: string): Promise<Campaign> {
    return this.request<Campaign>(`/campaigns/${id}/pause`, { method: 'POST' });
  }

  async addLeadsToCampaign(campaignId: string, leadIds: string[]): Promise<void> {
    return this.request(`/campaigns/${campaignId}/leads`, {
      method: 'POST',
      body: JSON.stringify({ lead_ids: leadIds }),
    });
  }

  async getCampaignLeads(campaignId: string): Promise<Lead[]> {
    return this.request<Lead[]>(`/campaigns/${campaignId}/leads`);
  }

  // ============================================================================
  // ANALYTICS ENDPOINTS
  // ============================================================================

  async getAnalyticsOverview(params?: { start_date?: string; end_date?: string }): Promise<OverviewStats> {
    const queryParams = new URLSearchParams();
    if (params?.start_date) queryParams.append('start_date', params.start_date);
    if (params?.end_date) queryParams.append('end_date', params.end_date);
    const query = queryParams.toString();
    return this.request<OverviewStats>(`/analytics/overview${query ? `?${query}` : ''}`);
  }

  async getCampaignAnalytics(campaignId: string): Promise<CampaignAnalytics> {
    return this.request<CampaignAnalytics>(`/analytics/campaigns/${campaignId}`);
  }

  async getLeadAnalytics(): Promise<LeadAnalytics> {
    return this.request<LeadAnalytics>('/analytics/leads');
  }

  async getDeliverabilityMetrics(): Promise<DeliverabilityMetrics> {
    return this.request<DeliverabilityMetrics>('/analytics/deliverability');
  }

  async getOverview(): Promise<OverviewStats & { total_leads: number; verified_leads: number; total_campaigns: number; active_campaigns: number }> {
    return this.request('/analytics/overview');
  }

  // ============================================================================
  // EMAIL ACCOUNTS / WARMUP ENDPOINTS
  // ============================================================================

  async getEmailAccounts(): Promise<EmailAccount[]> {
    return this.request<EmailAccount[]>('/email-accounts');
  }

  async createEmailAccount(account: CreateEmailAccountParams): Promise<EmailAccount> {
    return this.request<EmailAccount>('/email-accounts', {
      method: 'POST',
      body: JSON.stringify(account),
    });
  }

  async startWarmup(accountId: string): Promise<EmailAccount> {
    return this.request<EmailAccount>(`/email-accounts/${accountId}/warmup/start`, { method: 'POST' });
  }

  async pauseWarmup(accountId: string): Promise<EmailAccount> {
    return this.request<EmailAccount>(`/email-accounts/${accountId}/warmup/pause`, { method: 'POST' });
  }

  async getWarmupStats(accountId: string): Promise<WarmupStats> {
    return this.request<WarmupStats>(`/email-accounts/${accountId}/warmup/stats`);
  }

  // ============================================================================
  // FOUNDER DASHBOARD ENDPOINTS
  // ============================================================================

  async getFounderDashboard(): Promise<FounderDashboardData> {
    return this.request<FounderDashboardData>('/founder/dashboard');
  }

  async getFounderCampaigns(): Promise<CampaignCard[]> {
    return this.request<CampaignCard[]>('/founder/campaigns');
  }

  async pauseFounderCampaign(campaignId: string): Promise<void> {
    return this.request(`/founder/campaigns/${campaignId}/pause`, { method: 'POST' });
  }

  async resumeFounderCampaign(campaignId: string): Promise<void> {
    return this.request(`/founder/campaigns/${campaignId}/resume`, { method: 'POST' });
  }

  async getInboxHealth(): Promise<InboxHealthCard[]> {
    return this.request<InboxHealthCard[]>('/founder/inboxes');
  }

  async getInboxHealthDetail(inboxId: string): Promise<InboxHealthCard> {
    return this.request<InboxHealthCard>(`/founder/inboxes/${inboxId}/health`);
  }

  async getReplies(params?: { intent?: string; limit?: number; offset?: number }): Promise<ReplyCard[]> {
    const queryParams = new URLSearchParams();
    if (params?.intent) queryParams.append('intent', params.intent);
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    const query = queryParams.toString();
    return this.request<ReplyCard[]>(`/founder/replies${query ? `?${query}` : ''}`);
  }

  async actionReply(replyId: string, action: string): Promise<void> {
    return this.request(`/founder/replies/${replyId}/action`, {
      method: 'POST',
      body: JSON.stringify({ action }),
    });
  }

  async classifyReply(replyId: string): Promise<{ reply_id: string; intent: string; confidence: number }> {
    return this.request('/founder/replies/classify', {
      method: 'POST',
      body: JSON.stringify({ reply_id: replyId }),
    });
  }

  async getAutoPauseEvents(): Promise<AutoPauseEvent[]> {
    return this.request<AutoPauseEvent[]>('/founder/auto-pause-events');
  }

  async resolveAutoPauseEvent(eventId: string): Promise<void> {
    return this.request(`/founder/auto-pause-events/${eventId}/resolve`, { method: 'POST' });
  }

  async getCostStats(): Promise<CostPerMeetingStats> {
    return this.request<CostPerMeetingStats>('/founder/costs');
  }

  async updateCosts(data: {
    campaign_id: string;
    domain_cost?: number;
    inbox_cost?: number;
    lead_cost?: number;
    tool_cost?: number;
    other_cost?: number;
  }): Promise<void> {
    return this.request('/founder/costs', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getMeetings(): Promise<Meeting[]> {
    return this.request<Meeting[]>('/founder/meetings');
  }

  async createMeeting(data: {
    campaign_id?: string;
    lead_id?: string;
    reply_id?: string;
    title?: string;
    scheduled_at?: string;
  }): Promise<{ id: string }> {
    return this.request('/founder/meetings', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getFounderSettings(): Promise<WorkspaceSettings> {
    return this.request<WorkspaceSettings>('/founder/settings');
  }

  async updateFounderSettings(settings: Partial<WorkspaceSettings>): Promise<void> {
    return this.request('/founder/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }
}

// Export singleton instance
export const api = new ApiClient();
