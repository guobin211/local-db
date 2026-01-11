import React, { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { Header } from './components/Header';
import { Dashboard } from './components/Dashboard';
import { InstancesView } from './components/InstancesView';
import { ResourceLogs } from './components/ResourceLogs';
import { ViewType } from './types';

const App: React.FC = () => {
  const [currentView, setCurrentView] = useState<ViewType>('dashboard');
  const [searchQuery, setSearchQuery] = useState('');

  const renderView = () => {
    switch (currentView) {
      case 'dashboard':
        return <Dashboard onViewChange={setCurrentView} />;
      case 'instances':
        return <InstancesView searchQuery={searchQuery} />;
      case 'logs':
        return <ResourceLogs />;
      default:
        return <Dashboard onViewChange={setCurrentView} />;
    }
  };

  return (
    <div className="bg-background-dark flex h-screen overflow-hidden text-slate-100">
      <Sidebar activeView={currentView} onViewChange={setCurrentView} />

      <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <Header searchQuery={searchQuery} onSearchChange={setSearchQuery} />

        <main className="flex-1 overflow-y-auto p-4 md:p-8">
          <div className="mx-auto h-full max-w-7xl">{renderView()}</div>
        </main>

        <footer className="border-border-dark flex items-center justify-between border-t px-8 py-4 text-[#9da6b9]">
          <div className="flex items-center gap-6">
            <a href="#" className="hover:text-primary flex items-center gap-1.5 text-xs transition-colors">
              <span className="material-symbols-outlined text-[16px]">description</span>
              Documentation
            </a>
            <a href="#" className="hover:text-primary flex items-center gap-1.5 text-xs transition-colors">
              <span className="material-symbols-outlined text-[16px]">chat</span>
              Discord Support
            </a>
          </div>
          <p className="font-mono text-[10px] tracking-widest uppercase opacity-60">Local Engine: Connected (v1.9.2)</p>
        </footer>
      </div>
    </div>
  );
};

export default App;
