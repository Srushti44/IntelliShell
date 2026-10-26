/**
 * Top Bar Component
 * Page title, status, and actions
 */

import { RefreshCw } from 'lucide-react';
import { ConnectionStatus } from './ConnectionStatus';

export function TopBar({ title, subtitle, onRefresh, isRefreshing }) {
  return (
    <header 
      className="bg-white border-b px-8 py-4 flex items-center justify-between"
      style={{ 
        backgroundColor: 'var(--offwhite-main)',
        borderColor: 'var(--color-border)',
      }}
    >
      <div>
        <h2 
          className="font-heading font-bold text-2xl tracking-tight"
          style={{ color: 'var(--ink-dark)' }}
        >
          {title}
        </h2>
        {subtitle && (
          <p className="text-sm mt-1" style={{ color: 'var(--color-text-muted)' }}>
            {subtitle}
          </p>
        )}
      </div>

      <div className="flex items-center gap-4">
        {onRefresh && (
          <button
            onClick={onRefresh}
            disabled={isRefreshing}
            className="flex items-center gap-2 px-4 py-2 rounded-md border transition-colors hover:bg-opacity-5 hover:bg-black disabled:opacity-50"
            style={{
              color: 'var(--color-primary)',
              borderColor: 'var(--color-border)',
            }}
          >
            <RefreshCw className={cn('w-4 h-4', isRefreshing && 'animate-spin')} />
            <span className="text-sm font-medium">Refresh</span>
          </button>
        )}
        <ConnectionStatus />
      </div>
    </header>
  );
}

function cn(...classes) {
  return classes.filter(Boolean).join(' ');
}


export default TopBar;
