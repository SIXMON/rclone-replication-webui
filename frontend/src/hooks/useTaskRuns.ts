import { useQuery } from '@tanstack/react-query';
import * as api from '../api/runs';

export function useTaskRuns(taskId: string) {
  return useQuery({
    queryKey: ['runs', taskId],
    queryFn: () => api.getTaskRuns(taskId),
    enabled: !!taskId,
  });
}

export function useRun(runId: string) {
  return useQuery({
    queryKey: ['run', runId],
    queryFn: () => api.getRun(runId),
    enabled: !!runId,
  });
}
