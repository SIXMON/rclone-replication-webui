import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as api from '../api/remotes';
import { CreateRemotePayload } from '../types/remote';

export function useRemotes() {
  return useQuery({ queryKey: ['remotes'], queryFn: api.getRemotes });
}

export function useRemote(id: string) {
  return useQuery({ queryKey: ['remotes', id], queryFn: () => api.getRemote(id), enabled: !!id });
}

export function useCreateRemote() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (payload: CreateRemotePayload) => api.createRemote(payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['remotes'] }),
  });
}

export function useUpdateRemote() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: CreateRemotePayload }) => api.updateRemote(id, payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['remotes'] }),
  });
}

export function useDeleteRemote() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteRemote(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['remotes'] }),
  });
}

export function useTestRemote() {
  return useMutation({ mutationFn: (id: string) => api.testRemote(id) });
}
