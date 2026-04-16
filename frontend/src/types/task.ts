export interface LastRunSummary {
  status: 'success' | 'failure' | 'running';
  started_at: string;
  duration_ms: number | null;
}

export interface Task {
  id: string;
  name: string;
  source_remote_id: string;
  source_remote_name: string;
  source_path: string;
  dest_remote_id: string;
  dest_remote_name: string;
  dest_path: string;
  cron_expression: string | null;
  enabled: boolean;
  rclone_flags: string[];
  notification_channel_id: string | null;
  notify_on: string[];
  max_retries: number;
  retry_delay_seconds: number;
  last_run: LastRunSummary | null;
  running: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateTaskPayload {
  name: string;
  source_remote_id: string;
  source_path: string;
  dest_remote_id: string;
  dest_path: string;
  cron_expression: string | null;
  enabled: boolean;
  rclone_flags: string[];
  notification_channel_id: string | null;
  notify_on: string[];
  max_retries: number;
  retry_delay_seconds: number;
}

export interface PatchTaskPayload {
  name?: string;
  cron_expression?: string | null;
  enabled?: boolean;
  rclone_flags?: string[];
  notification_channel_id?: string | null;
  notify_on?: string[];
  max_retries?: number;
  retry_delay_seconds?: number;
}
