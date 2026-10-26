/**
 * Hook for monitoring backend service health
 */

import { useQuery } from '@tanstack/react-query';
import { checkAllServices } from '../lib/api';

export function useBackendHealth() {
  return useQuery({
    queryKey: ['backend-health'],
    queryFn: checkAllServices,
    refetchInterval: 10000, // Check every 10 seconds
    retry: 1,
    staleTime: 5000,
  });
}
