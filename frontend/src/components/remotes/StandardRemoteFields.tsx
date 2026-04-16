import { UseFormRegister, FieldErrors } from 'react-hook-form';
import type { RemoteFieldDef } from '../../config/remoteFieldSchemas';

interface Props {
  fields: RemoteFieldDef[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  register: UseFormRegister<any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  errors: FieldErrors<any>;
}

const inputCls = "w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow";

export function StandardRemoteFields({ fields, register, errors }: Props) {
  return (
    <fieldset className="border border-surface-200 rounded-lg p-4 space-y-4">
      <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1">
        Paramètres de connexion
      </legend>
      {fields.map((field) => {
        const fieldError = (errors.config as Record<string, { message?: string }> | undefined)?.[field.key];

        return (
          <div key={field.key}>
            <label className="block text-sm font-medium text-surface-700 mb-1">
              {field.label}
              {field.required && <span className="text-red-400 ml-0.5">*</span>}
            </label>

            {field.type === 'select' ? (
              <select {...register(`config.${field.key}`)} className={inputCls}>
                {field.options?.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            ) : (
              <input
                {...register(`config.${field.key}`)}
                type={field.type === 'password' ? 'password' : 'text'}
                inputMode={field.type === 'number' ? 'numeric' : undefined}
                placeholder={field.placeholder}
                className={inputCls}
              />
            )}

            {field.helpText && <p className="text-xs text-surface-400 mt-1">{field.helpText}</p>}
            {fieldError?.message && <p className="text-red-500 text-xs mt-1">{fieldError.message}</p>}
          </div>
        );
      })}
    </fieldset>
  );
}
