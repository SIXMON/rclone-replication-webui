import React, { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Play, RotateCcw, Pencil, ChevronDown, ChevronRight, ArrowLeft } from 'lucide-react';
import { useTask, useTriggerTask, useRestoreTask } from '../hooks/useTasks';
import { useTaskRuns, useRun } from '../hooks/useTaskRuns';
import { useTaskProgress } from '../hooks/useTaskProgress';
import { LiveProgressPanel } from '../components/progress/LiveProgressPanel';
import { StatusBadge } from '../components/ui/StatusBadge';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { Tooltip } from '../components/ui/Tooltip';
import type { RcloneStats } from '../types/taskRun';
import cronstrue from 'cronstrue';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 o';
  const units = ['o', 'Ko', 'Mo', 'Go', 'To'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[i]}`;
}

function StatsCell({ stats }: { stats: RcloneStats | null }) {
  if (!stats) return <span className="text-surface-300">—</span>;
  return (
    <div className="flex flex-wrap gap-x-3 gap-y-0.5 text-xs">
      <Tooltip content="Nombre de fichiers transférés sur le total">
        <span className="cursor-help">
          <span className="text-surface-400">Transf.</span>{' '}
          <span className="font-medium text-surface-700">{stats.transfers}/{stats.totalTransfers}</span>
        </span>
      </Tooltip>
      <Tooltip content="Volume total de données transférées">
        <span className="cursor-help">
          <span className="text-surface-400">Vol.</span>{' '}
          <span className="font-medium text-surface-700">{formatBytes(stats.bytes)}</span>
        </span>
      </Tooltip>
      <Tooltip content="Fichiers vérifiés (comparés sans transfert)">
        <span className="cursor-help">
          <span className="text-surface-400">Vérif.</span>{' '}
          <span className="font-medium text-surface-700">{stats.checks}</span>
        </span>
      </Tooltip>
      {stats.deletes > 0 && (
        <Tooltip content="Fichiers supprimés sur la destination">
          <span className="cursor-help">
            <span className="text-surface-400">Suppr.</span>{' '}
            <span className="font-medium text-orange-600">{stats.deletes}</span>
          </span>
        </Tooltip>
      )}
      {stats.errors > 0 && (
        <Tooltip content="Erreurs rencontrées pendant le transfert">
          <span className="cursor-help">
            <span className="text-surface-400">Err.</span>{' '}
            <span className="font-medium text-red-600">{stats.errors}</span>
          </span>
        </Tooltip>
      )}
    </div>
  );
}

export function TaskDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: task, isLoading } = useTask(id || '');
  const { data: runs } = useTaskRuns(id || '');
  const triggerTask = useTriggerTask();
  const restoreTask = useRestoreTask();
  const [showRestoreConfirm, setShowRestoreConfirm] = useState(false);
  const [forceConnect, setForceConnect] = useState(false);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const { data: runDetail } = useRun(selectedRunId || '');
  const progress = useTaskProgress(id || null, task?.running || false, forceConnect);

  // Réinitialiser forceConnect quand la tâche est reconnue comme running par le cache
  // ou quand la progression est terminée
  if (forceConnect && (task?.running || progress.done)) {
    setForceConnect(false);
  }

  if (isLoading || !task) return <div className="p-8 text-surface-400">Chargement...</div>;

  const cronLabel = task.cron_expression ? (() => { try { return cronstrue.toString(task.cron_expression, { locale: 'fr' }); } catch { return task.cron_expression; } })() : 'Manuel uniquement';

  return (
    <div className="p-8 space-y-6 max-w-5xl animate-fade-in">
      {/* Back link */}
      <button onClick={() => navigate('/tasks')} className="flex items-center gap-1.5 text-sm text-surface-500 hover:text-brand-600 transition-colors">
        <ArrowLeft size={14} /> Retour aux tâches
      </button>

      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">{task.name}</h1>
          <div className="mt-1.5 flex items-center gap-3">
            <StatusBadge status={task.running ? 'running' : !task.enabled ? 'disabled' : task.last_run?.status ?? 'unknown'} />
            <span className="text-sm text-surface-500">{cronLabel}</span>
          </div>
        </div>
        <div className="flex gap-2">
          <Tooltip content="Lancer la synchronisation maintenant">
            <button onClick={() => { triggerTask.mutate(task.id); setForceConnect(true); }} disabled={task.running}
              className="flex items-center gap-1.5 px-3.5 py-2 bg-emerald-600 text-white rounded-lg text-sm font-medium hover:bg-emerald-700 disabled:opacity-40 transition-colors shadow-sm">
              <Play size={14} /> Lancer
            </button>
          </Tooltip>
          <Tooltip content="Synchroniser en sens inverse (destination vers source)">
            <button onClick={() => setShowRestoreConfirm(true)} disabled={task.running}
              className="flex items-center gap-1.5 px-3.5 py-2 bg-orange-600 text-white rounded-lg text-sm font-medium hover:bg-orange-700 disabled:opacity-40 transition-colors shadow-sm">
              <RotateCcw size={14} /> Restaurer
            </button>
          </Tooltip>
          <Tooltip content="Modifier le nom, la planification ou les options">
            <button onClick={() => navigate(`/tasks/${task.id}/edit`)}
              className="flex items-center gap-1.5 px-3.5 py-2 border border-surface-300 text-surface-600 rounded-lg text-sm font-medium hover:bg-surface-50 transition-colors">
              <Pencil size={14} /> Modifier
            </button>
          </Tooltip>
        </div>
      </div>

      {/* Config card */}
      <div className="bg-white rounded-xl border border-surface-200 p-5 shadow-sm">
        <p className="text-xs font-semibold text-surface-500 uppercase tracking-wider mb-3">Configuration</p>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-surface-400 text-xs">Source</span>
            <p className="font-mono mt-0.5 text-surface-800">{task.source_remote_name}:{task.source_path}</p>
          </div>
          <div>
            <span className="text-surface-400 text-xs">Destination</span>
            <p className="font-mono mt-0.5 text-surface-800">{task.dest_remote_name}:{task.dest_path}</p>
          </div>
          {task.rclone_flags.length > 0 && (
            <div className="col-span-2">
              <span className="text-surface-400 text-xs">Options rclone supplémentaires</span>
              <p className="font-mono mt-0.5 text-surface-800">{task.rclone_flags.join(' ')}</p>
            </div>
          )}
        </div>
      </div>

      {/* Live progress */}
      {(task.running || progress.lines.length > 0) && (
        <LiveProgressPanel lines={progress.lines} done={progress.done} status={progress.status} />
      )}

      {/* History */}
      <div>
        <div className="flex items-center gap-2 mb-3">
          <h2 className="text-lg font-semibold text-surface-800">Historique des exécutions</h2>
          <span className="text-xs text-surface-400">({runs?.length ?? 0} / 100 max)</span>
        </div>
        <div className="bg-white rounded-xl border border-surface-200 shadow-sm overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-surface-50 border-b border-surface-200">
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Date</th>
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                  <Tooltip content="Comment l'exécution a été déclenchée" position="bottom">
                    <span className="cursor-help border-b border-dashed border-surface-400">Déclencheur</span>
                  </Tooltip>
                </th>
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Statut</th>
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Durée</th>
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                  <Tooltip content="Fichiers transférés, volume, vérifications, erreurs" position="bottom">
                    <span className="cursor-help border-b border-dashed border-surface-400">Statistiques</span>
                  </Tooltip>
                </th>
                <th className="text-right px-5 py-3"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-100">
              {runs?.map(run => {
                const isExpanded = run.id === selectedRunId;
                return (
                  <React.Fragment key={run.id}>
                    <tr
                      onClick={() => setSelectedRunId(isExpanded ? null : run.id)}
                      className={`cursor-pointer transition-colors ${isExpanded ? 'bg-surface-100' : 'hover:bg-surface-50/50'}`}
                    >
                      <td className="px-5 py-3 text-surface-700 whitespace-nowrap">{new Date(run.started_at).toLocaleString('fr')}</td>
                      <td className="px-5 py-3">
                        <span className="inline-flex px-2 py-0.5 rounded bg-surface-100 text-surface-600 text-xs font-medium">
                          {run.triggered_by === 'manual' ? 'Manuel' : run.triggered_by === 'scheduler' ? 'Planifié' : 'Restauration'}
                        </span>
                      </td>
                      <td className="px-5 py-3"><StatusBadge status={run.status as 'success' | 'failure' | 'running' | 'disabled' | 'unknown'} /></td>
                      <td className="px-5 py-3 text-surface-500 whitespace-nowrap">{run.duration_ms != null ? `${(run.duration_ms / 1000).toFixed(1)}s` : '—'}</td>
                      <td className="px-5 py-3"><StatsCell stats={run.stats} /></td>
                      <td className="px-5 py-3 text-right">
                        <span className="text-brand-600 text-xs flex items-center gap-0.5 ml-auto font-medium">
                          Logs {isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                        </span>
                      </td>
                    </tr>
                    {isExpanded && (
                      <tr>
                        <td colSpan={6} className="p-0 border-t-0">
                          <div className="bg-surface-950 px-5 py-4 font-mono text-xs text-surface-300 whitespace-pre-wrap max-h-72 overflow-y-auto animate-slide-down">
                            {runDetail?.log_output || <span className="text-surface-600 animate-pulse">Chargement des logs...</span>}
                          </div>
                        </td>
                      </tr>
                    )}
                  </React.Fragment>
                );
              })}
              {!runs?.length && (
                <tr>
                  <td colSpan={6} className="px-5 py-12 text-center text-surface-400">
                    <p className="text-base font-medium mb-1">Aucune exécution</p>
                    <p className="text-xs">Cliquez sur "Lancer" pour déclencher la première synchronisation.</p>
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      <ConfirmDialog
        open={showRestoreConfirm}
        title="Lancer une restauration ?"
        message="La restauration synchronise en sens inverse : les fichiers de la destination écraseront ceux de la source. Cette opération peut entraîner une perte de données."
        confirmLabel="Restaurer"
        variant="warning"
        onConfirm={() => { restoreTask.mutate(task.id); setShowRestoreConfirm(false); setForceConnect(true); }}
        onCancel={() => setShowRestoreConfirm(false)}
      />
    </div>
  );
}
