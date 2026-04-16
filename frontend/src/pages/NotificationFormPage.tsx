import { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { ArrowLeft, Info } from 'lucide-react';
import { useCreateChannel, useUpdateChannel, useNotifications } from '../hooks/useNotifications';
import { ErrorBanner } from '../components/ui/ErrorBanner';

type FormValues = { name: string; apprise_url: string; enabled: boolean };

export function NotificationFormPage() {
  const { id } = useParams<{ id: string }>();
  const isEdit = !!id;
  const navigate = useNavigate();
  const { data: channels } = useNotifications();
  const existing = channels?.find(c => c.id === id);
  const createChannel = useCreateChannel();
  const updateChannel = useUpdateChannel();

  const { register, handleSubmit, reset, formState: { errors, isSubmitting } } = useForm<FormValues>({
    defaultValues: { name: '', apprise_url: '', enabled: true },
  });

  useEffect(() => {
    if (existing) reset({ name: existing.name, apprise_url: existing.apprise_url, enabled: existing.enabled });
  }, [existing, reset]);

  const onSubmit = async (values: FormValues) => {
    try {
      if (isEdit && id) {
        await updateChannel.mutateAsync({ id, payload: values });
      } else {
        await createChannel.mutateAsync(values);
      }
      navigate('/notifications');
    } catch { /* handled */ }
  };

  const mutationError = createChannel.error || updateChannel.error;
  const inputCls = "w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow";

  return (
    <div className="p-8 max-w-lg animate-fade-in">
      <button onClick={() => navigate('/notifications')} className="flex items-center gap-1.5 text-sm text-surface-500 hover:text-brand-600 mb-4 transition-colors">
        <ArrowLeft size={14} /> Retour aux notifications
      </button>

      <h1 className="text-2xl font-bold text-surface-900 mb-1">
        {isEdit ? 'Modifier le canal' : 'Nouveau canal de notification'}
      </h1>
      <p className="text-sm text-surface-500 mb-6">
        {isEdit
          ? 'Modifiez les paramètres de ce canal de notification.'
          : 'Configurez un canal pour recevoir des alertes par email, Slack, Mattermost, etc.'}
      </p>

      {mutationError && <div className="mb-4"><ErrorBanner message={(mutationError as Error).message} /></div>}

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">Nom du canal <span className="text-red-400">*</span></label>
          <input {...register('name', { required: 'Requis' })} className={inputCls} placeholder="ex: Alertes Mattermost équipe infra" />
          <p className="text-xs text-surface-400 mt-1">Un nom clair pour identifier ce canal dans la liste.</p>
          {errors.name && <p className="text-red-500 text-xs mt-1">{errors.name.message}</p>}
        </div>
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">URL Apprise <span className="text-red-400">*</span></label>
          <input {...register('apprise_url', { required: 'Requis' })} className={`${inputCls} font-mono`} placeholder="mmost://mattermost.example.com/webhook-token" />
          <div className="flex items-start gap-1.5 mt-1.5">
            <Info size={12} className="shrink-0 mt-0.5 text-surface-400" />
            <p className="text-xs text-surface-400">
              Format Apprise. Exemples :<br />
              <code className="bg-surface-100 px-1 rounded">mailto://user:pass@smtp.example.com</code> (email)<br />
              <code className="bg-surface-100 px-1 rounded">slack://tokenA/tokenB/tokenC</code> (Slack)<br />
              <code className="bg-surface-100 px-1 rounded">mmost://host/webhook</code> (Mattermost)
            </p>
          </div>
          {errors.apprise_url && <p className="text-red-500 text-xs mt-1">{errors.apprise_url.message}</p>}
        </div>
        <div className="flex items-center gap-3 py-1">
          <input type="checkbox" {...register('enabled')} id="enabled" className="rounded border-surface-300 text-brand-600 focus:ring-brand-500" />
          <label htmlFor="enabled" className="text-sm text-surface-700 cursor-pointer">
            Canal activé
            <span className="block text-xs text-surface-400">Si désactivé, aucune notification ne sera envoyée via ce canal.</span>
          </label>
        </div>
        <div className="flex gap-3 pt-4 border-t border-surface-200">
          <button type="submit" disabled={isSubmitting} className="px-5 py-2.5 bg-brand-600 text-white rounded-lg text-sm font-medium hover:bg-brand-700 disabled:opacity-50 transition-colors shadow-sm">
            {isEdit ? 'Enregistrer' : 'Créer le canal'}
          </button>
          <button type="button" onClick={() => navigate('/notifications')} className="px-5 py-2.5 border border-surface-300 text-surface-600 rounded-lg text-sm font-medium hover:bg-surface-50 transition-colors">
            Annuler
          </button>
        </div>
      </form>
    </div>
  );
}
