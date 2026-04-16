import { useEffect, useRef } from 'react';
import { Terminal, CheckCircle2, XCircle } from 'lucide-react';

interface Props {
  lines: string[];
  done: boolean;
  status: string | null;
}

export function LiveProgressPanel({ lines, done, status }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [lines]);

  return (
    <div className="bg-surface-950 rounded-xl overflow-hidden shadow-sm animate-slide-up">
      <div className="flex items-center justify-between px-4 py-2.5 bg-surface-900 border-b border-white/10">
        <span className="flex items-center gap-2 text-xs text-surface-400 font-mono">
          <Terminal size={13} /> Progression en temps réel
        </span>
        {done && status && (
          <span className={`flex items-center gap-1 text-xs font-medium ${status === 'success' ? 'text-emerald-400' : 'text-red-400'}`}>
            {status === 'success' ? <><CheckCircle2 size={13} /> Terminé</> : <><XCircle size={13} /> Échec</>}
          </span>
        )}
      </div>
      <div className="h-64 overflow-y-auto p-4 font-mono text-xs text-surface-300 space-y-0.5">
        {lines.length === 0 && !done && (
          <p className="text-surface-600 animate-pulse">En attente des logs...</p>
        )}
        {lines.map((line, i) => (
          <div key={i} className="leading-relaxed whitespace-pre-wrap break-all">{line}</div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
