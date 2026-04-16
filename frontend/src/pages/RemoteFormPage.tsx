import { useEffect, useRef } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useForm, Resolver } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { ArrowLeft, Info } from 'lucide-react';
import { useCreateRemote, useUpdateRemote, useRemote } from '../hooks/useRemotes';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { StandardRemoteFields } from '../components/remotes/StandardRemoteFields';
import { GenericConfigFields } from '../components/remotes/GenericConfigFields';
import {
  STANDARD_REMOTES,
  ADVANCED_REMOTE_TYPES,
  getStandardRemote,
  isStandardType,
  buildDefaultConfig,
  buildZodSchema,
} from '../config/remoteFieldSchemas';

type ConfigEntry = { key: string; value: string };
type FormValues = {
  name: string;
  remote_type: string;
  config: Record<string, string>;
  configEntries: ConfigEntry[];
};

export function RemoteFormPage() {
  const { id } = useParams<{ id: string }>();
  const isEdit = !!id;
  const navigate = useNavigate();
  const { data: existing } = useRemote(id || '');
  const createRemote = useCreateRemote();
  const updateRemote = useUpdateRemote();

  const typeRef = useRef('s3');
  const initialLoadDone = useRef(false);
  const originalConfigRef = useRef<Record<string, string>>({});

  const {
    register,
    handleSubmit,
    control,
    reset,
    watch,
    setValue,
    formState: { errors, isSubmitting },
  } = useForm<FormValues>({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    resolver: (async (values: any, context: any, options: any) => {
      const schema = buildZodSchema(typeRef.current);
      const resolve = zodResolver(schema);
      return resolve(values, context, options);
    }) as Resolver<FormValues>,
    defaultValues: {
      name: '',
      remote_type: 's3',
      config: buildDefaultConfig('s3'),
      configEntries: [{ key: '', value: '' }],
    },
  });

  const selectedType = watch('remote_type');
  typeRef.current = selectedType;
  const standardSchema = getStandardRemote(selectedType);

  useEffect(() => {
    if (!initialLoadDone.current) return;
    if (isStandardType(selectedType)) {
      setValue('config', buildDefaultConfig(selectedType));
      setValue('configEntries', []);
    } else {
      setValue('config', {});
      setValue('configEntries', [{ key: '', value: '' }]);
    }
  }, [selectedType, setValue]);

  useEffect(() => {
    if (!existing) return;
    originalConfigRef.current = existing.config;
    if (isStandardType(existing.remote_type)) {
      const defaults = buildDefaultConfig(existing.remote_type);
      reset({
        name: existing.name,
        remote_type: existing.remote_type,
        config: { ...defaults, ...existing.config },
        configEntries: [],
      });
    } else {
      reset({
        name: existing.name,
        remote_type: existing.remote_type,
        config: {},
        configEntries: Object.entries(existing.config).map(([key, value]) => ({ key, value })),
      });
    }
    requestAnimationFrame(() => { initialLoadDone.current = true; });
  }, [existing, reset]);

  useEffect(() => {
    if (!isEdit) initialLoadDone.current = true;
  }, [isEdit]);

  const onSubmit = async (values: FormValues) => {
    let config: Record<string, string>;
    if (isStandardType(values.remote_type)) {
      config = {};
      for (const [k, v] of Object.entries(values.config)) {
        if (v !== undefined && v !== '') config[k] = v;
      }
      if (isEdit) {
        const schemaKeys = new Set(getStandardRemote(values.remote_type)?.fields.map((f) => f.key) ?? []);
        for (const [k, v] of Object.entries(originalConfigRef.current)) {
          if (!schemaKeys.has(k) && !(k in config)) config[k] = v;
        }
      }
    } else {
      config = {};
      values.configEntries.filter((e) => e.key).forEach((e) => { config[e.key] = e.value; });
    }

    const payload = { name: values.name, remote_type: values.remote_type, config };
    try {
      if (isEdit && id) {
        await updateRemote.mutateAsync({ id, payload });
      } else {
        await createRemote.mutateAsync(payload);
      }
      navigate('/remotes');
    } catch { /* handled */ }
  };

  const mutationError = createRemote.error || updateRemote.error;

  return (
    <div className="p-8 max-w-2xl animate-fade-in">
      {/* Back link */}
      <button onClick={() => navigate('/remotes')} className="flex items-center gap-1.5 text-sm text-surface-500 hover:text-brand-600 mb-4 transition-colors">
        <ArrowLeft size={14} /> Retour aux stockages
      </button>

      <h1 className="text-2xl font-bold text-surface-900 mb-1">
        {isEdit ? 'Modifier le stockage' : 'Nouveau stockage distant'}
      </h1>
      <p className="text-sm text-surface-500 mb-6">
        {isEdit
          ? 'Modifiez les paramètres de connexion de ce stockage.'
          : 'Configurez une nouvelle source ou destination pour vos tâches de réplication.'}
      </p>

      {mutationError && <div className="mb-4"><ErrorBanner message={(mutationError as Error).message} /></div>}

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
        {/* Name */}
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">
            Nom du stockage
            <span className="text-red-400 ml-0.5">*</span>
          </label>
          <input
            {...register('name')}
            className="w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow"
            placeholder="ex: backup-s3-prod"
          />
          <p className="text-xs text-surface-400 mt-1">Identifiant unique pour ce stockage. Utilisé dans les tâches de réplication.</p>
          {errors.name && <p className="text-red-500 text-xs mt-1">{errors.name.message}</p>}
        </div>

        {/* Type selector */}
        <div>
          <label className="block text-sm font-medium text-surface-700 mb-1">
            Type de stockage
            <span className="text-red-400 ml-0.5">*</span>
          </label>
          <select
            {...register('remote_type')}
            disabled={isEdit}
            className={`w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 transition-shadow ${isEdit ? 'bg-surface-100 cursor-not-allowed text-surface-500' : ''}`}
          >
            <optgroup label="Types standard (formulaire guidé)">
              {STANDARD_REMOTES.map((t) => <option key={t.value} value={t.value}>{t.label}</option>)}
            </optgroup>
            <optgroup label="Types avancés (configuration manuelle)">
              {ADVANCED_REMOTE_TYPES.map((t) => <option key={t.value} value={t.value}>{t.label}</option>)}
            </optgroup>
          </select>
          {isEdit && (
            <p className="flex items-center gap-1 text-xs text-amber-600 mt-1">
              <Info size={12} /> Le type ne peut pas être modifié après la création.
            </p>
          )}
          {!isEdit && (
            <p className="text-xs text-surface-400 mt-1">
              Les types standard affichent un formulaire guidé. Les types avancés utilisent des paires clé/valeur manuelles.
            </p>
          )}
        </div>

        {/* Adaptive config section */}
        <div className="pt-1">
          {standardSchema ? (
            <StandardRemoteFields fields={standardSchema.fields} register={register} errors={errors} />
          ) : (
            <GenericConfigFields control={control as never} register={register} />
          )}
        </div>

        {/* Actions */}
        <div className="flex gap-3 pt-4 border-t border-surface-200">
          <button
            type="submit"
            disabled={isSubmitting}
            className="px-5 py-2.5 bg-brand-600 text-white rounded-lg text-sm font-medium hover:bg-brand-700 disabled:opacity-50 transition-colors shadow-sm"
          >
            {isEdit ? 'Enregistrer les modifications' : 'Créer le stockage'}
          </button>
          <button
            type="button"
            onClick={() => navigate('/remotes')}
            className="px-5 py-2.5 border border-surface-300 text-surface-600 rounded-lg text-sm font-medium hover:bg-surface-50 transition-colors"
          >
            Annuler
          </button>
        </div>
      </form>
    </div>
  );
}
