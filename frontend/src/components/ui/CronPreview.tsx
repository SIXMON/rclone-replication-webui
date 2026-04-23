import { CronExpressionParser } from 'cron-parser';
import cronstrue from 'cronstrue/i18n';
import { Calendar, CheckCircle2, XCircle } from 'lucide-react';

interface Props {
  expression: string;
}

/**
 * Détecte le format d'une expression cron :
 * - 5 champs : format Unix standard (minute heure jour mois j.semaine)
 * - 6 champs : format étendu avec secondes en premier
 * - @macro   : raccourcis comme @hourly, @daily, @weekly...
 */
function detectFormat(expr: string): '5-fields' | '6-fields' | 'macro' | 'unknown' {
  const trimmed = expr.trim();
  if (trimmed.startsWith('@')) return 'macro';
  const count = trimmed.split(/\s+/).length;
  if (count === 5) return '5-fields';
  if (count === 6) return '6-fields';
  return 'unknown';
}

/**
 * Affiche un aperçu d'une expression cron :
 * - Validation (valide / invalide)
 * - Traduction en français ("Toutes les deux heures")
 * - 3 prochaines exécutions prévues
 *
 * Supporte les formats 5 champs (standard Unix) et 6 champs (avec secondes).
 */
export function CronPreview({ expression }: Props) {
  const trimmed = expression.trim();
  if (!trimmed) return null;

  const format = detectFormat(trimmed);

  // 1. Validation + traduction (cronstrue gère nativement 5, 6 et 7 champs)
  let humanReadable: string | null = null;
  let error: string | null = null;
  try {
    humanReadable = cronstrue.toString(trimmed, { locale: 'fr' });
  } catch (e) {
    error = e instanceof Error ? e.message : 'Expression invalide';
  }

  // 2. Calcul des prochaines exécutions (cron-parser accepte 5 et 6 champs)
  let nextRuns: Date[] = [];
  if (!error) {
    try {
      const it = CronExpressionParser.parse(trimmed, { tz: 'Europe/Paris' });
      for (let i = 0; i < 3; i++) {
        nextRuns.push(it.next().toDate());
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Expression invalide';
      nextRuns = [];
    }
  }

  if (error) {
    return (
      <div className="mt-2 flex items-start gap-2 text-xs bg-red-50 border border-red-200 rounded-lg px-3 py-2">
        <XCircle size={14} className="shrink-0 mt-0.5 text-red-500" />
        <div>
          <p className="font-medium text-red-700">Expression cron invalide</p>
          <p className="text-red-600 mt-0.5">{error}</p>
        </div>
      </div>
    );
  }

  const formatLabel =
    format === '5-fields' ? '5 champs' :
    format === '6-fields' ? '6 champs (avec secondes)' :
    format === 'macro' ? 'macro' : '';

  return (
    <div className="mt-2 bg-emerald-50 border border-emerald-200 rounded-lg px-3 py-2 space-y-1.5">
      <div className="flex items-start gap-2 text-xs">
        <CheckCircle2 size={14} className="shrink-0 mt-0.5 text-emerald-600" />
        <p className="text-emerald-800">
          <span className="font-medium">Valide</span>
          {formatLabel && (
            <span className="ml-1.5 px-1.5 py-0.5 rounded bg-emerald-100 text-emerald-700 text-[10px] font-mono">
              {formatLabel}
            </span>
          )}
          {' — '}{humanReadable}
        </p>
      </div>
      {nextRuns.length > 0 && (
        <div className="flex items-start gap-2 text-xs pt-1 border-t border-emerald-200/70">
          <Calendar size={14} className="shrink-0 mt-0.5 text-emerald-600" />
          <div className="min-w-0">
            <p className="font-medium text-emerald-800 mb-0.5">Prochaines exécutions :</p>
            <ul className="text-emerald-700 space-y-0.5">
              {nextRuns.map((d, i) => (
                <li key={i} className="font-mono">
                  {d.toLocaleString('fr', { dateStyle: 'medium', timeStyle: 'short' })}
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
