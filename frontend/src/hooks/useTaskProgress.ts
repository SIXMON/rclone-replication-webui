import { useEffect, useRef, useState } from 'react';

interface ProgressState {
  lines: string[];
  done: boolean;
  status: string | null;
}

/**
 * S'abonne au flux SSE de progression d'une tâche.
 *
 * @param taskId  - UUID de la tâche à suivre
 * @param running - true si la tâche est en cours (d'après le cache API)
 * @param forceConnect - true pour se connecter immédiatement sans attendre `running`
 *                       (utilisé après un trigger/restore pour éviter la race condition)
 */
export function useTaskProgress(taskId: string | null, running: boolean, forceConnect = false) {
  const [progress, setProgress] = useState<ProgressState>({ lines: [], done: false, status: null });
  const esRef = useRef<EventSource | null>(null);

  const shouldConnect = running || forceConnect;

  useEffect(() => {
    if (!taskId || !shouldConnect) return;

    setProgress({ lines: [], done: false, status: null });
    const es = new EventSource(`/api/tasks/${taskId}/progress`);
    esRef.current = es;

    es.addEventListener('log', (e) => {
      try {
        const data = JSON.parse(e.data);
        setProgress(prev => ({ ...prev, lines: [...prev.lines, data.line] }));
      } catch { /* ignore */ }
    });

    es.addEventListener('done', (e) => {
      try {
        const data = JSON.parse(e.data);
        setProgress(prev => ({ ...prev, done: true, status: data.status }));
      } catch { /* ignore */ }
      es.close();
    });

    es.addEventListener('idle', () => {
      // La tâche n'est pas encore en cours — réessayer dans 500ms
      es.close();
      const retry = setTimeout(() => {
        setProgress(prev => {
          // Si entretemps on a reçu des données, ne pas réinitialiser
          if (prev.lines.length > 0 || prev.done) return prev;
          return prev;
        });
        // Relancer la connexion en modifiant la ref pour forcer un re-render
        esRef.current = null;
      }, 500);
      return () => clearTimeout(retry);
    });

    es.onerror = () => es.close();

    return () => {
      es.close();
      esRef.current = null;
    };
  }, [taskId, shouldConnect]);

  return progress;
}
