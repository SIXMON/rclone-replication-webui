import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as api from '../api/notifications';
import { CreateChannelPayload } from '../types/notification';

export function useNotifications() {
  return useQuery({ queryKey: ['notifications'], queryFn: api.getChannels });
}

export function useCreateChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (payload: CreateChannelPayload) => api.createChannel(payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notifications'] }),
  });
}

export function useUpdateChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: CreateChannelPayload }) => api.updateChannel(id, payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notifications'] }),
  });
}

export function useDeleteChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteChannel(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notifications'] }),
  });
}

export function useTestChannel() {
  return useMutation({ mutationFn: (id: string) => api.testChannel(id) });
}
