import { useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';

/**
 * S'abonne au flux SSE global `/api/events` et invalide
 * les caches react-query quand l'état des tâches change.
 *
 * Événements gérés :
 * - task_started  → rafraîchit la liste des tâches (statut running)
 * - task_finished → rafraîchit la liste, le détail et l'historique des runs
 */
export function useGlobalEvents() {
  const qc = useQueryClient();
  const esRef = useRef<EventSource | null>(null);

  useEffect(() => {
    let reconnectTimer: ReturnType<typeof setTimeout>;

    function connect() {
      const es = new EventSource('/api/events');
      esRef.current = es;

      es.addEventListener('task_started', (e) => {
        try {
          const { task_id } = JSON.parse(e.data);
          qc.invalidateQueries({ queryKey: ['tasks'] });
          qc.invalidateQueries({ queryKey: ['tasks', task_id] });
        } catch { /* ignore */ }
      });

      es.addEventListener('task_finished', (e) => {
        try {
          const { task_id } = JSON.parse(e.data);
          qc.invalidateQueries({ queryKey: ['tasks'] });
          qc.invalidateQueries({ queryKey: ['tasks', task_id] });
          qc.invalidateQueries({ queryKey: ['runs', task_id] });
        } catch { /* ignore */ }
      });

      es.onerror = () => {
        es.close();
        esRef.current = null;
        // Reconnexion automatique après 3s
        reconnectTimer = setTimeout(connect, 3000);
      };
    }

    connect();

    return () => {
      clearTimeout(reconnectTimer);
      esRef.current?.close();
      esRef.current = null;
    };
  }, [qc]);
}
