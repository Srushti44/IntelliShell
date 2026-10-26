/**
 * Command History Component
 * Displays executed commands with their results
 */

import { useState } from 'react';
import { Clock, Terminal, Copy, ChevronDown, ChevronRight } from 'lucide-react';
import { useCommandHistory } from '../hooks/useCommandHistory';
import { Card } from './Card';
import { LoadingSpinner } from './LoadingSpinner';
import { ErrorAlert } from './ErrorAlert';
import { StatusBadge } from './StatusBadge';
import { formatRelativeTime } from '../lib/utils';

function HistoryItem({ item, onReuse }) {
  const [expanded, setExpanded] = useState(false);

  const commandString = item.args
    ? `${item.command} ${item.args.join(' ')}`
    : item.command;

  return (
    <div 
      className="border rounded-lg overflow-hidden"
      style={{ borderColor: 'var(--color-border)' }}
    >
      <div
        className="p-4 hover:bg-gray-50 cursor-pointer"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-2">
              <Terminal className="w-4 h-4 flex-shrink-0" style={{ color: 'var(--color-text-muted)' }} />
              <code 
                className="text-sm truncate"
                style={{ color: 'var(--ink-dark)', fontFamily: 'var(--font-mono)' }}
              >
                {commandString}
              </code>
            </div>
            <div className="flex items-center gap-3 text-xs" style={{ color: 'var(--color-text-muted)' }}>
              <span className="flex items-center gap-1">
                <Clock className="w-3 h-3" />
                {item.executed_at ? formatRelativeTime(new Date(item.executed_at).getTime() / 1000) : 'Unknown'}
              </span>
              {item.exit_code !== undefined && (
                <span style={{ color: item.exit_code === 0 ? 'var(--color-success)' : 'var(--color-error)' }}>
                  Exit: {item.exit_code}
                </span>
              )}
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            {item.risk_level && <StatusBadge type="risk">{item.risk_level}</StatusBadge>}
            {item.execution_mode && <StatusBadge type="method">{item.execution_mode}</StatusBadge>}
            <button
              onClick={(e) => {
                e.stopPropagation();
                setExpanded(!expanded);
              }}
              style={{ color: 'var(--color-text-muted)' }}
              className="hover:opacity-70"
            >
              {expanded ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
            </button>
          </div>
        </div>
      </div>

      {expanded && (
        <div 
          className="border-t p-4 space-y-3"
          style={{ borderColor: 'var(--color-border)', backgroundColor: 'var(--color-bg-subtle)' }}
        >
          {item.stdout && (
            <div>
              <label className="text-xs font-semibold mb-1 block" style={{ color: 'var(--ink-dark)' }}>Output:</label>
              <pre 
                className="text-xs border rounded p-2 overflow-x-auto max-h-40"
                style={{ 
                  backgroundColor: 'var(--offwhite-main)', 
                  borderColor: 'var(--color-border)',
                  fontFamily: 'var(--font-mono)' 
                }}
              >
                {item.stdout}
              </pre>
            </div>
          )}
          {item.stderr && (
            <div>
              <label className="text-xs font-semibold mb-1 block" style={{ color: 'var(--color-error)' }}>Error:</label>
              <pre 
                className="text-xs border rounded p-2 overflow-x-auto max-h-40"
                style={{ 
                  backgroundColor: 'var(--color-error-bg)', 
                  borderColor: 'var(--color-error)',
                  color: 'var(--color-error)',
                  fontFamily: 'var(--font-mono)' 
                }}
              >
                {item.stderr}
              </pre>
            </div>
          )}
          <button
            onClick={() => onReuse(commandString)}
            className="flex items-center gap-2 text-sm hover:opacity-80"
            style={{ color: 'var(--color-primary)' }}
          >
            <Copy className="w-4 h-4" />
            Reuse Command
          </button>
        </div>
      )}
    </div>
  );
}

export function CommandHistory({ onReuse }) {
  const { data: history, isLoading, error, refetch } = useCommandHistory();

  if (isLoading) {
    return (
      <Card title="Command History">
        <div className="py-8">
          <LoadingSpinner text="Loading history..." />
        </div>
      </Card>
    );
  }

  if (error) {
    return (
      <Card title="Command History">
        <ErrorAlert
          title="Failed to load history"
          message={error.message}
          onRetry={refetch}
        />
      </Card>
    );
  }

  if (!history || history.length === 0) {
    return (
      <Card title="Command History">
        <div className="text-center py-8" style={{ color: 'var(--color-text-muted)' }}>
          <Terminal className="w-12 h-12 mx-auto mb-2" style={{ color: 'var(--color-border)' }} />
          <p>No command history yet</p>
          <p className="text-sm mt-1">Executed commands will appear here</p>
        </div>
      </Card>
    );
  }

  return (
    <Card title="Command History">
      <div className="space-y-3">
        {history.map((item, index) => (
          <HistoryItem key={item.id || index} item={item} onReuse={onReuse} />
        ))}
      </div>
    </Card>
  );
}
