import React, { useState, useEffect } from 'react';
import { FiMessageCircle } from 'react-icons/fi';
import { Sidebar } from './components/sidebar';
import { Dashboard } from './components/dashboard';
import { InstancesView } from './components/instances-view';
import { ResourceLogs } from './components/resource-logs';
import { Settings } from './components/settings';
import { ViewType } from './types';
import { syncDatabasesStatus } from './command/database';
import { getStoredTheme, applyTheme } from './command/settings';

const App: React.FC = () => {
  const [currentView, setCurrentView] = useState<ViewType>('dashboard');

  // 页面加载完成后同步数据库状态和应用主题
  useEffect(() => {
    // 应用保存的主题
    applyTheme(getStoredTheme());

    syncDatabasesStatus().catch((err) => {
      console.error('Failed to sync databases status:', err);
    });
  }, []);

  const renderView = () => {
    switch (currentView) {
      case 'dashboard':
        return <Dashboard onViewChange={setCurrentView} />;
      case 'instances':
        return <InstancesView />;
      case 'logs':
        return <ResourceLogs />;
      case 'settings':
        return <Settings />;
      default:
        return <Dashboard onViewChange={setCurrentView} />;
    }
  };

  return (
    <div className="bg-background-light dark:bg-background-dark flex h-screen overflow-hidden text-slate-900 transition-colors duration-200 dark:text-slate-100">
      <Sidebar activeView={currentView} onViewChange={setCurrentView} />

      <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
        {/* 顶部拖动区域 */}
        <div className="h-8 w-full shrink-0" data-tauri-drag-region />
        <main className="flex-1 overflow-y-auto p-3 md:p-5">
          <div className="mx-auto h-full max-w-7xl">{renderView()}</div>
        </main>
        <footer className="border-border-dark flex items-center justify-between border-t px-6 py-3 text-[#9da6b9]">
          <div className="flex items-center gap-6">
            <a href="#" className="hover:text-primary flex items-center gap-1.5 text-xs transition-colors">
              <FiMessageCircle size={16} />
              Discord Support
            </a>
          </div>
          <p className="font-mono text-[10px] tracking-widest uppercase opacity-60">
            © 2026 LocalDB Management Suite. All rights reserved.
          </p>
        </footer>
      </div>
    </div>
  );
};

export default App;
