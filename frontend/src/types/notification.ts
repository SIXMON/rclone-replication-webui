export interface NotificationChannel {
  id: string;
  name: string;
  apprise_url: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
  task_count?: number;
}

export interface CreateChannelPayload {
  name: string;
  apprise_url: string;
  enabled: boolean;
}
