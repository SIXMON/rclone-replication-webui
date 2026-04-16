import { AlertTriangle } from 'lucide-react';

type Variant = 'danger' | 'warning';

const variants: Record<Variant, { iconBg: string; iconColor: string; btnBg: string; btnHover: string }> = {
  danger:  { iconBg: 'bg-red-100', iconColor: 'text-red-600', btnBg: 'bg-red-600', btnHover: 'hover:bg-red-700' },
  warning: { iconBg: 'bg-orange-100', iconColor: 'text-orange-600', btnBg: 'bg-orange-600', btnHover: 'hover:bg-orange-700' },
};

interface Props {
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  variant?: Variant;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({ open, title, message, confirmLabel = 'Supprimer', variant = 'danger', onConfirm, onCancel }: Props) {
  if (!open) return null;
  const v = variants[variant];
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-surface-950/50 backdrop-blur-sm animate-fade-in">
      <div className="bg-white rounded-xl shadow-2xl p-6 max-w-sm w-full mx-4 animate-slide-up">
        <div className="flex items-start gap-3 mb-4">
          <div className={`w-10 h-10 rounded-full ${v.iconBg} flex items-center justify-center shrink-0`}>
            <AlertTriangle size={20} className={v.iconColor} />
          </div>
          <div>
            <h2 className="text-base font-semibold text-surface-900">{title}</h2>
            <p className="text-surface-500 text-sm mt-1">{message}</p>
          </div>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm font-medium text-surface-600 border border-surface-200 rounded-lg hover:bg-surface-50 transition-colors"
          >
            Annuler
          </button>
          <button
            onClick={onConfirm}
            className={`px-4 py-2 text-sm font-medium text-white rounded-lg transition-colors ${v.btnBg} ${v.btnHover}`}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
