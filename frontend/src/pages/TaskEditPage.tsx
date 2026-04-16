import { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { ArrowLeft, Info } from 'lucide-react';
import { useTask, usePatchTask } from '../hooks/useTasks';
import { useNotifications } from '../hooks/useNotifications';
import { ErrorBanner } from '../components/ui/ErrorBanner';

type FormValues = {
  name: string;
  cron_expression: string;
  enabled: boolean;
  rclone_flags: string;
  notification_channel_id: string;
  notify_on_error: boolean;
  notify_on_success: boolean;
  notify_on_skipped: boolean;
  max_retries: number;
  retry_delay_seconds: number;
};

export function TaskEditPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: task } = useTask(id || '');
  const { data: channels } = useNotifications();
  const patchTask = usePatchTask();

  const { register, handleSubmit, reset, watch, formState: { errors, isSubmitting } } = useForm<FormValues>({
    defaultValues: { name: '', cron_expression: '', enabled: true, rclone_flags: '', notification_channel_id: '', notify_on_error: true, notify_on_success: false, notify_on_skipped: false, max_retries: 3, retry_delay_seconds: 15 },
  });

  const selectedChannel = watch('notification_channel_id');
  const watchRetries = watch('max_retries');
  const watchDelay = watch('retry_delay_seconds');

  useEffect(() => {
    if (task) {
      reset({
        name: task.name,
        cron_expression: task.cron_expression || '',
        enabled: task.enabled,
        rclone_flags: task.rclone_flags.join(' '),
        notification_channel_id: task.notification_channel_id || '',
        notify_on_error: task.notify_on?.includes('error') ?? true,
        notify_on_success: task.notify_on?.includes('success') ?? false,
        notify_on_skipped: task.notify_on?.includes('skipped') ?? false,
        max_retries: task.max_retries ?? 3,
        retry_delay_seconds: task.retry_delay_seconds ?? 15,
      });
    }
  }, [task, reset]);

  const onSubmit = async (values: FormValues) => {
    if (!id) return;
    const flags = values.rclone_flags.trim() ? values.rclone_flags.trim().split(/\s+/) : [];
    const notify_on: string[] = [];
    if (values.notify_on_error) notify_on.push('error');
    if (values.notify_on_success) notify_on.push('success');
    if (values.notify_on_skipped) notify_on.push('skipped');
    try {
      await patchTask.mutateAsync({
        id,
        payload: {
          name: values.name,
          cron_expression: values.cron_expression || null,
          enabled: values.enabled,
          rclone_flags: flags,
          notification_channel_id: values.notification_channel_id || null,
          notify_on,
          max_retries: Number(values.max_retries),
          retry_delay_seconds: Number(values.retry_delay_seconds),
        },
      });
      navigate(`/tasks/${id}`);
    } catch { /* handled */ }
  };

  const inputCls = "w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow";

  return (
    <div className="p-8 max-w-lg animate-fade-in">
      <button onClick={() => navigate(`/tasks/${id}`)} className="flex items-center gap-1.5 text-sm text-surface-500 hover:text-brand-600 mb-4 transition-colors">
        <ArrowLeft size={14} /> Retour au détail
      </button>

      <h1 className="text-2xl font-bold text-surface-900 mb-1">Modifier la tâche</h1>
      <p className="text-sm text-surface-500 mb-4">
        Ajustez le nom, la planification, les options ou les notifications.
      </p>

      <div className="flex items-start gap-2 text-sm text-amber-700 bg-amber-50 border border-amber-200 rounded-lg px-4 py-3 mb-6">
        <Info size={16} className="shrink-0 mt-0.5 text-amber-500" />
        <span>Les chemins source et destination ne peuvent pas être modifiés après création.</span>
      </div>

      {patchTask.error && <div className="mb-4"><ErrorBanner message={(patchTask.error as Error).message} /></div>}

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">Nom <span className="text-red-400">*</span></label>
          <input {...register('name', { required: 'Requis' })} className={inputCls} />
          {errors.name && <p className="text-red-500 text-xs mt-1">{errors.name.message}</p>}
        </div>
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">Planification automatique</label>
          <input {...register('cron_expression')} className={`${inputCls} font-mono`} placeholder="0 2 * * *" />
          <p className="text-xs text-surface-400 mt-1">Expression cron. Laisser vide = déclenchement manuel uniquement.</p>
        </div>
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">Options rclone supplémentaires</label>
          <input {...register('rclone_flags')} className={`${inputCls} font-mono`} placeholder="--bwlimit 10M" />
        </div>

        {/* Retry */}
        <fieldset className="border border-surface-200 rounded-lg p-4 space-y-3">
          <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1">Retry en cas d'échec</legend>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">Nombre de tentatives</label>
              <input {...register('max_retries', { valueAsNumber: true, min: 0, max: 20 })} type="number" min={0} max={20} className={inputCls} />
              <p className="text-xs text-surface-400 mt-1">0 = pas de retry.</p>
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
        </div>

        <div className="flex items-center gap-3 py-1">
          <input type="checkbox" {...register('enabled')} id="enabled" className="rounded border-surface-300 text-brand-600 focus:ring-brand-500" />
          <label htmlFor="enabled" className="text-sm text-surface-700 cursor-pointer">
            Tâche activée
            <span className="block text-xs text-surface-400">Si désactivée, la planification cron est ignorée.</span>
          </label>
        </div>

        <div className="flex gap-3 pt-4 border-t border-surface-200">
          <button type="submit" disabled={isSubmitting} className="px-5 py-2.5 bg-brand-600 text-white rounded-lg text-sm font-medium hover:bg-brand-700 disabled:opacity-50 transition-colors shadow-sm">
            Enregistrer
          </button>
          <button type="button" onClick={() => navigate(`/tasks/${id}`)} className="px-5 py-2.5 border border-surface-300 text-surface-600 rounded-lg text-sm font-medium hover:bg-surface-50 transition-colors">
            Annuler
          </button>
        </div>
      </form>
    </div>
  );
}
