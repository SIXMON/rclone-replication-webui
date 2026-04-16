import client from './client';
import { Task, CreateTaskPayload, PatchTaskPayload } from '../types/task';

export const getTasks = () => client.get<Task[]>('/tasks').then(r => r.data);
export const getTask = (id: string) => client.get<Task>(`/tasks/${id}`).then(r => r.data);
export const createTask = (payload: CreateTaskPayload) => client.post<Task>('/tasks', payload).then(r => r.data);
export const patchTask = (id: string, payload: PatchTaskPayload) => client.patch<Task>(`/tasks/${id}`, payload).then(r => r.data);
export const deleteTask = (id: string) => client.delete(`/tasks/${id}`);
export const triggerTask = (id: string) => client.post<{ run_id: string; message: string }>(`/tasks/${id}/trigger`).then(r => r.data);
export const restoreTask = (id: string) => client.post<{ run_id: string; message: string }>(`/tasks/${id}/restore`).then(r => r.data);
export const getTaskStatus = (id: string) => client.get<{ running: boolean; run_id?: string }>(`/tasks/${id}/status`).then(r => r.data);
