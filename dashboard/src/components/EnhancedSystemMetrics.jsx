/**
 * EnhancedSystemMetrics - Comprehensive 7+ Linux system metrics
 * Displays all required metrics with appropriate visualizations
 */

import React from 'react';
import { Activity, Cpu, HardDrive, Network, Zap, Clock, GitBranch } from 'lucide-react';
import { PieChart, Pie, Cell, BarChart, Bar, ResponsiveContainer, Tooltip, Legend, XAxis, YAxis } from 'recharts';
import '../styles/SystemCharts.css';

export function EnhancedSystemMetrics({ snapshot }) {
  if (!snapshot) {
    return (
      <div className="enhanced-metrics-loading">
        <Activity size={32} />
        <p>Waiting for telemetry data...</p>
      </div>
    );
  }

  // Format helpers
  const formatBytes = (bytes) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatUptime = (seconds) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${mins}m`;
    return `${mins}m`;
  };

  // Metric 1: System Load (Metric Cards)
  const systemLoadData = [
    { label: '1 min', value: snapshot.load_1min?.toFixed(2) || '0.00' },
    { label: '5 min', value: snapshot.load_5min?.toFixed(2) || '0.00' },
    { label: '15 min', value: snapshot.load_15min?.toFixed(2) || '0.00' },
  ];

  // Metric 2: Swap Usage (Donut Chart)
  const swapTotal = snapshot.swap_total_bytes || 0;
  const swapUsed = snapshot.swap_used_bytes || 0;
  const swapFree = swapTotal - swapUsed;
  const swapData = [
    { name: 'Used', value: swapUsed, color: '#995F2F' },
    { name: 'Free', value: swapFree, color: '#E4D6A9' },
  ];

  // Metric 3: CPU Per-Core (Bar Chart)
  const cpuCores = snapshot.cpu_cores || [];
  const cpuCoreData = cpuCores.map((usage, idx) => ({
    core: `Core ${idx}`,
    usage: parseFloat(usage.toFixed(1)),
  }));

  // Metric 4: Disk I/O (Metric Cards)
  const diskReadMB = (snapshot.disk_read_bytes || 0) / 1024 / 1024;
  const diskWriteMB = (snapshot.disk_write_bytes || 0) / 1024 / 1024;

  // Metric 5: Process State Distribution (Donut Chart)
  const processStateData = [
    { name: 'Running', value: snapshot.process_running || 0, color: '#622B14' },
    { name: 'Sleeping', value: snapshot.process_sleeping || 0, color: '#978F66' },
    { name: 'Zombie', value: snapshot.process_zombie || 0, color: '#D4A574' },
  ].filter(item => item.value > 0);

  // Metric 6: Uptime
  const uptime = formatUptime(snapshot.uptime_seconds || 0);

  // Metric 7: Context Switches
  const contextSwitches = (snapshot.context_switches || 0).toLocaleString();

  const colors = {
    primary: '#622B14',
    secondary: '#995F2F',
    tertiary: '#978F66',
    light: '#E4D6A9',
  };

  return (
    <div className="enhanced-metrics-container">
      {/* Row 1: System Load */}
      <div className="metric-section">
        <div className="metric-header">
          <Zap size={18} strokeWidth={1.5} />
          <span>System Load Averages</span>
        </div>
        <div className="metric-cards-row">
          {systemLoadData.map((item, idx) => (
            <div key={idx} className="metric-card-small">
              <div className="metric-label">{item.label}</div>
              <div className="metric-value" style={{ color: colors.primary }}>
                {item.value}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Row 2: Swap Usage (Donut) + CPU Per-Core (Bar) */}
      <div className="metric-row-two-col">
        {/* Swap Usage */}
        <div className="metric-section">
          <div className="metric-header">
            <HardDrive size={18} strokeWidth={1.5} />
            <span>Swap Usage</span>
          </div>
          {swapTotal > 0 ? (
            <ResponsiveContainer width="100%" height={200}>
              <PieChart>
                <Pie
                  data={swapData}
                  cx="50%"
                  cy="50%"
                  innerRadius={60}
                  outerRadius={80}
                  paddingAngle={2}
                  dataKey="value"
                >
                  {swapData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip
                  formatter={(value) => formatBytes(value)}
                  contentStyle={{
                    background: 'var(--color-surface)',
                    border: '1px solid var(--color-border)',
                    borderRadius: '4px',
                  }}
                />
                <Legend
                  wrapperStyle={{ fontSize: '12px', fontFamily: 'Inter, sans-serif' }}
                  formatter={(value, entry) => `${value}: ${formatBytes(entry.payload.value)}`}
                />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="metric-unavailable">No swap configured</div>
          )}
          <div className="metric-summary">
            {formatBytes(swapUsed)} / {formatBytes(swapTotal)} ({(snapshot.swap_used_percent || 0).toFixed(1)}%)
          </div>
        </div>

        {/* CPU Per-Core */}
        <div className="metric-section">
          <div className="metric-header">
            <Cpu size={18} strokeWidth={1.5} />
            <span>CPU Per-Core Usage</span>
          </div>
          {cpuCoreData.length > 0 ? (
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={cpuCoreData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                <XAxis
                  dataKey="core"
                  stroke="#978F66"
                  style={{ fontSize: '10px', fontFamily: 'Inter, sans-serif' }}
                  tick={{ fill: '#978F66' }}
                />
                <YAxis
                  stroke="#978F66"
                  style={{ fontSize: '10px', fontFamily: 'Inter, sans-serif' }}
                  tick={{ fill: '#978F66' }}
                  domain={[0, 100]}
                />
                <Tooltip
                  formatter={(value) => `${value}%`}
                  contentStyle={{
                    background: 'var(--color-surface)',
                    border: '1px solid var(--color-border)',
                    borderRadius: '4px',
                  }}
                />
                <Bar dataKey="usage" fill={colors.primary} />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div className="metric-unavailable">No per-core data available</div>
          )}
        </div>
      </div>

      {/* Row 3: Process State Distribution + Disk I/O */}
      <div className="metric-row-two-col">
        {/* Process States */}
        <div className="metric-section">
          <div className="metric-header">
            <Activity size={18} strokeWidth={1.5} />
            <span>Process State Distribution</span>
          </div>
          {processStateData.length > 0 ? (
            <ResponsiveContainer width="100%" height={200}>
              <PieChart>
                <Pie
                  data={processStateData}
                  cx="50%"
                  cy="50%"
                  innerRadius={60}
                  outerRadius={80}
                  paddingAngle={2}
                  dataKey="value"
                  label={(entry) => entry.value}
                >
                  {processStateData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip
                  contentStyle={{
                    background: 'var(--color-surface)',
                    border: '1px solid var(--color-border)',
                    borderRadius: '4px',
                  }}
                />
                <Legend wrapperStyle={{ fontSize: '12px', fontFamily: 'Inter, sans-serif' }} />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="metric-unavailable">No process data</div>
          )}
        </div>

        {/* Disk I/O + Additional Metrics */}
        <div className="metric-section">
          <div className="metric-header">
            <GitBranch size={18} strokeWidth={1.5} />
            <span>System Statistics</span>
          </div>
          <div className="metric-cards-grid">
            <div className="metric-card-stat">
              <div className="metric-label">
                <HardDrive size={14} />
                Disk Read (Total)
              </div>
              <div className="metric-value" style={{ color: colors.primary }}>
                {formatBytes(snapshot.disk_read_bytes || 0)}
              </div>
            </div>
            <div className="metric-card-stat">
              <div className="metric-label">
                <HardDrive size={14} />
                Disk Write (Total)
              </div>
              <div className="metric-value" style={{ color: colors.secondary }}>
                {formatBytes(snapshot.disk_write_bytes || 0)}
              </div>
            </div>
            <div className="metric-card-stat">
              <div className="metric-label">
                <Clock size={14} />
                System Uptime
              </div>
              <div className="metric-value" style={{ color: colors.tertiary }}>
                {uptime}
              </div>
            </div>
            <div className="metric-card-stat">
              <div className="metric-label">
                <Network size={14} />
                Context Switches
              </div>
              <div className="metric-value" style={{ color: colors.primary, fontSize: '16px' }}>
                {contextSwitches}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
