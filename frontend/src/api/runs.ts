import client from './client';
import { TaskRun, TaskRunSummary } from '../types/taskRun';

export const getTaskRuns = (taskId: string) => client.get<TaskRunSummary[]>(`/tasks/${taskId}/runs`).then(r => r.data);
export const getRun = (runId: string) => client.get<TaskRun>(`/runs/${runId}`).then(r => r.data);
