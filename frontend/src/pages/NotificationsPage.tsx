import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Plus, Pencil, Trash2, Send } from 'lucide-react';
import { useNotifications, useDeleteChannel, useTestChannel } from '../hooks/useNotifications';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { Tooltip } from '../components/ui/Tooltip';

export function NotificationsPage() {
  const { data: channels, isLoading, error } = useNotifications();
  const deleteChannel = useDeleteChannel();
  const testChannel = useTestChannel();
  const navigate = useNavigate();
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, string>>({});

  const handleTest = async (id: string) => {
    const result = await testChannel.mutateAsync(id);
    setTestResults(prev => ({ ...prev, [id]: result.success ? 'Envoyé' : `Erreur : ${result.message}` }));
  };

  if (isLoading) return <div className="p-8 text-surface-400">Chargement...</div>;

  return (
    <div className="p-8 max-w-5xl animate-fade-in">
      {/* Header */}
      <div className="flex items-end justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">Canaux de notification</h1>
          <p className="text-sm text-surface-500 mt-1">
            Configurez où recevoir les alertes (email, Slack, Mattermost, etc.) en cas d'erreur ou de succès.
          </p>
        </div>
        <Tooltip content="Ajouter un nouveau canal de notification">
          <button
            onClick={() => navigate('/notifications/new')}
            className="flex items-center gap-2 px-4 py-2.5 bg-brand-600 text-white rounded-lg hover:bg-brand-700 text-sm font-medium transition-colors shadow-sm"
          >
            <Plus size={16} /> Ajouter
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
                <Tooltip content="URL au format Apprise (protocole://...). Définit où envoyer les alertes." position="bottom">
                  <span className="cursor-help border-b border-dashed border-surface-400">URL Apprise</span>
                </Tooltip>
              </th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">Activé</th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                <Tooltip content="Nombre de tâches utilisant ce canal" position="bottom">
                  <span className="cursor-help border-b border-dashed border-surface-400">Utilisé par</span>
                </Tooltip>
              </th>
              <th className="text-right px-5 py-3"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-surface-100">
            {channels?.map(ch => (
              <tr key={ch.id} className="hover:bg-surface-50/50 transition-colors">
                <td className="px-5 py-3.5 font-medium text-surface-900">{ch.name}</td>
                <td className="px-5 py-3.5 text-surface-500 font-mono text-xs truncate max-w-xs">{ch.apprise_url}</td>
                <td className="px-5 py-3.5">
                  <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${ch.enabled ? 'bg-emerald-50 text-emerald-700' : 'bg-surface-100 text-surface-500'}`}>
                    {ch.enabled ? 'Actif' : 'Inactif'}
                  </span>
                </td>
                <td className="px-5 py-3.5 text-surface-500 text-xs">
                  {ch.task_count
                    ? <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-brand-50 text-brand-700 font-medium">{ch.task_count} tâche{ch.task_count > 1 ? 's' : ''}</span>
                    : <span className="text-surface-300">Aucune</span>}
                </td>
                <td className="px-5 py-3.5">
                  <div className="flex items-center justify-end gap-1">
                    {testResults[ch.id] && (
                      <span className={`text-xs mr-2 ${testResults[ch.id].startsWith('Envoyé') ? 'text-emerald-600' : 'text-red-600'}`}>
                        {testResults[ch.id]}
                      </span>
                    )}
                    <Tooltip content="Envoyer une notification de test">
                      <button onClick={() => handleTest(ch.id)} className="p-2 text-surface-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-colors">
                        <Send size={15} />
                      </button>
                    </Tooltip>
                    <Tooltip content="Modifier ce canal">
                      <button onClick={() => navigate(`/notifications/${ch.id}/edit`)} className="p-2 text-surface-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-colors">
                        <Pencil size={15} />
                      </button>
                    </Tooltip>
                    <Tooltip content={ch.task_count ? 'Suppression impossible : utilisé par des tâches' : 'Supprimer ce canal'}>
                      <button
                        onClick={() => setConfirmId(ch.id)}
                        disabled={!!ch.task_count}
                        className={`p-2 rounded-lg transition-colors ${ch.task_count ? 'text-surface-200 cursor-not-allowed' : 'text-surface-400 hover:text-red-600 hover:bg-red-50'}`}
                      >
                        <Trash2 size={15} />
                      </button>
                    </Tooltip>
                  </div>
                </td>
              </tr>
            ))}
            {!channels?.length && (
              <tr>
                <td colSpan={5} className="px-5 py-12 text-center text-surface-400">
                  <p className="text-base font-medium mb-1">Aucun canal configuré</p>
                  <p className="text-xs">Cliquez sur "Ajouter" pour configurer votre premier canal de notification.</p>
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <ConfirmDialog
        open={!!confirmId}
        title="Supprimer ce canal ?"
        message="Cette action est irréversible. Les tâches qui l'utilisent ne recevront plus de notifications."
        onConfirm={() => { if (confirmId) deleteChannel.mutate(confirmId); setConfirmId(null); }}
        onCancel={() => setConfirmId(null)}
      />
    </div>
  );
}
