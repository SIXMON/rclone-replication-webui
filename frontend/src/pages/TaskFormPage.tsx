import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { ArrowLeft, Info } from 'lucide-react';
import { useCreateTask } from '../hooks/useTasks';
import { useRemotes } from '../hooks/useRemotes';
import { useNotifications } from '../hooks/useNotifications';
import { ErrorBanner } from '../components/ui/ErrorBanner';

type FormValues = {
  name: string;
  source_remote_id: string;
  source_path: string;
  dest_remote_id: string;
  dest_path: string;
  cron_expression: string;
  notification_channel_id: string;
  notify_on_error: boolean;
  notify_on_success: boolean;
  notify_on_skipped: boolean;
  rclone_flags: string;
  max_retries: number;
  retry_delay_seconds: number;
};

export function TaskFormPage() {
  const navigate = useNavigate();
  const createTask = useCreateTask();
  const { data: remotes } = useRemotes();
  const { data: channels } = useNotifications();

  const { register, handleSubmit, watch, formState: { errors, isSubmitting } } = useForm<FormValues>({
    defaultValues: { name: '', source_path: '', dest_path: '', cron_expression: '', rclone_flags: '', notification_channel_id: '', notify_on_error: true, notify_on_success: false, notify_on_skipped: false, max_retries: 3, retry_delay_seconds: 15 },
  });

  const selectedChannel = watch('notification_channel_id');
  const watchRetries = watch('max_retries');
  const watchDelay = watch('retry_delay_seconds');

  const onSubmit = async (values: FormValues) => {
    const flags = values.rclone_flags.trim()
      ? values.rclone_flags.trim().split(/\s+/)
      : [];
    const notify_on: string[] = [];
    if (values.notify_on_error) notify_on.push('error');
    if (values.notify_on_success) notify_on.push('success');
    if (values.notify_on_skipped) notify_on.push('skipped');
    try {
      await createTask.mutateAsync({
        name: values.name,
        source_remote_id: values.source_remote_id,
        source_path: values.source_path,
        dest_remote_id: values.dest_remote_id,
        dest_path: values.dest_path,
        cron_expression: values.cron_expression || null,
        enabled: true,
        rclone_flags: flags,
        notification_channel_id: values.notification_channel_id || null,
        notify_on,
        max_retries: Number(values.max_retries),
        retry_delay_seconds: Number(values.retry_delay_seconds),
      });
      navigate('/tasks');
    } catch { /* handled */ }
  };

  const inputCls = "w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow";

  return (
    <div className="p-8 max-w-2xl animate-fade-in">
      <button onClick={() => navigate('/tasks')} className="flex items-center gap-1.5 text-sm text-surface-500 hover:text-brand-600 mb-4 transition-colors">
        <ArrowLeft size={14} /> Retour aux tâches
      </button>

      <h1 className="text-2xl font-bold text-surface-900 mb-1">Nouvelle tâche de réplication</h1>
      <p className="text-sm text-surface-500 mb-6">
        Configurez la source, la destination et la fréquence de synchronisation.
      </p>

      {createTask.error && <div className="mb-4"><ErrorBanner message={(createTask.error as Error).message} /></div>}

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">Nom de la tâche <span className="text-red-400">*</span></label>
          <input {...register('name', { required: 'Requis' })} className={inputCls} placeholder="ex: Backup prod quotidien" />
          <p className="text-xs text-surface-400 mt-1">Un nom clair pour identifier cette tâche dans la liste.</p>
          {errors.name && <p className="text-red-500 text-xs mt-1">{errors.name.message}</p>}
        </div>

        {/* Source */}
        <fieldset className="border border-surface-200 rounded-lg p-4 space-y-3">
          <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1">Source (d'où copier)</legend>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">Stockage source <span className="text-red-400">*</span></label>
              <select {...register('source_remote_id', { required: 'Requis' })} className={inputCls}>
                <option value="">-- choisir --</option>
                {remotes?.map(r => <option key={r.id} value={r.id}>{r.name} ({r.remote_type})</option>)}
              </select>
              {errors.source_remote_id && <p className="text-red-500 text-xs mt-1">{errors.source_remote_id.message}</p>}
            </div>
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">Chemin source</label>
              <input {...register('source_path')} className={`${inputCls} font-mono`} placeholder="bucket/dossier" />
              <p className="text-xs text-surface-400 mt-1">Chemin relatif dans le stockage.</p>
            </div>
          </div>
        </fieldset>

        {/* Destination */}
        <fieldset className="border border-surface-200 rounded-lg p-4 space-y-3">
          <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1">Destination (où copier)</legend>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">Stockage destination <span className="text-red-400">*</span></label>
              <select {...register('dest_remote_id', { required: 'Requis' })} className={inputCls}>
                <option value="">-- choisir --</option>
                {remotes?.map(r => <option key={r.id} value={r.id}>{r.name} ({r.remote_type})</option>)}
              </select>
              {errors.dest_remote_id && <p className="text-red-500 text-xs mt-1">{errors.dest_remote_id.message}</p>}
            </div>
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">Chemin destination</label>
              <input {...register('dest_path')} className={`${inputCls} font-mono`} placeholder="archive/2026" />
              <p className="text-xs text-surface-400 mt-1">Chemin relatif dans le stockage.</p>
            </div>
          </div>
        </fieldset>

        {/* Scheduling */}
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">Planification automatique</label>
          <input {...register('cron_expression')} className={`${inputCls} font-mono`} placeholder="0 2 * * *" />
          <p className="flex items-start gap-1 text-xs text-surface-400 mt-1">
            <Info size={12} className="shrink-0 mt-0.5" />
            Expression cron (ex: <code className="bg-surface-100 px-1 rounded">0 2 * * *</code> = chaque jour à 2h). Laisser vide pour déclencher uniquement manuellement.
          </p>
        </div>

        {/* Rclone flags */}
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">Options rclone supplémentaires</label>
          <input {...register('rclone_flags')} className={`${inputCls} font-mono`} placeholder="--bwlimit 10M --transfers 4" />
          <p className="text-xs text-surface-400 mt-1">Flags additionnels passés à rclone (optionnel). Séparés par des espaces.</p>
        </div>

        {/* Retry */}
        <fieldset className="border border-surface-200 rounded-lg p-4 space-y-3">
          <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1">Retry en cas d'échec</legend>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">Nombre de tentatives</label>
              <input {...register('max_retries', { valueAsNumber: true, min: 0, max: 20 })} type="number" min={0} max={20} className={inputCls} />
              <p className="text-xs text-surface-400 mt-1">0 = pas de retry. Max 20.</p>
            </div>
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">Délai de base (secondes)</label>
              <input {...register('retry_delay_seconds', { valueAsNumber: true, min: 1 })} type="number" min={1} className={inputCls} />
              <p className="text-xs text-surface-400 mt-1">Multiplié par le n° de tentative.</p>
            </div>
          </div>
          {Number(watchRetries) > 0 && (
            <p className="text-xs text-surface-500 bg-surface-50 rounded-lg px-3 py-2">
              Si la tâche échoue, elle sera relancée jusqu'à <strong>{watchRetries} fois</strong> :{' '}
              {Array.from({ length: Math.min(Number(watchRetries), 3) }, (_, i) => {
                const delay = Number(watchDelay) * (i + 1);
                return `tentative ${i + 2} après ${delay}s`;
              }).join(', ')}
              {Number(watchRetries) > 3 && ', ...'}
            </p>
          )}
        </fieldset>

        {/* Notifications */}
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">Notifications</label>
          <select {...register('notification_channel_id')} className={inputCls}>
            <option value="">-- aucune notification --</option>
            {channels?.map(c => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
          <div className={`mt-2.5 flex items-center gap-4 transition-opacity ${!selectedChannel ? 'opacity-30 pointer-events-none' : ''}`}>
            <span className="text-xs text-surface-500">Notifier en cas de :</span>
            <label className="flex items-center gap-1.5 text-sm text-surface-700 cursor-pointer">
              <input type="checkbox" {...register('notify_on_error')} className="rounded border-surface-300 text-brand-600 focus:ring-brand-500" />
              Erreur
            </label>
            <label className="flex items-center gap-1.5 text-sm text-surface-700 cursor-pointer">
              <input type="checkbox" {...register('notify_on_success')} className="rounded border-surface-300 text-brand-600 focus:ring-brand-500" />
              Succès
            </label>
            <label className="flex items-center gap-1.5 text-sm text-surface-700 cursor-pointer">
              <input type="checkbox" {...register('notify_on_skipped')} className="rounded border-surface-300 text-amber-600 focus:ring-amber-500" />
              Ignorée
            </label>
          </div>
          <p className="text-xs text-surface-400 mt-1">Choisissez un canal pour recevoir des alertes. "Ignorée" = la tâche n'a pas pu démarrer car l'exécution précédente était encore en cours.</p>
        </div>

        {/* Actions */}
        <div className="flex gap-3 pt-4 border-t border-surface-200">
          <button type="submit" disabled={isSubmitting} className="px-5 py-2.5 bg-brand-600 text-white rounded-lg text-sm font-medium hover:bg-brand-700 disabled:opacity-50 transition-colors shadow-sm">
            Créer la tâche
          </button>
          <button type="button" onClick={() => navigate('/tasks')} className="px-5 py-2.5 border border-surface-300 text-surface-600 rounded-lg text-sm font-medium hover:bg-surface-50 transition-colors">
            Annuler
          </button>
        </div>
      </form>
    </div>
  );
}
