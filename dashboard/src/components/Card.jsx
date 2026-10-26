/**
 * Card component for dashboard sections
 */

import { cn } from '../lib/utils';

export function Card({ title, children, className, action }) {
  return (
    <div 
      className={cn('rounded-lg shadow-sm border', className)}
      style={{ backgroundColor: 'var(--offwhite-main)', borderColor: 'var(--color-border)' }}
    >
      {title && (
        <div 
          className="px-6 py-4 border-b flex items-center justify-between"
          style={{ borderColor: 'var(--color-border)' }}
        >
          <h3 
            className="text-lg font-semibold"
            style={{ color: 'var(--ink-dark)', fontFamily: 'var(--font-heading)' }}
          >
            {title}
          </h3>
          {action && <div>{action}</div>}
        </div>
      )}
      <div className="p-6">{children}</div>
    </div>
  );
}
