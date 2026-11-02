/**
 * ProjectOverview Component
 * Showcases IntelliShell features and capabilities
 */

import React from 'react';
import { 
  Brain, 
  Shield, 
  Activity, 
  Cpu, 
  GitBranch, 
  Search,
  History,
  Terminal
} from 'lucide-react';
import '../styles/ProjectOverview.css';

const features = [
  {
    icon: Brain,
    title: 'AI-Powered Command Generation',
    description: 'Convert natural-language requests into OS-compatible commands. Support for Linux and Windows execution environments.',
    color: 'var(--brown-deep)'
  },
  {
    icon: Terminal,
    title: 'Intelligent Command Explanations',
    description: 'Explain generated commands in understandable language. Clarify what each command does before execution.',
    color: 'var(--brown-terracotta)'
  },
  {
    icon: Shield,
    title: 'AI-Based Risk Assessment',
    description: 'SAFE, MEDIUM RISK and HIGH RISK classification with contextual explanations of potential system impact.',
    color: 'var(--color-error)'
  },
  {
    icon: Activity,
    title: 'Real-Time System Monitoring',
    description: 'Monitor CPU, RAM, disk, network and 13+ system metrics. Dynamically visualize system activity every 2 seconds.',
    color: 'var(--color-success)'
  },
  {
    icon: Cpu,
    title: 'Process Management',
    description: 'View running processes and their resource consumption. Search, inspect and control processes with protection.',
    color: 'var(--olive-muted)'
  },
  {
    icon: GitBranch,
    title: 'Dual Operating System Support',
    description: 'Separate Linux (WSL2) and Windows (host) command generation and execution environments with platform-specific telemetry.',
    color: 'var(--brown-terracotta)'
  },
  {
    icon: Search,
    title: 'AI Troubleshooting',
    description: 'Analyse system information and help investigate problems. Provide contextual troubleshooting guidance.',
    color: 'var(--color-info)'
  },
  {
    icon: History,
    title: 'Command History',
    description: 'Maintain a history of executed commands and their outputs with execution status tracking.',
    color: 'var(--olive-muted)'
  }
];

export function ProjectOverview() {
  return (
    <section className="project-overview-section">
      {/* Header */}
      <div className="section-header">
        <div className="header-badge">IntelliShell Project</div>
        <h1 className="section-title">
          Intelligent Command Line & System Management
        </h1>
        <p className="section-description">
          IntelliShell is an AI-powered command-line assistant that translates natural-language 
          instructions into operating-system commands, explains their functionality, assesses 
          execution risks and enables controlled execution. It combines AI-driven command generation 
          with real-time system monitoring and process management to make system interaction more 
          accessible, transparent and secure.
        </p>
      </div>

      {/* Features Grid */}
      <div className="features-grid">
        {features.map((feature, index) => (
          <FeatureCard key={index} feature={feature} index={index} />
        ))}
      </div>
    </section>
  );
}

function FeatureCard({ feature, index }) {
  const Icon = feature.icon;

  return (
    <div 
      className="feature-card"
      style={{ animationDelay: `${index * 0.05}s` }}
    >
      <div className="feature-icon-container" style={{ backgroundColor: `${feature.color}15` }}>
        <Icon 
          className="feature-icon" 
          size={24}
          strokeWidth={1.5}
          style={{ color: feature.color }}
        />
      </div>
      <div className="feature-content">
        <h3 className="feature-title">{feature.title}</h3>
        <p className="feature-description">{feature.description}</p>
      </div>
    </div>
  );
}
