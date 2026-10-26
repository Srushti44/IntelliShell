/**
 * Hook for fetching command history
 */

import { useQuery } from '@tanstack/react-query';
import { rustApi } from '../lib/api';

export function useCommandHistory() {
  return useQuery({
    queryKey: ['command-history'],
    queryFn: () => rustApi.getHistory(),
    refetchInterval: 10000, // Refresh every 10 seconds
    retry: 2,
  });
}
