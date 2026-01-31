'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { 
  LayoutDashboard, 
  Users, 
  Mail, 
  Activity, 
  Settings, 
  Zap,
  Radio,
  Wand2,
  ChevronDown
} from 'lucide-react';
import { useState } from 'react';

const navItems = [
  { href: '/dashboard/founder', label: 'Dashboard', icon: LayoutDashboard },
  { href: '/dashboard/leads', label: 'Leads', icon: Users },
  { href: '/dashboard/campaigns', label: 'Campaigns', icon: Mail },
  { href: '/dashboard/warmup', label: 'Inboxes', icon: Activity },
  { href: '/signals', label: 'Signals', icon: Radio },
];

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const pathname = usePathname();
  const [showToolsMenu, setShowToolsMenu] = useState(false);

  return (
    <div className="min-h-screen bg-nord-bg">
      {/* Top Navigation */}
      <header className="bg-nord-surface border-b border-nord-elevated/50 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6">
          <div className="flex items-center justify-between h-16">
            {/* Logo */}
            <Link href="/" className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-lg bg-nord-frost3 flex items-center justify-center">
                <Zap className="w-4 h-4 text-nord-bg" />
              </div>
              <span className="text-lg font-bold text-nord-text">OutreachIQ</span>
            </Link>

            {/* Main Nav */}
            <nav className="hidden md:flex items-center gap-1">
              {navItems.map((item) => {
                const Icon = item.icon;
                const isActive = pathname === item.href || 
                  (item.href !== '/dashboard/founder' && pathname.startsWith(item.href));
                
                return (
                  <Link
                    key={item.href}
                    href={item.href}
                    className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all ${
                      isActive
                        ? 'bg-nord-frost3/20 text-nord-frost3'
                        : 'text-nord-text-muted hover:text-nord-text hover:bg-nord-elevated/50'
                    }`}
                  >
                    <Icon size={18} />
                    <span>{item.label}</span>
                  </Link>
                );
              })}

              {/* Tools Dropdown */}
              <div className="relative">
                <button
                  onClick={() => setShowToolsMenu(!showToolsMenu)}
                  onBlur={() => setTimeout(() => setShowToolsMenu(false), 150)}
                  className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium text-nord-text-muted hover:text-nord-text hover:bg-nord-elevated/50 transition-all"
                >
                  <Wand2 size={18} />
                  <span>Tools</span>
                  <ChevronDown size={14} className={`transition-transform ${showToolsMenu ? 'rotate-180' : ''}`} />
                </button>
                
                {showToolsMenu && (
                  <div className="absolute top-full right-0 mt-1 w-48 bg-nord-surface border border-nord-elevated rounded-lg shadow-lg py-1 z-50">
                    <Link
                      href="/dashboard/settings?tab=generator"
                      className="flex items-center gap-2 px-4 py-2 text-sm text-nord-text-muted hover:text-nord-text hover:bg-nord-elevated/50"
                    >
                      <Wand2 size={16} />
                      Email Generator
                    </Link>
                  </div>
                )}
              </div>
            </nav>

            {/* Right Side */}
            <div className="flex items-center gap-2">
              <Link
                href="/dashboard/settings"
                className={`p-2 rounded-lg transition-all ${
                  pathname.startsWith('/dashboard/settings')
                    ? 'bg-nord-frost3/20 text-nord-frost3'
                    : 'text-nord-text-muted hover:text-nord-text hover:bg-nord-elevated/50'
                }`}
              >
                <Settings size={20} />
              </Link>
            </div>
          </div>
        </div>

        {/* Mobile Nav */}
        <div className="md:hidden border-t border-nord-elevated/50 overflow-x-auto">
          <div className="flex items-center gap-1 px-4 py-2">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = pathname === item.href || 
                (item.href !== '/dashboard/founder' && pathname.startsWith(item.href));
              
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium whitespace-nowrap transition-all ${
                    isActive
                      ? 'bg-nord-frost3/20 text-nord-frost3'
                      : 'text-nord-text-muted hover:text-nord-text'
                  }`}
                >
                  <Icon size={14} />
                  <span>{item.label}</span>
                </Link>
              );
            })}
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main>
        {children}
      </main>
    </div>
  );
}
