import { AlertTriangle } from 'lucide-react';

export function ErrorBanner({ message }: { message: string }) {
  return (
    <div className="flex items-start gap-3 bg-red-50 border border-red-200 text-red-800 rounded-lg px-4 py-3 text-sm animate-fade-in">
      <AlertTriangle size={16} className="shrink-0 mt-0.5 text-red-500" />
      <span>{message}</span>
    </div>
  );
}
