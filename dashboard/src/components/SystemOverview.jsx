/**
 * System Overview Component
 * Displays live CPU, RAM, and process metrics
 */

import { Cpu, MemoryStick, Activity, Clock } from 'lucide-react';
import { useSystemSnapshot } from '../hooks/useSystemSnapshot';
import { Card } from './Card';
import { LoadingSpinner } from './LoadingSpinner';
import { ErrorAlert } from './ErrorAlert';
import { formatBytes, formatPercent, formatRelativeTime } from '../lib/utils';

function MetricCard({ icon: Icon, label, value, subtitle }) {
  return (
    <div 
      className="rounded-lg border p-4"
      style={{ backgroundColor: 'var(--offwhite-main)', borderColor: 'var(--color-border)' }}
    >
      <div className="flex items-center gap-3">
        <div 
          className="p-2 rounded-lg"
          style={{ backgroundColor: 'var(--color-bg-subtle)' }}
        >
          <Icon className="w-5 h-5" style={{ color: 'var(--color-primary)' }} />
        </div>
        <div className="flex-1">
          <p className="text-sm" style={{ color: 'var(--color-text-muted)' }}>{label}</p>
          <p 
            className="text-2xl font-bold"
            style={{ color: 'var(--ink-dark)', fontFamily: 'var(--font-heading)' }}
          >
            {value}
          </p>
          {subtitle && <p className="text-xs mt-1" style={{ color: 'var(--color-text-muted)' }}>{subtitle}</p>}
        </div>
      </div>
    </div>
  );
}

export function SystemOverview() {
  const { data, isLoading, error, refetch } = useSystemSnapshot();

  if (isLoading) {
    return (
      <Card title="System Overview">
        <div className="py-8">
          <LoadingSpinner text="Loading system metrics..." />
        </div>
      </Card>
    );
  }

  if (error) {
    return (
      <Card title="System Overview">
        <ErrorAlert
          title="Failed to load system metrics"
          message={error.message || 'Could not connect to backend server'}
          onRetry={refetch}
        />
      </Card>
    );
  }

  const topProcesses = data.processes
    ?.sort((a, b) => b.memory_kb - a.memory_kb)
    .slice(0, 5) || [];

  return (
    <Card title="System Overview">
      <div className="space-y-6">
        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <MetricCard
            icon={Cpu}
            label="CPU Usage"
            value={formatPercent(data.cpu_percent)}
            color="blue"
          />
          <MetricCard
            icon={MemoryStick}
            label="Memory Usage"
            value={formatPercent(data.memory_used_percent)}
            subtitle={`${formatBytes(data.memory_used_bytes)} / ${formatBytes(data.memory_total_bytes)}`}
            color="purple"
          />
          <MetricCard
            icon={Activity}
            label="Active Processes"
            value={data.processes?.length || 0}
            color="green"
          />
          <MetricCard
            icon={Clock}
            label="Last Updated"
            value={formatRelativeTime(data.timestamp)}
            color="gray"
          />
        </div>

        {/* Top Processes */}
        {topProcesses.length > 0 && (
          <div>
            <h4 
              className="text-sm font-semibold mb-3"
              style={{ color: 'var(--ink-dark)', fontFamily: 'var(--font-heading)' }}
            >
              Top Memory Consumers
            </h4>
            <div className="space-y-2">
              {topProcesses.map((proc) => (
                <div
                  key={proc.pid}
                  className="flex items-center justify-between text-sm border-b pb-2"
                  style={{ borderColor: 'var(--color-border)' }}
                >
                  <div className="flex items-center gap-3 flex-1 min-w-0">
                    <span 
                      className="text-xs font-mono"
                      style={{ color: 'var(--color-text-muted)', fontFamily: 'var(--font-mono)' }}
                    >
                      {proc.pid}
                    </span>
                    <span className="truncate" style={{ color: 'var(--ink-dark)' }}>{proc.name}</span>
                    <span className="text-xs" style={{ color: 'var(--color-text-muted)' }}>{proc.state}</span>
                  </div>
                  <div className="flex items-center gap-4 text-xs">
                    <span style={{ color: 'var(--color-text-muted)' }}>{formatBytes(proc.memory_kb * 1024)}</span>
                    <span style={{ color: 'var(--color-text-muted)' }}>{formatPercent(proc.cpu_percent || 0)}</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}
