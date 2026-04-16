import { Control, UseFormRegister, useFieldArray } from 'react-hook-form';
import { Plus, Trash2, Info } from 'lucide-react';

interface Props {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  control: Control<any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  register: UseFormRegister<any>;
}

export function GenericConfigFields({ control, register }: Props) {
  const { fields, append, remove } = useFieldArray({ control, name: 'configEntries' });

  return (
    <fieldset className="border border-surface-200 rounded-lg p-4 space-y-3">
      <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1">
        Configuration avancée
      </legend>
      <p className="flex items-start gap-1.5 text-xs text-surface-400">
        <Info size={12} className="shrink-0 mt-0.5" />
        Entrez les paires clé/valeur correspondant aux options de configuration rclone pour ce type de stockage.
      </p>
      <div className="space-y-2">
        {fields.map((field, index) => (
          <div key={field.id} className="flex gap-2">
            <input
              {...register(`configEntries.${index}.key`)}
              placeholder="clé (ex: endpoint)"
              className="flex-1 border border-surface-300 rounded-lg px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-brand-500 transition-shadow"
            />
            <input
              {...register(`configEntries.${index}.value`)}
              placeholder="valeur"
              className="flex-1 border border-surface-300 rounded-lg px-3 py-2 text-sm focus:ring-2 focus:ring-brand-500 transition-shadow"
            />
            <button type="button" onClick={() => remove(index)} className="p-2 text-surface-400 hover:text-red-500 rounded-lg hover:bg-red-50 transition-colors">
              <Trash2 size={14} />
            </button>
          </div>
        ))}
      </div>
      <button
        type="button"
        onClick={() => append({ key: '', value: '' })}
        className="flex items-center gap-1.5 text-xs text-brand-600 hover:text-brand-700 font-medium transition-colors"
      >
        <Plus size={13} /> Ajouter une entrée
      </button>
    </fieldset>
  );
}
