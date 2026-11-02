/**
 * Hook for fetching system monitoring snapshot
 * @param {string} os - Operating system: "linux" or "windows"
 */

import { useQuery } from '@tanstack/react-query';
import { rustApi } from '../lib/api';

export function useSystemSnapshot(os = 'linux') {
  return useQuery({
    queryKey: ['system-snapshot', os],
    queryFn: () => rustApi.getSnapshot(os),
    refetchInterval: 2000, // Refresh every 2 seconds
    retry: 2,
    staleTime: 1000, // Consider data stale after 1 second to ensure cache is used between polls
  });
}
