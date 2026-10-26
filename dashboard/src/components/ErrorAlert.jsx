/**
 * Error alert component
 */

import { AlertTriangle, XCircle } from 'lucide-react';

export function ErrorAlert({ title = 'Error', message, type = 'error', onRetry }) {
  const Icon = type === 'error' ? XCircle : AlertTriangle;
  const style = type === 'error'
    ? { 
        backgroundColor: 'var(--color-error-bg)', 
        borderColor: 'var(--color-error)', 
        color: 'var(--color-error)' 
      }
    : { 
        backgroundColor: 'var(--color-warning-bg)', 
        borderColor: 'var(--color-warning)', 
        color: 'var(--color-warning)' 
      };

  return (
    <div className="rounded-lg border p-4" style={style}>
      <div className="flex items-start gap-3">
        <Icon className="w-5 h-5 flex-shrink-0 mt-0.5" />
        <div className="flex-1">
          <h4 className="font-semibold text-sm">{title}</h4>
          <p className="text-sm mt-1 opacity-90">{message}</p>
          {onRetry && (
            <button
              onClick={onRetry}
              className="mt-2 text-sm underline hover:no-underline"
            >
              Retry
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
