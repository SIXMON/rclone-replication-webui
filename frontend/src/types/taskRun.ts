/** Statistiques extraites des logs JSON rclone (dernière ligne "stats") */
export interface RcloneStats {
  bytes: number;
  checks: number;
  deletedDirs: number;
  deletes: number;
  elapsedTime: number;
  errors: number;
  fatalError: boolean;
  renames: number;
  retryError: boolean;
  speed: number;
  totalBytes: number;
  totalChecks: number;
  totalTransfers: number;
  transferTime: number;
  transfers: number;
}

export interface TaskRunSummary {
  id: string;
  task_id: string;
  triggered_by: 'scheduler' | 'manual' | 'restore';
  status: 'running' | 'success' | 'failure';
  started_at: string;
  finished_at: string | null;
  duration_ms: number | null;
  exit_code: number | null;
  stats: RcloneStats | null;
}

export interface TaskRun extends TaskRunSummary {
  log_output: string | null;
}
