import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as api from '../api/tasks';
import { CreateTaskPayload, PatchTaskPayload } from '../types/task';

export function useTasks() {
  return useQuery({ queryKey: ['tasks'], queryFn: api.getTasks });
}

export function useTask(id: string) {
  return useQuery({ queryKey: ['tasks', id], queryFn: () => api.getTask(id), enabled: !!id });
}

export function useCreateTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (payload: CreateTaskPayload) => api.createTask(payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  });
}

export function usePatchTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: PatchTaskPayload }) => api.patchTask(id, payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  });
}

export function useDeleteTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteTask(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  });
}

export function useTriggerTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.triggerTask(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  });
}

export function useRestoreTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.restoreTask(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['tasks'] }),
  });
}
