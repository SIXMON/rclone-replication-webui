import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Plus, Pencil, Play, RotateCcw, ToggleLeft, ToggleRight, Trash2 } from 'lucide-react';
import { useTasks, useDeleteTask, useTriggerTask, useRestoreTask, usePatchTask } from '../hooks/useTasks';
import { StatusBadge } from '../components/ui/StatusBadge';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { Tooltip } from '../components/ui/Tooltip';
import cronstrue from 'cronstrue';

function formatCron(expr: string | null): string {
  if (!expr) return '—';
  try { return cronstrue.toString(expr, { locale: 'fr' }); } catch { return expr; }
}

function formatDuration(ms: number | null): string {
  if (ms == null) return '';
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

export function TasksPage() {
  const { data: tasks, isLoading, error } = useTasks();
  const deleteTask = useDeleteTask();
  const triggerTask = useTriggerTask();
  const restoreTask = useRestoreTask();
  const patchTask = usePatchTask();
  const navigate = useNavigate();
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [restoreId, setRestoreId] = useState<string | null>(null);

  if (isLoading) return <div className="p-8 text-surface-400">Chargement...</div>;

  return (
    <div className="p-8 animate-fade-in">
      {/* Header */}
      <div className="flex items-end justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">Tâches de réplication</h1>
          <p className="text-sm text-surface-500 mt-1">
            Planifiez et suivez la synchronisation de vos fichiers entre stockages distants.
          </p>
        </div>
        <Tooltip content="Créer une nouvelle tâche de réplication">
          <button
            onClick={() => navigate('/tasks/new')}
            className="flex items-center gap-2 px-4 py-2.5 bg-brand-600 text-white rounded-lg hover:bg-brand-700 text-sm font-medium transition-colors shadow-sm"
          >
            <Plus size={16} /> Nouvelle tâche
          </button>
        </Tooltip>
      </div>

      {error && <div className="mb-4"><ErrorBanner message={(error as Error).message} /></div>}

      <div className="bg-white rounded-xl border border-surface-200 shadow-sm overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-surface-50 border-b border-surface-200">
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Nom</th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                <Tooltip content="Stockage source vers stockage destination" position="bottom">
                  <span className="cursor-help border-b border-dashed border-surface-400">Source / Destination</span>
                </Tooltip>
              </th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                <Tooltip content="Fréquence d'exécution automatique (expression cron)" position="bottom">
                  <span className="cursor-help border-b border-dashed border-surface-400">Planification</span>
                </Tooltip>
              </th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Dernier run</th>
              <th className="text-right px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-surface-100">
            {tasks?.map(task => {
              const statusVal = task.running ? 'running' : !task.enabled ? 'disabled' : task.last_run?.status ?? 'unknown';
              return (
                <tr key={task.id} className="hover:bg-surface-50/50 transition-colors">
                  <td className="px-5 py-3.5">
                    <button onClick={() => navigate(`/tasks/${task.id}`)} className="font-medium text-brand-700 hover:text-brand-800 hover:underline underline-offset-2">
                      {task.name}
                    </button>
                    <div className="mt-1"><StatusBadge status={statusVal as 'success' | 'failure' | 'running' | 'disabled' | 'unknown'} /></div>
                  </td>
                  <td className="px-5 py-3.5 text-xs font-mono">
                    <div className="text-surface-700">{task.source_remote_name}<span className="text-surface-300">:</span>{task.source_path}</div>
                    <div className="text-surface-300 text-[10px] my-0.5">vers</div>
                    <div className="text-surface-700">{task.dest_remote_name}<span className="text-surface-300">:</span>{task.dest_path}</div>
                  </td>
                  <td className="px-5 py-3.5 text-surface-600 text-xs">{formatCron(task.cron_expression)}</td>
                  <td className="px-5 py-3.5 text-xs text-surface-500">
                    {task.last_run ? (
                      <div>
                        <span>{new Date(task.last_run.started_at).toLocaleString('fr')}</span>
                        {task.last_run.duration_ms != null && (
                          <span className="text-surface-400 ml-1">({formatDuration(task.last_run.duration_ms)})</span>
                        )}
                      </div>
                    ) : <span className="text-surface-300">Jamais exécutée</span>}
                  </td>
                  <td className="px-5 py-3.5">
                    <div className="flex items-center justify-end gap-0.5">
                      <Tooltip content="Lancer la synchronisation maintenant">
                        <button onClick={() => triggerTask.mutate(task.id)} disabled={task.running} className="p-2 text-surface-400 hover:text-emerald-600 hover:bg-emerald-50 rounded-lg disabled:opacity-30 transition-colors">
                          <Play size={15} />
                        </button>
                      </Tooltip>
                      <Tooltip content="Restaurer : synchroniser en sens inverse">
                        <button onClick={() => setRestoreId(task.id)} disabled={task.running} className="p-2 text-surface-400 hover:text-orange-600 hover:bg-orange-50 rounded-lg disabled:opacity-30 transition-colors">
                          <RotateCcw size={15} />
                        </button>
                      </Tooltip>
                      <Tooltip content={task.enabled ? 'Désactiver la planification auto' : 'Activer la planification auto'}>
                        <button onClick={() => patchTask.mutate({ id: task.id, payload: { enabled: !task.enabled } })} className="p-2 text-surface-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-colors">
                          {task.enabled ? <ToggleRight size={15} /> : <ToggleLeft size={15} />}
                        </button>
                      </Tooltip>
                      <Tooltip content="Modifier les paramètres">
                        <button onClick={() => navigate(`/tasks/${task.id}/edit`)} className="p-2 text-surface-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-colors">
                          <Pencil size={15} />
                        </button>
                      </Tooltip>
                      <Tooltip content="Supprimer cette tâche">
                        <button onClick={() => setConfirmId(task.id)} className="p-2 text-surface-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors">
                          <Trash2 size={15} />
                        </button>
                      </Tooltip>
                    </div>
                  </td>
                </tr>
              );
            })}
            {!tasks?.length && (
              <tr>
                <td colSpan={5} className="px-5 py-12 text-center text-surface-400">
                  <p className="text-base font-medium mb-1">Aucune tâche configurée</p>
                  <p className="text-xs">Cliquez sur "Nouvelle tâche" pour planifier votre première réplication.</p>
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <ConfirmDialog
        open={!!confirmId}
        title="Supprimer cette tâche ?"
        message="La tâche et tout son historique d'exécutions seront supprimés définitivement."
        onConfirm={() => { if (confirmId) deleteTask.mutate(confirmId); setConfirmId(null); }}
        onCancel={() => setConfirmId(null)}
      />

      <ConfirmDialog
        open={!!restoreId}
        title="Lancer une restauration ?"
        message="La restauration synchronise en sens inverse : les fichiers de la destination écraseront ceux de la source. Cette opération peut entraîner une perte de données."
        confirmLabel="Restaurer"
        variant="warning"
        onConfirm={() => { if (restoreId) restoreTask.mutate(restoreId); setRestoreId(null); }}
        onCancel={() => setRestoreId(null)}
      />
    </div>
  );
}
