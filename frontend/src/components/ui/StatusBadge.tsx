import { CheckCircle2, XCircle, Loader2, MinusCircle, HelpCircle, SkipForward } from 'lucide-react';

type Status = 'success' | 'failure' | 'running' | 'disabled' | 'skipped' | 'unknown';

const config: Record<Status, { bg: string; text: string; label: string; Icon: typeof CheckCircle2 }> = {
  success:  { bg: 'bg-emerald-50', text: 'text-emerald-700', label: 'Succès', Icon: CheckCircle2 },
  failure:  { bg: 'bg-red-50', text: 'text-red-700', label: 'Échec', Icon: XCircle },
  running:  { bg: 'bg-brand-50', text: 'text-brand-700', label: 'En cours', Icon: Loader2 },
  skipped:  { bg: 'bg-amber-50', text: 'text-amber-700', label: 'Ignorée', Icon: SkipForward },
  disabled: { bg: 'bg-surface-100', text: 'text-surface-500', label: 'Désactivé', Icon: MinusCircle },
  unknown:  { bg: 'bg-surface-100', text: 'text-surface-500', label: 'Inconnu', Icon: HelpCircle },
};

export function StatusBadge({ status }: { status: Status }) {
  const { bg, text, label, Icon } = config[status] ?? config.unknown;
  return (
    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${bg} ${text}`}>
      <Icon size={12} className={status === 'running' ? 'animate-spin' : ''} />
      {label}
    </span>
  );
}
