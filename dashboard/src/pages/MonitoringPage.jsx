import React from 'react';
import { useSystemSnapshot } from '../hooks/useSystemSnapshot';
import { SystemCharts } from '../components/SystemCharts';
import { PageSection } from '../components/PageSection';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { ErrorAlert } from '../components/ErrorAlert';

/**
 * MonitoringPage - Detailed system monitoring with real-time charts
 * Shows telemetry visualization without any mock data
 */
export function MonitoringPage() {
  const { snapshot, isLoading, error } = useSystemSnapshot();

  if (isLoading) {
    return (
      <div className="page-content">
        <LoadingSpinner message="Loading monitoring data..." />
      </div>
    );
  }

  if (error) {
    return (
      <div className="page-content">
        <ErrorAlert 
          title="Failed to load monitoring data" 
          message={error.message}
        />
      </div>
    );
  }

  return (
    <div className="page-content">
      <PageSection
        title="Real-Time Telemetry"
        description="Live resource metrics updated every 5 seconds. Charts display rolling window of recent snapshots."
      >
        <SystemCharts currentSnapshot={snapshot} />
      </PageSection>

      <PageSection
        title="Current Resource Usage"
        description="Latest snapshot values"
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
            value={`${snapshot?.system?.cpu_usage?.toFixed(1) || '0.0'}%`}
            color="var(--color-brown-deep)"
          />
          <MetricCard 
            label="Memory Used" 
            value={`${snapshot?.system?.memory_used_percent?.toFixed(1) || '0.0'}%`}
            color="var(--color-terracotta)"
          />
          <MetricCard 
            label="Disk Used" 
            value={`${snapshot?.system?.disk_used_percent?.toFixed(1) || '0.0'}%`}
            color="var(--color-olive-muted)"
          />
          <MetricCard 
            label="Active Processes" 
            value={snapshot?.processes?.length || 0}
            color="var(--color-brown-deep)"
          />
          <MetricCard 
            label="Network RX" 
            value={`${((snapshot?.system?.network_rx_bytes || 0) / 1024 / 1024).toFixed(2)} MB`}
            color="var(--color-terracotta)"
          />
          <MetricCard 
            label="Network TX" 
            value={`${((snapshot?.system?.network_tx_bytes || 0) / 1024 / 1024).toFixed(2)} MB`}
            color="var(--color-olive-muted)"
          />
        </div>
      </PageSection>
    </div>
  );
}

function MetricCard({ label, value, color }) {
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
    </div>
  );
}


export default MonitoringPage;
