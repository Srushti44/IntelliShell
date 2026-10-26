/**
 * Sidebar Navigation Component
 * Swiss-style persistent navigation
 */

import { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { 
  Terminal, 
  Activity, 
  Cpu, 
  History, 
  Brain, 
  Settings,
  ChevronLeft,
  ChevronRight 
} from 'lucide-react';
import { cn } from '../lib/utils';

const navigation = [
  { name: 'System Overview', href: '/overview', icon: Activity },
  { name: 'AI Command Assistant', href: '/assistant', icon: Brain },
  { name: 'Process Manager', href: '/processes', icon: Cpu },
  { name: 'Monitoring', href: '/monitoring', icon: Activity },
  { name: 'Command History', href: '/history', icon: History },
  { name: 'Settings', href: '/settings', icon: Settings },
];

export function Sidebar() {
  const [collapsed, setCollapsed] = useState(false);
  const location = useLocation();

  return (
    <aside 
      className={cn(
        'bg-primary flex flex-col border-r transition-all duration-300',
        collapsed ? 'w-16' : 'w-64'
      )}
      style={{
        backgroundColor: 'var(--brown-deep)',
        borderColor: 'var(--brown-terracotta)',
      }}
    >
      {/* Brand */}
      <div className="px-4 py-6 border-b" style={{ borderColor: 'var(--brown-terracotta)' }}>
        <div className="flex items-center gap-3">
          <Terminal className="w-8 h-8 flex-shrink-0" style={{ color: 'var(--cream-warm)' }} />
          {!collapsed && (
            <div>
              <h1 className="font-heading font-bold text-xl tracking-tight" style={{ color: 'var(--cream-warm)' }}>
                IntelliShell
              </h1>
              <p className="text-xs mt-0.5" style={{ color: 'var(--olive-muted)' }}>
                v0.7.0
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-2 py-4 space-y-1">
        {navigation.map((item) => {
          const Icon = item.icon;
          const isActive = location.pathname === item.href;
          
          return (
            <Link
              key={item.name}
              to={item.href}
              className={cn(
                'w-full flex items-center gap-3 px-3 py-2.5 rounded-md transition-colors',
                'hover:bg-opacity-10 hover:bg-white',
                isActive && 'bg-opacity-10 bg-white'
              )}
              style={{
                color: isActive ? 'var(--cream-warm)' : 'var(--olive-muted)',
              }}
              title={collapsed ? item.name : undefined}
            >
              <Icon className="w-5 h-5 flex-shrink-0" />
              {!collapsed && (
                <span className="font-medium text-sm">{item.name}</span>
              )}
            </Link>
          );
        })}
      </nav>

      {/* Collapse Toggle */}
      <div className="p-4 border-t" style={{ borderColor: 'var(--brown-terracotta)' }}>
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-md transition-colors hover:bg-opacity-10 hover:bg-white"
          style={{ color: 'var(--olive-muted)' }}
        >
          {collapsed ? (
            <ChevronRight className="w-5 h-5" />
          ) : (
            <>
              <ChevronLeft className="w-5 h-5" />
              <span className="text-sm">Collapse</span>
            </>
          )}
        </button>
      </div>
    </aside>
  );
}


export default Sidebar;
