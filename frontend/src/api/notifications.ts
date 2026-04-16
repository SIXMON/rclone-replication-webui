import client from './client';
import { NotificationChannel, CreateChannelPayload } from '../types/notification';

export const getChannels = () => client.get<NotificationChannel[]>('/notifications').then(r => r.data);
export const getChannel = (id: string) => client.get<NotificationChannel>(`/notifications/${id}`).then(r => r.data);
export const createChannel = (payload: CreateChannelPayload) => client.post<NotificationChannel>('/notifications', payload).then(r => r.data);
export const updateChannel = (id: string, payload: CreateChannelPayload) => client.put<NotificationChannel>(`/notifications/${id}`, payload).then(r => r.data);
export const deleteChannel = (id: string) => client.delete(`/notifications/${id}`);
export const testChannel = (id: string) => client.post<{ success: boolean; message: string }>(`/notifications/${id}/test`).then(r => r.data);
