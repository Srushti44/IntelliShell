/**
 * Loading spinner component
 */

import { Loader2 } from 'lucide-react';

export function LoadingSpinner({ size = 'md', text }) {
  const sizeMap = {
    sm: 'w-4 h-4',
    md: 'w-8 h-8',
    lg: 'w-12 h-12',
  };

  return (
    <div className="flex flex-col items-center justify-center gap-2">
      <Loader2 
        className={`${sizeMap[size]} animate-spin`}
        style={{ color: 'var(--color-primary)' }}
      />
      {text && <p className="text-sm" style={{ color: 'var(--color-text-muted)' }}>{text}</p>}
    </div>
  );
}
