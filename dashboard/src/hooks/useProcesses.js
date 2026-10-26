/**
 * Hook for fetching process list
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { rustApi } from '../lib/api';

export function useProcesses() {
  return useQuery({
    queryKey: ['processes'],
    queryFn: () => rustApi.listProcesses(),
    refetchInterval: 5000, // Refresh every 5 seconds
    retry: 2,
  });
}

export function useControlProcess() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ pid, action }) => rustApi.controlProcess(pid, action),
    onSuccess: () => {
      // Invalidate and refetch process list
      queryClient.invalidateQueries({ queryKey: ['processes'] });
    },
  });
}
