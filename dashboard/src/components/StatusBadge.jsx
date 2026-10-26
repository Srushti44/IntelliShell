/**
 * Status badge component for displaying service status, risk levels, etc.
 */

import { cn, getRiskColor, getMethodColor } from '../lib/utils';

export function StatusBadge({ type = 'default', children, className }) {
  const colorStyle = type === 'risk' && typeof children === 'string'
    ? getRiskColor(children)
    : type === 'method' && typeof children === 'string'
    ? getMethodColor(children)
    : { 
        color: 'var(--color-text-muted)', 
        backgroundColor: 'var(--color-bg-subtle)', 
        borderColor: 'var(--color-border)' 
      };

  return (
    <span
      className={cn(
        'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border',
        className
      )}
      style={colorStyle}
    >
      {children}
    </span>
  );
}
