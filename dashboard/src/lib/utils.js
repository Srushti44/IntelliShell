/**
 * Utility functions for the dashboard
 */

import { clsx } from 'clsx';

/**
 * Merge class names conditionally
 */
export function cn(...inputs) {
  return clsx(inputs);
}

/**
 * Format bytes to human-readable string
 */
export function formatBytes(bytes, decimals = 2) {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

/**
 * Format percentage with fixed decimals
 */
export function formatPercent(value, decimals = 1) {
  return value.toFixed(decimals) + '%';
}

/**
 * Format duration in milliseconds
 */
export function formatDuration(ms) {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
}

/**
 * Format Unix timestamp to relative time
 */
export function formatRelativeTime(timestamp) {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;

  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

/**
 * Get color style object for risk level
 */
export function getRiskColor(riskLevel) {
  const styles = {
    LOW: { 
      color: 'var(--color-success)', 
      backgroundColor: 'var(--color-success-bg)', 
      borderColor: 'var(--color-success)' 
    },
    MEDIUM: { 
      color: 'var(--color-warning)', 
      backgroundColor: 'var(--color-warning-bg)', 
      borderColor: 'var(--color-warning)' 
    },
    HIGH: { 
      color: 'var(--color-error)', 
      backgroundColor: 'var(--color-error-bg)', 
      borderColor: 'var(--color-error)' 
    },
  };
  return styles[riskLevel] || { 
    color: 'var(--color-text-muted)', 
    backgroundColor: 'var(--color-bg-subtle)', 
    borderColor: 'var(--color-border)' 
  };
}

/**
 * Get color style object for execution method
 */
export function getMethodColor(method) {
  const styles = {
    direct: { 
      color: 'var(--color-primary)', 
      backgroundColor: 'var(--color-info-bg)' 
    },
    sandboxed: { 
      color: 'var(--olive-muted)', 
      backgroundColor: '#F0EFE8' 
    },
    blocked: { 
      color: 'var(--color-error)', 
      backgroundColor: 'var(--color-error-bg)' 
    },
  };
  return styles[method] || { 
    color: 'var(--color-text-muted)', 
    backgroundColor: 'var(--color-bg-subtle)' 
  };
}

/**
 * Parse command string into command and args
 */
export function parseCommand(commandString) {
  const parts = commandString.trim().split(/\s+/);
  const command = parts[0];
  const args = parts.slice(1);
  return { command, args };
}

/**
 * Truncate text with ellipsis
 */
export function truncate(text, maxLength) {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength) + '...';
}

/**
 * Check if timestamp is expired
 */
export function isExpired(expiresAt) {
  return new Date(expiresAt) < new Date();
}

/**
 * Get time remaining until expiry
 */
export function getTimeRemaining(expiresAt) {
  const now = new Date();
  const expiry = new Date(expiresAt);
  const diff = expiry - now;

  if (diff <= 0) return 'Expired';

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;

  if (minutes > 0) {
    return `${minutes}m ${remainingSeconds}s`;
  }
  return `${remainingSeconds}s`;
}
