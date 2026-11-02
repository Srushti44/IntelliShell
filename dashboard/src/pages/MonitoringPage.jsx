import React from 'react';
import { useSystemSnapshot } from '../hooks/useSystemSnapshot';
import { SystemCharts } from '../components/SystemCharts';
import { EnhancedSystemMetrics } from '../components/EnhancedSystemMetrics';
import { PageSection } from '../components/PageSection';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { ErrorAlert } from '../components/ErrorAlert';
import '../styles/EnhancedMetrics.css';

/**
 * MonitoringPage - Comprehensive system monitoring with 7+ real-time metrics
 * FIXED: Now synchronized with System Overview data source
 * Shows live telemetry visualization with NO mock data
 */
export function MonitoringPage() {
  // FIXED: Using consistent data structure with SystemOverview
  const { data: snapshot, isLoading, error, refetch } = useSystemSnapshot();

  if (isLoading) {
    return (
      <div className="page-content">
        <LoadingSpinner message="Loading comprehensive monitoring data..." />
      </div>
    );
  }

  if (error) {
    return (
      <div className="page-content">
        <ErrorAlert 
          title="Failed to load monitoring data" 
          message={error.message}
          onRetry={refetch}
        />
      </div>
    );
  }

  return (
    <div className="page-content">
      <PageSection
        title="Real-Time Telemetry"
        description="Live resource metrics updated every 2 seconds. Charts display rolling window of recent snapshots."
      >
        <SystemCharts currentSnapshot={snapshot} />
      </PageSection>

      <PageSection
        title="Comprehensive System Metrics (7+ Linux Metrics)"
        description="System Load, Swap Usage, CPU Per-Core, Disk I/O, Process States, Uptime, and Context Switches"
        style={{ marginTop: '32px' }}
      >
        <EnhancedSystemMetrics snapshot={snapshot} />
      </PageSection>

      <PageSection
        title="Current Resource Snapshot"
        description="Latest instantaneous values from Linux /proc filesystem"
        style={{ marginTop: '32px' }}
      >
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
          gap: '16px',
          padding: '20px',
          background: 'var(--color-surface)',
          border: '1px solid var(--color-border)',
          borderRadius: '4px'
        }}>
          <MetricCard 
            label="CPU Usage" 
            value={`${snapshot?.cpu_percent?.toFixed(1) || '0.0'}%`}
            color="var(--color-brown-deep)"
          />
          <MetricCard 
            label="Memory Used" 
            value={`${snapshot?.memory_used_percent?.toFixed(1) || '0.0'}%`}
            subtitle={`${formatBytes(snapshot?.memory_used_bytes || 0)} / ${formatBytes(snapshot?.memory_total_bytes || 0)}`}
            color="var(--color-terracotta)"
          />
          <MetricCard 
            label="Disk Used" 
            value={`${snapshot?.disk_used_percent?.toFixed(1) || '0.0'}%`}
            subtitle={`${formatBytes(snapshot?.disk_used_bytes || 0)} / ${formatBytes(snapshot?.disk_total_bytes || 0)}`}
            color="var(--color-olive-muted)"
          />
          <MetricCard 
            label="Active Processes" 
            value={snapshot?.processes?.length || 0}
            color="var(--color-brown-deep)"
          />
          <MetricCard 
            label="Network RX" 
            value={formatBytes(snapshot?.network_rx_bytes || 0)}
            color="var(--color-terracotta)"
          />
          <MetricCard 
            label="Network TX" 
            value={formatBytes(snapshot?.network_tx_bytes || 0)}
            color="var(--color-olive-muted)"
          />
          <MetricCard 
            label="System Load (1m)" 
            value={snapshot?.load_1min?.toFixed(2) || '0.00'}
            color="var(--color-brown-deep)"
          />
          <MetricCard 
            label="Swap Used" 
            value={`${snapshot?.swap_used_percent?.toFixed(1) || '0.0'}%`}
            subtitle={snapshot?.swap_total_bytes > 0 ? formatBytes(snapshot?.swap_used_bytes || 0) : 'No swap'}
            color="var(--color-terracotta)"
          />
        </div>
      </PageSection>
    </div>
  );
}

function formatBytes(bytes) {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function MetricCard({ label, value, subtitle, color }) {
  return (
    <div style={{
      padding: '16px',
      background: 'var(--color-background)',
      border: '1px solid var(--color-border)',
      borderRadius: '2px'
    }}>
      <div style={{
        fontFamily: 'var(--font-body)',
        fontSize: '12px',
        color: 'var(--color-text-secondary)',
        marginBottom: '8px',
        textTransform: 'uppercase',
        letterSpacing: '0.5px'
      }}>
        {label}
      </div>
      <div style={{
        fontFamily: 'var(--font-heading)',
        fontSize: '24px',
        fontWeight: '600',
        color: color
      }}>
        {value}
      </div>
      {subtitle && (
        <div style={{
          fontFamily: 'var(--font-body)',
          fontSize: '11px',
          color: 'var(--color-text-muted)',
          marginTop: '4px'
        }}>
          {subtitle}
        </div>
      )}
    </div>
  );
}

export default MonitoringPage;
