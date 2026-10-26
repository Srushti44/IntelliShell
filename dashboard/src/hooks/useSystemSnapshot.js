/**
 * Hook for fetching system monitoring snapshot
 */

import { useQuery } from '@tanstack/react-query';
import { rustApi } from '../lib/api';

export function useSystemSnapshot() {
  return useQuery({
    queryKey: ['system-snapshot'],
    queryFn: () => rustApi.getSnapshot(),
    refetchInterval: 2000, // Refresh every 2 seconds
    retry: 2,
    staleTime: 1000, // Consider data stale after 1 second to ensure cache is used between polls
  });
}
