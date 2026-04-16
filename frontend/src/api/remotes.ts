import client from './client';
import { Remote, CreateRemotePayload } from '../types/remote';

export const getRemotes = () => client.get<Remote[]>('/remotes').then(r => r.data);
export const getRemote = (id: string) => client.get<Remote>(`/remotes/${id}`).then(r => r.data);
export const createRemote = (payload: CreateRemotePayload) => client.post<Remote>('/remotes', payload).then(r => r.data);
export const updateRemote = (id: string, payload: CreateRemotePayload) => client.put<Remote>(`/remotes/${id}`, payload).then(r => r.data);
export const deleteRemote = (id: string) => client.delete(`/remotes/${id}`);
export const testRemote = (id: string) => client.post<{ success: boolean; message: string }>(`/remotes/${id}/test`).then(r => r.data);
