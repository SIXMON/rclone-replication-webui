import { Outlet } from 'react-router-dom';
import { Sidebar } from './components/layout/Sidebar';
import { useGlobalEvents } from './hooks/useGlobalEvents';

export default function App() {
  useGlobalEvents();

  return (
    <div className="flex min-h-screen bg-surface-50">
      <Sidebar />
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
