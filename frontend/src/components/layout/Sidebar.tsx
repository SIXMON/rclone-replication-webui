import { NavLink } from 'react-router-dom';
import { Server, RefreshCw, Bell } from 'lucide-react';

const links = [
  { to: '/tasks', label: 'Tâches', desc: 'Gérer les réplications', icon: RefreshCw },
  { to: '/remotes', label: 'Stockages', desc: 'Sources et destinations', icon: Server },
  { to: '/notifications', label: 'Notifications', desc: 'Alertes et canaux', icon: Bell },
];

export function Sidebar() {
  return (
    <aside className="w-60 shrink-0 bg-surface-900 text-white flex flex-col min-h-screen">
      {/* Logo */}
      <div className="px-5 py-5 border-b border-white/10">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-brand-600 flex items-center justify-center shadow-lg shadow-brand-600/25">
            <RefreshCw size={15} className="text-white" />
          </div>
          <div>
            <span className="font-semibold text-sm tracking-tight">rclone-ui</span>
            <p className="text-[10px] text-surface-500 leading-tight">Réplication de fichiers</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-4">
        <p className="px-2 mb-2.5 text-[10px] font-semibold uppercase tracking-widest text-surface-500">
          Navigation
        </p>
        <ul className="space-y-0.5">
          {links.map(({ to, label, desc, icon: Icon }) => (
            <li key={to}>
              <NavLink
                to={to}
                className={({ isActive }) =>
                  `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-150 group ${
                    isActive
                      ? 'bg-brand-600/20 text-brand-300 font-medium'
                      : 'text-surface-400 hover:bg-white/5 hover:text-white'
                  }`
                }
              >
                <Icon size={17} className="shrink-0" />
                <div className="min-w-0">
                  <span className="block leading-tight">{label}</span>
                  <span className="block text-[10px] text-surface-500 group-hover:text-surface-400 leading-tight">
                    {desc}
                  </span>
                </div>
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>

      {/* Footer */}
      <div className="px-5 py-4 border-t border-white/10">
        <p className="text-[10px] text-surface-600">
          Propulsé par <span className="text-surface-400">rclone</span>
        </p>
      </div>
    </aside>
  );
}
