import React from 'react';
import { FiDatabase, FiHome, FiActivity, FiSettings } from 'react-icons/fi';
import { ViewType } from '../types';

interface SidebarProps {
  activeView: ViewType;
  onViewChange: (view: ViewType) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ activeView, onViewChange }) => {
  const navItems = [
    { id: 'dashboard', label: 'Dashboard', icon: FiHome },
    { id: 'instances', label: 'Instances', icon: FiDatabase },
    { id: 'logs', label: 'Resource Logs', icon: FiActivity }
  ];

  return (
    <aside className="dark:border-border-dark dark:bg-card-dark flex w-50 shrink-0 flex-col justify-between border-r border-gray-200 bg-white p-4 transition-all">
      <div className="flex flex-col gap-8">
        <div className="flex items-center gap-3.5 p-2">
          <div className="bg-primary shadow-primary/20 flex size-10 shrink-0 items-center justify-center rounded-xl text-white shadow-lg">
            <FiDatabase size={24} />
          </div>
          <div className="flex min-w-0 flex-col">
            <h1 className="truncate text-sm leading-none font-bold tracking-tight text-slate-900 dark:text-white">
              LocalDB
            </h1>
            <p className="mt-1 text-[10px] font-bold tracking-widest text-slate-500 uppercase opacity-90 dark:text-slate-400">
              Manager
            </p>
          </div>
        </div>

        <nav className="flex flex-col gap-1.5">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.id}
                onClick={() => onViewChange(item.id as ViewType)}
                className={`group flex items-center gap-3 rounded-lg px-3 py-2.5 transition-all ${
                  activeView === item.id
                    ? 'bg-primary/10 text-primary'
                    : 'text-slate-500 hover:bg-slate-100 hover:text-slate-900 dark:text-[#9da6b9] dark:hover:bg-white/5 dark:hover:text-white'
                }`}
              >
                <Icon size={22} className={activeView === item.id ? 'filled-icon' : ''} />
                <p className={`text-sm ${activeView === item.id ? 'font-bold' : 'font-medium'}`}>{item.label}</p>
              </button>
            );
          })}
        </nav>
      </div>

      <div className="space-y-4">
        <button
          onClick={() => onViewChange('settings')}
          className={`group flex w-full items-center gap-3 rounded-lg px-3 py-2.5 transition-all ${
            activeView === 'settings'
              ? 'bg-primary/10 text-primary'
              : 'text-slate-500 hover:bg-slate-100 hover:text-slate-900 dark:text-[#9da6b9] dark:hover:bg-white/5 dark:hover:text-white'
          }`}
        >
          <FiSettings size={22} className={activeView === 'settings' ? 'filled-icon' : ''} />
          <p className={`text-sm ${activeView === 'settings' ? 'font-bold' : 'font-medium'}`}>Settings</p>
        </button>
      </div>
    </aside>
  );
};
