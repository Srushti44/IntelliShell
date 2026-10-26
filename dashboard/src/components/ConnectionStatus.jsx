/**
 * Connection Status Component
 * Shows backend service health
 */

import { CheckCircle, XCircle, AlertCircle } from 'lucide-react';
import { useBackendHealth } from '../hooks/useBackendHealth';

function ServiceStatus({ name, status, error }) {
  const Icon = status === 'online' ? CheckCircle : XCircle;
  const colorStyle = status === 'online' 
    ? { color: 'var(--color-success)' } 
    : { color: 'var(--color-error)' };

  return (
    <div className="flex items-center gap-2 text-sm">
      <Icon className="w-4 h-4" style={colorStyle} />
      <span className="font-medium" style={{ color: 'var(--ink-dark)' }}>{name}:</span>
      <span style={colorStyle}>{status === 'online' ? 'Online' : 'Offline'}</span>
      {status === 'offline' && error && (
        <span className="text-xs" style={{ color: 'var(--color-text-muted)' }}>({error.message})</span>
      )}
    </div>
  );
}

export function ConnectionStatus() {
  const { data: health, isLoading } = useBackendHealth();

  if (isLoading) {
    return (
      <div 
        className="rounded-lg border p-3"
        style={{ backgroundColor: 'var(--color-info-bg)', borderColor: 'var(--color-border)' }}
      >
        <div className="flex items-center gap-2 text-sm" style={{ color: 'var(--ink-dark)' }}>
          <AlertCircle className="w-4 h-4" />
          <span>Checking backend status...</span>
        </div>
      </div>
    );
  }

  if (!health) {
    return (
      <div 
        className="rounded-lg border p-3"
        style={{ backgroundColor: 'var(--color-error-bg)', borderColor: 'var(--color-error)' }}
      >
        <div className="flex items-center gap-2 text-sm" style={{ color: 'var(--color-error)' }}>
          <XCircle className="w-4 h-4" />
          <span>Cannot reach backend services</span>
        </div>
      </div>
    );
  }

  const allOnline = health.rust?.status === 'online' && health.python?.status === 'online';

  return (
    <div
      className="rounded-lg border p-3"
      style={
        allOnline
          ? { backgroundColor: 'var(--color-success-bg)', borderColor: 'var(--color-success)' }
          : { backgroundColor: 'var(--color-warning-bg)', borderColor: 'var(--color-warning)' }
      }
    >
      <div className="space-y-2">
        <ServiceStatus
          name="Rust Server"
          status={health.rust?.status}
          error={health.rust?.error}
        />
        <ServiceStatus
          name="Python AI Service"
          status={health.python?.status}
          error={health.python?.error}
        />
      </div>
    </div>
  );
}
