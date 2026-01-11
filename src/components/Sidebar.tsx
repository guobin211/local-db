import React from 'react';
import { ViewType } from '../types';

interface SidebarProps {
  activeView: ViewType;
  onViewChange: (view: ViewType) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ activeView, onViewChange }) => {
  const navItems = [
    { id: 'dashboard', label: 'Dashboard', icon: 'dashboard' },
    { id: 'instances', label: 'Instances', icon: 'database' },
    { id: 'logs', label: 'Resource Logs', icon: 'monitoring' },
  ];

  return (
    <aside className="border-border-dark bg-card-dark flex w-64 shrink-0 flex-col justify-between border-r p-4 transition-all">
      <div className="flex flex-col gap-8">
        {/* Branding Area (Logo Top Left) */}
        <div className="flex items-center gap-3.5 p-2">
          <div className="bg-primary shadow-primary/20 flex size-10 shrink-0 items-center justify-center rounded-xl text-white shadow-lg">
            <span className="material-symbols-outlined text-[24px]">database</span>
          </div>
          <div className="flex min-w-0 flex-col">
            <h1 className="truncate text-sm leading-none font-bold tracking-tight text-white">LocalDB</h1>
            <p className="mt-1 text-[10px] font-bold tracking-widest uppercase opacity-90">Manager</p>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex flex-col gap-1.5">
          {navItems.map((item) => (
            <button
              key={item.id}
              onClick={() => onViewChange(item.id as ViewType)}
              className={`group flex items-center gap-3 rounded-lg px-3 py-2.5 transition-all ${
                activeView === item.id
                  ? 'bg-primary/10 text-primary'
                  : 'text-[#9da6b9] hover:bg-white/5 hover:text-white'
              }`}
            >
              <span className={`material-symbols-outlined text-[22px] ${activeView === item.id ? 'filled-icon' : ''}`}>
                {item.icon}
              </span>
              <p className={`text-sm ${activeView === item.id ? 'font-bold' : 'font-medium'}`}>{item.label}</p>
            </button>
          ))}
        </nav>
      </div>

      <div className="space-y-4">
      </div>
    </aside>
  );
};
