export interface Remote {
  id: string;
  name: string;
  remote_type: string;
  config: Record<string, string>;
  created_at: string;
  updated_at: string;
  task_count?: number;
}

export interface CreateRemotePayload {
  name: string;
  remote_type: string;
  config: Record<string, string>;
}
