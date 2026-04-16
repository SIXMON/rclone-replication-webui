import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Plus, Pencil, Trash2, Wifi } from 'lucide-react';
import { useRemotes, useDeleteRemote, useTestRemote } from '../hooks/useRemotes';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { Tooltip } from '../components/ui/Tooltip';

export function RemotesPage() {
  const { data: remotes, isLoading, error } = useRemotes();
  const deleteRemote = useDeleteRemote();
  const testRemote = useTestRemote();
  const navigate = useNavigate();
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<Record<string, { success: boolean; message: string }>>({});

  const handleTest = async (id: string) => {
    const result = await testRemote.mutateAsync(id);
    setTestResult(prev => ({ ...prev, [id]: result }));
  };

  if (isLoading) return <div className="p-8 text-surface-400">Chargement...</div>;

  return (
    <div className="p-8 max-w-5xl animate-fade-in">
      {/* Header */}
      <div className="flex items-end justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">Stockages distants</h1>
          <p className="text-sm text-surface-500 mt-1">
            Configurez vos sources et destinations de fichiers (S3, SFTP, SMB, dossier local, etc.)
          </p>
        </div>
        <Tooltip content="Ajouter un nouveau stockage distant">
          <button
            onClick={() => navigate('/remotes/new')}
            className="flex items-center gap-2 px-4 py-2.5 bg-brand-600 text-white rounded-lg hover:bg-brand-700 text-sm font-medium transition-colors shadow-sm"
          >
            <Plus size={16} /> Ajouter
          </button>
        </Tooltip>
      </div>

      {error && <div className="mb-4"><ErrorBanner message={(error as Error).message} /></div>}

      {/* Table */}
      <div className="bg-white rounded-xl border border-surface-200 shadow-sm overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-surface-50 border-b border-surface-200">
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Nom</th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Type</th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                <Tooltip content="Nombre de tâches utilisant ce stockage" position="bottom">
                  <span className="cursor-help border-b border-dashed border-surface-400">Utilisé par</span>
                </Tooltip>
              </th>
              <th className="text-right px-5 py-3"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-surface-100">
            {remotes?.map(remote => (
              <tr key={remote.id} className="hover:bg-surface-50/50 transition-colors">
                <td className="px-5 py-3.5 font-medium text-surface-900">{remote.name}</td>
                <td className="px-5 py-3.5">
                  <span className="inline-flex px-2 py-0.5 rounded bg-surface-100 text-surface-600 font-mono text-xs">
                    {remote.remote_type}
                  </span>
                </td>
                <td className="px-5 py-3.5 text-surface-500 text-xs">
                  {remote.task_count
                    ? <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-brand-50 text-brand-700 font-medium">{remote.task_count} tâche{remote.task_count > 1 ? 's' : ''}</span>
                    : <span className="text-surface-300">Aucune</span>}
                </td>
                <td className="px-5 py-3.5">
                  <div className="flex items-center justify-end gap-1">
                    {testResult[remote.id] && (
                      <span className={`text-xs mr-2 ${testResult[remote.id].success ? 'text-emerald-600' : 'text-red-600'}`}>
                        {testResult[remote.id].message}
                      </span>
                    )}
                    <Tooltip content="Tester la connexion au stockage">
                      <button
                        onClick={() => handleTest(remote.id)}
                        className="p-2 text-surface-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-colors"
                      >
                        <Wifi size={15} />
                      </button>
                    </Tooltip>
                    <Tooltip content="Modifier la configuration">
                      <button
                        onClick={() => navigate(`/remotes/${remote.id}/edit`)}
                        className="p-2 text-surface-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-colors"
                      >
                        <Pencil size={15} />
                      </button>
                    </Tooltip>
                    <Tooltip content={remote.task_count ? 'Suppression impossible : utilisé par des tâches' : 'Supprimer ce stockage'}>
                      <button
                        onClick={() => setConfirmId(remote.id)}
                        disabled={!!remote.task_count}
                        className={`p-2 rounded-lg transition-colors ${remote.task_count ? 'text-surface-200 cursor-not-allowed' : 'text-surface-400 hover:text-red-600 hover:bg-red-50'}`}
                      >
                        <Trash2 size={15} />
                      </button>
                    </Tooltip>
                  </div>
                </td>
              </tr>
            ))}
            {!remotes?.length && (
              <tr>
                <td colSpan={4} className="px-5 py-12 text-center text-surface-400">
                  <p className="text-base font-medium mb-1">Aucun stockage configuré</p>
                  <p className="text-xs">Cliquez sur "Ajouter" pour configurer votre premier stockage distant.</p>
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <ConfirmDialog
        open={!!confirmId}
        title="Supprimer ce stockage ?"
        message="Cette action est irréversible. Toutes les données de configuration seront perdues."
        onConfirm={() => { if (confirmId) deleteRemote.mutate(confirmId); setConfirmId(null); }}
        onCancel={() => setConfirmId(null)}
      />
    </div>
  );
}
