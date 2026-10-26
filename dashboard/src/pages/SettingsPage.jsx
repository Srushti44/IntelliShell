import React from 'react';
import { PageSection } from '../components/PageSection';
import { AlertCircle, Server, Palette } from 'lucide-react';

/**
 * SettingsPage - Configuration and information
 * Currently informational only (no persistent settings yet)
 */
export function SettingsPage() {
  return (
    <div className="page-content">
      <PageSection
        title="Backend Configuration"
        description="Service endpoint information"
      >
        <div style={{
          padding: '20px',
          background: 'var(--color-surface)',
          border: '1px solid var(--color-border)',
          borderRadius: '4px',
          display: 'flex',
          flexDirection: 'column',
          gap: '16px'
        }}>
          <SettingRow 
            icon={<Server size={18} />}
            label="Rust Service"
            value="http://localhost:3000"
            description="System snapshots, process control, command execution"
          />
          <SettingRow 
            icon={<Server size={18} />}
            label="Python AI Service"
            value="http://localhost:8000"
            description="Natural language processing, command generation, risk analysis"
          />
        </div>
      </PageSection>

      <PageSection
        title="Design System"
        description="Swiss modernist color palette"
        style={{ marginTop: '32px' }}
      >
        <div style={{
          padding: '20px',
          background: 'var(--color-surface)',
          border: '1px solid var(--color-border)',
          borderRadius: '4px'
        }}>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
            gap: '16px'
          }}>
            <ColorSwatch color="#622B14" name="Deep Brown" variable="--color-brown-deep" />
            <ColorSwatch color="#995F2F" name="Terracotta" variable="--color-terracotta" />
            <ColorSwatch color="#978F66" name="Muted Olive" variable="--color-olive-muted" />
            <ColorSwatch color="#E4D6A9" name="Warm Cream" variable="--color-cream-warm" />
            <ColorSwatch color="#FAF8F2" name="Off-White" variable="--color-off-white" />
            <ColorSwatch color="#25231F" name="Dark Ink" variable="--color-ink-dark" />
          </div>
        </div>
      </PageSection>

      <PageSection
        title="About"
        description="IntelliShell version information"
        style={{ marginTop: '32px' }}
      >
        <div style={{
          padding: '20px',
          background: 'var(--color-surface)',
          border: '1px solid var(--color-border)',
          borderRadius: '4px'
        }}>
          <div style={{
            fontFamily: 'var(--font-heading)',
            fontSize: '16px',
            fontWeight: '600',
            color: 'var(--color-text-primary)',
            marginBottom: '8px'
          }}>
            IntelliShell v0.7.0
          </div>
          <div style={{
            fontFamily: 'var(--font-body)',
            fontSize: '14px',
            color: 'var(--color-text-secondary)',
            lineHeight: '1.6'
          }}>
            AI-Based System Intent Engine for Safe Linux Command Execution
            <br />
            Phase 7: Swiss Modernist Design Enhancement
          </div>
        </div>
      </PageSection>

      <div style={{
        marginTop: '32px',
        padding: '16px 20px',
        background: 'var(--color-surface)',
        border: '1px solid var(--color-border)',
        borderRadius: '4px',
        display: 'flex',
        alignItems: 'flex-start',
        gap: '12px'
      }}>
        <AlertCircle size={20} color="var(--color-terracotta)" style={{ flexShrink: 0, marginTop: '2px' }} />
        <div style={{
          fontFamily: 'var(--font-body)',
          fontSize: '13px',
          color: 'var(--color-text-secondary)',
          lineHeight: '1.5'
        }}>
          <strong style={{ color: 'var(--color-text-primary)' }}>Note:</strong> User preferences and persistent 
          settings will be added in future phases. Currently, all configuration is read from environment variables 
          and cannot be modified through this interface.
        </div>
      </div>
    </div>
  );
}

function SettingRow({ icon, label, value, description }) {
  return (
    <div style={{
      display: 'flex',
      alignItems: 'flex-start',
      gap: '12px',
      paddingBottom: '16px',
      borderBottom: '1px solid var(--color-border)'
    }}>
      <div style={{ color: 'var(--color-brown-deep)', marginTop: '2px' }}>
        {icon}
      </div>
      <div style={{ flex: 1 }}>
        <div style={{
          fontFamily: 'var(--font-heading)',
          fontSize: '14px',
          fontWeight: '600',
          color: 'var(--color-text-primary)',
          marginBottom: '4px'
        }}>
          {label}
        </div>
        <div style={{
          fontFamily: 'var(--font-body)',
          fontSize: '13px',
          color: 'var(--color-terracotta)',
          marginBottom: '6px',
          fontWeight: '500'
        }}>
          {value}
        </div>
        <div style={{
          fontFamily: 'var(--font-body)',
          fontSize: '12px',
          color: 'var(--color-text-secondary)',
          lineHeight: '1.5'
        }}>
          {description}
        </div>
      </div>
    </div>
  );
}

function ColorSwatch({ color, name, variable }) {
  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      gap: '8px'
    }}>
      <div style={{
        width: '100%',
        height: '80px',
        backgroundColor: color,
        border: '1px solid var(--color-border)',
        borderRadius: '2px'
      }} />
      <div>
        <div style={{
          fontFamily: 'var(--font-heading)',
          fontSize: '13px',
          fontWeight: '600',
          color: 'var(--color-text-primary)',
          marginBottom: '2px'
        }}>
          {name}
        </div>
        <div style={{
          fontFamily: 'var(--font-body)',
          fontSize: '11px',
          color: 'var(--color-text-secondary)',
          fontFamily: 'monospace'
        }}>
          {color}
        </div>
      </div>
    </div>
  );
}


export default SettingsPage;
