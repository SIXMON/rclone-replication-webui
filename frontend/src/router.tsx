import { createBrowserRouter, Navigate } from 'react-router-dom';
import App from './App';
import { RemotesPage } from './pages/RemotesPage';
import { RemoteFormPage } from './pages/RemoteFormPage';
import { TasksPage } from './pages/TasksPage';
import { TaskFormPage } from './pages/TaskFormPage';
import { TaskDetailPage } from './pages/TaskDetailPage';
import { TaskEditPage } from './pages/TaskEditPage';
import { NotificationsPage } from './pages/NotificationsPage';
import { NotificationFormPage } from './pages/NotificationFormPage';

export const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    children: [
      { index: true, element: <Navigate to="/tasks" replace /> },
      { path: 'remotes', element: <RemotesPage /> },
      { path: 'remotes/new', element: <RemoteFormPage /> },
      { path: 'remotes/:id/edit', element: <RemoteFormPage /> },
      { path: 'tasks', element: <TasksPage /> },
      { path: 'tasks/new', element: <TaskFormPage /> },
      { path: 'tasks/:id', element: <TaskDetailPage /> },
      { path: 'tasks/:id/edit', element: <TaskEditPage /> },
      { path: 'notifications', element: <NotificationsPage /> },
      { path: 'notifications/new', element: <NotificationFormPage /> },
      { path: 'notifications/:id/edit', element: <NotificationFormPage /> },
    ],
  },
]);
