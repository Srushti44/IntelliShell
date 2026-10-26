import React, { useState, useEffect, useMemo } from 'react';
import { LineChart, Line, AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { Activity, Cpu, HardDrive, Wifi } from 'lucide-react';
import '../styles/SystemCharts.css';

/**
 * SystemCharts - Real-time telemetry visualization using Swiss design
 * Uses client-side rolling window to display recent system metrics
 * No mock data - all values come from live backend snapshots
 */
export function SystemCharts({ currentSnapshot, className = '' }) {
  // Rolling window of last 20 snapshots (client-side history)
  const [history, setHistory] = useState([]);
  const maxDataPoints = 20;

  // Update history when new snapshot arrives via prop
  useEffect(() => {
    if (!currentSnapshot?.system) return;

    const dataPoint = {
      timestamp: new Date().toLocaleTimeString('en-US', { 
        hour12: false, 
        hour: '2-digit', 
        minute: '2-digit', 
        second: '2-digit' 
      }),
      cpuUsage: currentSnapshot.system.cpu_usage || 0,
      memoryUsage: currentSnapshot.system.memory_used_percent || 0,
      diskUsage: currentSnapshot.system.disk_used_percent || 0,
      networkRx: (currentSnapshot.system.network_rx_bytes || 0) / 1024 / 1024, // MB
      networkTx: (currentSnapshot.system.network_tx_bytes || 0) / 1024 / 1024, // MB
      processCount: currentSnapshot.processes?.length || 0
    };

    setHistory(prev => {
      const updated = [...prev, dataPoint];
      // Keep only last N data points
      return updated.slice(-maxDataPoints);
    });
  }, [currentSnapshot]);

  // Self-polling when no currentSnapshot prop is provided
  useEffect(() => {
    // Only poll if no snapshot prop is provided
    if (currentSnapshot !== undefined) return;

    const pollInterval = 2000; // 2 seconds
    let intervalId;

    const fetchSnapshot = async () => {
      try {
        const response = await fetch('http://localhost:3000/snapshot');
        if (!response.ok) return;
        
        const data = await response.json();
        
        const dataPoint = {
          timestamp: new Date().toLocaleTimeString('en-US', { 
            hour12: false, 
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit' 
          }),
          cpuUsage: data.system?.cpu_usage || 0,
          memoryUsage: data.system?.memory_used_percent || 0,
          diskUsage: data.system?.disk_used_percent || 0,
          networkRx: (data.system?.network_rx_bytes || 0) / 1024 / 1024, // MB
          networkTx: (data.system?.network_tx_bytes || 0) / 1024 / 1024, // MB
          processCount: data.processes?.length || 0
        };

        setHistory(prev => {
          const updated = [...prev, dataPoint];
          return updated.slice(-maxDataPoints);
        });
      } catch (error) {
        console.error('Failed to fetch snapshot:', error);
      }
    };

    // Fetch immediately
    fetchSnapshot();
    
    // Then poll every 2 seconds
    intervalId = setInterval(fetchSnapshot, pollInterval);

    return () => {
      if (intervalId) clearInterval(intervalId);
    };
  }, [currentSnapshot]);

  // Chart color scheme using Swiss palette
  const colors = {
    cpu: '#622B14',      // Deep brown
    memory: '#995F2F',   // Terracotta
    disk: '#978F66',     // Muted olive
    networkRx: '#622B14',
    networkTx: '#995F2F',
    processes: '#978F66'
  };

  // Custom tooltip matching Swiss design
  const CustomTooltip = ({ active, payload, label }) => {
    if (!active || !payload || !payload.length) return null;
    
    return (
      <div className="swiss-chart-tooltip">
        <p className="tooltip-label">{label}</p>
        {payload.map((entry, index) => (
          <p key={index} style={{ color: entry.color }}>
            {entry.name}: {typeof entry.value === 'number' ? entry.value.toFixed(2) : entry.value}
            {entry.unit || ''}
          </p>
        ))}
      </div>
    );
  };

  // Show placeholder when not enough data yet
  if (history.length < 2) {
    return (
      <div className={`system-charts ${className}`}>
        <div className="chart-placeholder">
          <Activity size={48} strokeWidth={1.5} />
          <p>Collecting telemetry data...</p>
          <p className="placeholder-hint">Charts will appear after 2+ snapshots</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`system-charts ${className}`}>
      {/* CPU & Memory Chart */}
      <div className="chart-container">
        <div className="chart-header">
          <div className="chart-title">
            <Cpu size={18} strokeWidth={1.5} />
            <span>CPU & Memory Usage</span>
          </div>
          <span className="chart-unit">Percentage (%)</span>
        </div>
        <ResponsiveContainer width="100%" height={240}>
          <AreaChart data={history} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={colors.cpu} stopOpacity={0.2}/>
                <stop offset="95%" stopColor={colors.cpu} stopOpacity={0}/>
              </linearGradient>
              <linearGradient id="colorMemory" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={colors.memory} stopOpacity={0.2}/>
                <stop offset="95%" stopColor={colors.memory} stopOpacity={0}/>
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="#E4D6A9" opacity={0.3} />
            <XAxis 
              dataKey="timestamp" 
              stroke="#978F66"
              style={{ fontSize: '11px', fontFamily: 'Inter, sans-serif' }}
              tick={{ fill: '#978F66' }}
            />
            <YAxis 
              stroke="#978F66"
              style={{ fontSize: '11px', fontFamily: 'Inter, sans-serif' }}
              tick={{ fill: '#978F66' }}
              domain={[0, 100]}
            />
            <Tooltip content={<CustomTooltip />} />
            <Legend 
              wrapperStyle={{ fontSize: '12px', fontFamily: 'Inter, sans-serif' }}
              iconType="line"
            />
            <Area 
              type="monotone" 
              dataKey="cpuUsage" 
              stroke={colors.cpu} 
              fillOpacity={1} 
              fill="url(#colorCpu)" 
              name="CPU"
              strokeWidth={2}
            />
            <Area 
              type="monotone" 
              dataKey="memoryUsage" 
              stroke={colors.memory} 
              fillOpacity={1} 
              fill="url(#colorMemory)" 
              name="Memory"
              strokeWidth={2}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* Network Traffic Chart */}
      <div className="chart-container">
        <div className="chart-header">
          <div className="chart-title">
            <Wifi size={18} strokeWidth={1.5} />
            <span>Network Traffic</span>
          </div>
          <span className="chart-unit">MB transferred</span>
        </div>
        <ResponsiveContainer width="100%" height={240}>
          <LineChart data={history} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#E4D6A9" opacity={0.3} />
            <XAxis 
              dataKey="timestamp" 
              stroke="#978F66"
              style={{ fontSize: '11px', fontFamily: 'Inter, sans-serif' }}
              tick={{ fill: '#978F66' }}
            />
            <YAxis 
              stroke="#978F66"
              style={{ fontSize: '11px', fontFamily: 'Inter, sans-serif' }}
              tick={{ fill: '#978F66' }}
            />
            <Tooltip content={<CustomTooltip />} />
            <Legend 
              wrapperStyle={{ fontSize: '12px', fontFamily: 'Inter, sans-serif' }}
              iconType="line"
            />
            <Line 
              type="monotone" 
              dataKey="networkRx" 
              stroke={colors.networkRx} 
              name="RX (received)"
              strokeWidth={2}
              dot={{ r: 3 }}
            />
            <Line 
              type="monotone" 
              dataKey="networkTx" 
              stroke={colors.networkTx} 
              name="TX (sent)"
              strokeWidth={2}
              dot={{ r: 3 }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Disk & Process Count Chart */}
      <div className="chart-container">
        <div className="chart-header">
          <div className="chart-title">
            <HardDrive size={18} strokeWidth={1.5} />
            <span>Disk Usage & Process Count</span>
          </div>
          <span className="chart-unit">Mixed scale</span>
        </div>
        <ResponsiveContainer width="100%" height={240}>
          <LineChart data={history} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#E4D6A9" opacity={0.3} />
            <XAxis 
              dataKey="timestamp" 
              stroke="#978F66"
              style={{ fontSize: '11px', fontFamily: 'Inter, sans-serif' }}
              tick={{ fill: '#978F66' }}
            />
            <YAxis 
              yAxisId="left"
              stroke="#978F66"
              style={{ fontSize: '11px', fontFamily: 'Inter, sans-serif' }}
              tick={{ fill: '#978F66' }}
              domain={[0, 100]}
              label={{ value: 'Disk %', angle: -90, position: 'insideLeft', style: { fill: '#978F66', fontSize: '11px' } }}
            />
            <YAxis 
              yAxisId="right"
              orientation="right"
              stroke="#978F66"
              style={{ fontSize: '11px', fontFamily: 'Inter, sans-serif' }}
              tick={{ fill: '#978F66' }}
              label={{ value: 'Process Count', angle: 90, position: 'insideRight', style: { fill: '#978F66', fontSize: '11px' } }}
            />
            <Tooltip content={<CustomTooltip />} />
            <Legend 
              wrapperStyle={{ fontSize: '12px', fontFamily: 'Inter, sans-serif' }}
              iconType="line"
            />
            <Line 
              yAxisId="left"
              type="monotone" 
              dataKey="diskUsage" 
              stroke={colors.disk} 
              name="Disk Usage (%)"
              strokeWidth={2}
              dot={{ r: 3 }}
            />
            <Line 
              yAxisId="right"
              type="monotone" 
              dataKey="processCount" 
              stroke={colors.processes} 
              name="Processes"
              strokeWidth={2}
              dot={{ r: 3 }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}


export default SystemCharts;
