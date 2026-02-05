'use client';

import { useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import Link from 'next/link';
import { Check, ArrowRight, Zap, Shield, Users, BarChart3 } from 'lucide-react';
import { api, PricingTier, isAuthenticated } from '@/lib/api';

export default function PricingPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [tiers, setTiers] = useState<PricingTier[]>([]);
  const [billingCycle, setBillingCycle] = useState<'monthly' | 'yearly'>('monthly');
  const [loading, setLoading] = useState(true);
  const [checkoutLoading, setCheckoutLoading] = useState<string | null>(null);

  const checkoutStatus = searchParams.get('checkout');

  useEffect(() => {
    loadPricing();
  }, []);

  const loadPricing = async () => {
    try {
      const data = await api.getPricing();
      setTiers(data);
    } catch (error) {
      console.error('Failed to load pricing:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSelectPlan = async (tierId: string) => {
    if (!isAuthenticated()) {
      router.push(`/register?plan=${tierId}`);
      return;
    }

    setCheckoutLoading(tierId);
    try {
      const { checkout_url } = await api.createCheckout(tierId, billingCycle);
      window.location.href = checkout_url;
    } catch (error) {
      console.error('Failed to create checkout:', error);
      alert('Failed to start checkout. Please try again.');
    } finally {
      setCheckoutLoading(null);
    }
  };

  const formatPrice = (cents: number) => {
    return `$${(cents / 100).toFixed(0)}`;
  };

  const getYearlySavings = (tier: PricingTier) => {
    const monthlyCost = tier.price_monthly * 12;
    const yearlyCost = tier.price_yearly;
    return Math.round(((monthlyCost - yearlyCost) / monthlyCost) * 100);
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-stone-50 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-stone-900"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-stone-50">
      {/* Header */}
      <header className="border-b border-stone-200 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex justify-between items-center">
          <Link href="/" className="text-2xl font-bold text-stone-900">
            OutreachIQ
          </Link>
          <div className="flex items-center gap-4">
            {isAuthenticated() ? (
              <Link href="/dashboard" className="text-stone-600 hover:text-stone-900">
                Dashboard
              </Link>
            ) : (
              <>
                <Link href="/login" className="text-stone-600 hover:text-stone-900">
                  Sign in
                </Link>
                <Link
                  href="/register"
                  className="bg-stone-900 text-white px-4 py-2 rounded-lg hover:bg-stone-800"
                >
                  Get Started
                </Link>
              </>
            )}
          </div>
        </div>
      </header>

      {/* Success/Cancel Messages */}
      {checkoutStatus === 'success' && (
        <div className="bg-green-50 border-b border-green-200 px-4 py-3">
          <div className="max-w-7xl mx-auto text-center text-green-800">
            🎉 Payment successful! Your subscription is now active.
          </div>
        </div>
      )}
      {checkoutStatus === 'cancelled' && (
        <div className="bg-yellow-50 border-b border-yellow-200 px-4 py-3">
          <div className="max-w-7xl mx-auto text-center text-yellow-800">
            Checkout was cancelled. Feel free to try again when you're ready.
          </div>
        </div>
      )}

      {/* Hero */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 text-center">
        <h1 className="text-4xl md:text-5xl font-bold text-stone-900 mb-4">
          Simple, Transparent Pricing
        </h1>
        <p className="text-xl text-stone-600 max-w-2xl mx-auto mb-8">
          Choose the plan that fits your outreach needs. All plans include our core features.
        </p>

        {/* Billing Toggle */}
        <div className="flex items-center justify-center gap-4 mb-12">
          <span className={billingCycle === 'monthly' ? 'text-stone-900 font-medium' : 'text-stone-500'}>
            Monthly
          </span>
          <button
            onClick={() => setBillingCycle(billingCycle === 'monthly' ? 'yearly' : 'monthly')}
            className={`relative w-14 h-7 rounded-full transition-colors ${
              billingCycle === 'yearly' ? 'bg-stone-900' : 'bg-stone-300'
            }`}
          >
            <span
              className={`absolute top-1 w-5 h-5 bg-white rounded-full transition-transform ${
                billingCycle === 'yearly' ? 'translate-x-8' : 'translate-x-1'
              }`}
            />
          </button>
          <span className={billingCycle === 'yearly' ? 'text-stone-900 font-medium' : 'text-stone-500'}>
            Yearly
            <span className="ml-2 text-green-600 text-sm font-medium">Save 17%</span>
          </span>
        </div>
      </div>

      {/* Pricing Cards */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pb-16">
        <div className="grid md:grid-cols-3 gap-8">
          {tiers.map((tier, index) => {
            const isPopular = tier.id === 'professional';
            const price = billingCycle === 'yearly' ? tier.price_yearly / 12 : tier.price_monthly;
            
            return (
              <div
                key={tier.id}
                className={`bg-white rounded-2xl border-2 p-8 relative ${
                  isPopular ? 'border-stone-900 shadow-xl' : 'border-stone-200'
                }`}
              >
                {isPopular && (
                  <div className="absolute -top-4 left-1/2 -translate-x-1/2 bg-stone-900 text-white px-4 py-1 rounded-full text-sm font-medium">
                    Most Popular
                  </div>
                )}

                <h3 className="text-xl font-bold text-stone-900 mb-2">{tier.name}</h3>
                
                <div className="mb-6">
                  <span className="text-4xl font-bold text-stone-900">
                    {formatPrice(price)}
                  </span>
                  <span className="text-stone-500">/month</span>
                  {billingCycle === 'yearly' && (
                    <div className="text-sm text-green-600 mt-1">
                      Save {getYearlySavings(tier)}% with yearly billing
                    </div>
                  )}
                </div>

                <div className="space-y-3 mb-8">
                  <div className="flex items-center gap-2 text-stone-700">
                    <Users size={18} className="text-stone-400" />
                    <span>{tier.leads_per_month.toLocaleString()} leads/month</span>
                  </div>
                  <div className="flex items-center gap-2 text-stone-700">
                    <Zap size={18} className="text-stone-400" />
                    <span>{tier.inboxes} inbox{tier.inboxes > 1 ? 'es' : ''}</span>
                  </div>
                  <div className="flex items-center gap-2 text-stone-700">
                    <BarChart3 size={18} className="text-stone-400" />
                    <span>{tier.emails_per_month.toLocaleString()} emails/month</span>
                  </div>
                </div>

                <div className="border-t border-stone-200 pt-6 mb-8">
                  <ul className="space-y-3">
                    {tier.features.map((feature, i) => (
                      <li key={i} className="flex items-start gap-2">
                        <Check size={18} className="text-green-600 mt-0.5 flex-shrink-0" />
                        <span className="text-stone-600 text-sm">{feature}</span>
                      </li>
                    ))}
                  </ul>
                </div>

                <button
                  onClick={() => handleSelectPlan(tier.id)}
                  disabled={checkoutLoading === tier.id}
                  className={`w-full py-3 rounded-lg font-medium flex items-center justify-center gap-2 transition-all ${
                    isPopular
                      ? 'bg-stone-900 text-white hover:bg-stone-800'
                      : 'bg-stone-100 text-stone-900 hover:bg-stone-200'
                  } disabled:opacity-50`}
                >
                  {checkoutLoading === tier.id ? (
                    'Processing...'
                  ) : (
                    <>
                      Get Started
                      <ArrowRight size={18} />
                    </>
                  )}
                </button>
              </div>
            );
          })}
        </div>
      </div>

      {/* Features Section */}
      <div className="bg-white border-t border-stone-200 py-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-stone-900 text-center mb-12">
            All Plans Include
          </h2>
          <div className="grid md:grid-cols-4 gap-8">
            {[
              { icon: Shield, title: 'Email Verification', desc: 'Real-time email validation' },
              { icon: Zap, title: 'Inbox Warmup', desc: 'Protect your domain reputation' },
              { icon: Users, title: 'Signal-Based Leads', desc: 'Fresh, intent-driven data' },
              { icon: BarChart3, title: 'Analytics', desc: 'Track campaign performance' },
            ].map((feature, i) => (
              <div key={i} className="text-center">
                <div className="w-12 h-12 bg-stone-100 rounded-lg flex items-center justify-center mx-auto mb-4">
                  <feature.icon size={24} className="text-stone-700" />
                </div>
                <h3 className="font-medium text-stone-900 mb-1">{feature.title}</h3>
                <p className="text-sm text-stone-500">{feature.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* FAQ */}
      <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
        <h2 className="text-2xl font-bold text-stone-900 text-center mb-8">
          Frequently Asked Questions
        </h2>
        <div className="space-y-6">
          {[
            {
              q: 'Can I change plans later?',
              a: 'Yes! You can upgrade or downgrade your plan at any time. Changes take effect immediately.',
            },
            {
              q: 'What happens if I exceed my limits?',
              a: 'We\'ll notify you when you\'re approaching your limits. You can upgrade your plan or wait until the next billing cycle.',
            },
            {
              q: 'Is there a free trial?',
              a: 'We offer a 14-day money-back guarantee on all plans. Try risk-free!',
            },
            {
              q: 'How does inbox warmup work?',
              a: 'Our warmup system gradually increases your sending volume over 30 days to build domain reputation and ensure high deliverability.',
            },
          ].map((faq, i) => (
            <div key={i} className="bg-white rounded-lg border border-stone-200 p-6">
              <h3 className="font-medium text-stone-900 mb-2">{faq.q}</h3>
              <p className="text-stone-600">{faq.a}</p>
            </div>
          ))}
        </div>
      </div>

      {/* CTA */}
      <div className="bg-stone-900 text-white py-16">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 className="text-3xl font-bold mb-4">Ready to Scale Your Outreach?</h2>
          <p className="text-stone-300 mb-8">
            Join thousands of sales teams using OutreachIQ to book more meetings.
          </p>
          <Link
            href="/register"
            className="inline-flex items-center gap-2 bg-white text-stone-900 px-8 py-3 rounded-lg font-medium hover:bg-stone-100 transition-all"
          >
            Start Free Trial
            <ArrowRight size={18} />
          </Link>
        </div>
      </div>
    </div>
  );
}
