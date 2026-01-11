import React, { useEffect, useState } from 'react';
import { FiMoon, FiSun, FiDownload, FiRefreshCw, FiCheck } from 'react-icons/fi';
import { useUpdater } from '../hooks/use-updater';

// 设置页面组件
export const Settings: React.FC = () => {
  const [isDark, setIsDark] = useState(false);
  const { updateInfo, isChecking, isInstalling, error, checkForUpdates, installUpdateNow } = useUpdater();

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [isDark]);

  const toggleTheme = () => setIsDark(!isDark);

  const handleCheckForUpdates = async () => {
    await checkForUpdates();
  };

  const handleInstallUpdate = async () => {
    await installUpdateNow();
  };

  return (
    <div className="dark:bg-card-dark dark:border-border-dark flex h-full flex-col overflow-hidden rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
      <h1 className="mb-6 text-2xl font-bold text-slate-900 dark:text-white">Settings</h1>
      <div className="space-y-6">
        <div className="space-y-4">
          <h2 className="dark:border-border-dark border-b border-gray-200 pb-2 text-lg font-semibold text-slate-900 dark:text-white">
            General
          </h2>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-slate-900 dark:text-white">Start on boot</p>
              <p className="text-xs text-slate-500 dark:text-slate-400">
                Automatically start LocalDB when system starts
              </p>
            </div>
            <button className="dark:bg-border-dark focus:ring-primary relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-gray-200 transition-colors duration-200 ease-in-out focus:ring-2 focus:ring-offset-2 focus:outline-none">
              <span className="pointer-events-none inline-block h-5 w-5 translate-x-0 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"></span>
            </button>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-slate-900 dark:text-white">Theme</p>
              <p className="text-xs text-slate-500 dark:text-slate-400">Theme</p>
            </div>
            <button
              className="dark:bg-border-dark flex h-10 w-10 items-center justify-center rounded-lg bg-gray-100 text-slate-600 transition-all hover:bg-gray-200 active:scale-95 dark:text-white dark:hover:bg-white/10"
              title={isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
              onClick={toggleTheme}
            >
              {isDark ? <FiSun size={20} /> : <FiMoon size={20} />}
            </button>
          </div>
        </div>
        <div className="space-y-4">
          <h2 className="dark:border-border-dark border-b border-gray-200 pb-2 text-lg font-semibold text-slate-900 dark:text-white">
            Updates
          </h2>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-slate-900 dark:text-white">Current Version</p>
                <p className="text-xs text-slate-500 dark:text-slate-400">0.1.0</p>
              </div>
              <button
                className="dark:bg-border-dark inline-flex items-center gap-2 rounded-lg bg-gray-100 px-4 py-2 text-sm font-medium text-slate-700 transition-all hover:bg-gray-200 disabled:cursor-not-allowed disabled:opacity-50 dark:text-white dark:hover:bg-white/10"
                onClick={handleCheckForUpdates}
                disabled={isChecking || isInstalling}
              >
                {isChecking ? (
                  <>
                    <FiRefreshCw size={16} className="animate-spin" />
                    Checking...
                  </>
                ) : (
                  <>
                    <FiRefreshCw size={16} />
                    Check for Updates
                  </>
                )}
              </button>
            </div>

            {updateInfo && (
              <div
                className={`rounded-lg p-4 ${
                  updateInfo.available ? 'bg-blue-50 dark:bg-blue-900/20' : 'bg-green-50 dark:bg-green-900/20'
                }`}
              >
                {updateInfo.available ? (
                  <>
                    <div className="mb-2 flex items-center gap-2">
                      <FiDownload size={18} className="text-blue-600 dark:text-blue-400" />
                      <p className="text-sm font-medium text-blue-900 dark:text-blue-100">
                        New Version Available: {updateInfo.latestVersion}
                      </p>
                    </div>
                    {updateInfo.body && (
                      <p className="mb-3 text-xs text-blue-800 dark:text-blue-200">{updateInfo.body}</p>
                    )}
                    <button
                      className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-all hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-blue-600 dark:hover:bg-blue-700"
                      onClick={handleInstallUpdate}
                      disabled={isInstalling}
                    >
                      {isInstalling ? (
                        <>
                          <FiRefreshCw size={16} className="animate-spin" />
                          Installing...
                        </>
                      ) : (
                        <>
                          <FiDownload size={16} />
                          Install Update
                        </>
                      )}
                    </button>
                  </>
                ) : (
                  <div className="flex items-center gap-2">
                    <FiCheck size={18} className="text-green-600 dark:text-green-400" />
                    <p className="text-sm font-medium text-green-900 dark:text-green-100">You are up to date!</p>
                  </div>
                )}
              </div>
            )}

            {error && (
              <div className="rounded-lg bg-red-50 p-4 dark:bg-red-900/20">
                <p className="text-sm font-medium text-red-900 dark:text-red-100">Update Error</p>
                <p className="text-xs text-red-800 dark:text-red-200">{error}</p>
              </div>
            )}
          </div>
        </div>
        <div className="space-y-4">
          <h2 className="dark:border-border-dark border-b border-gray-200 pb-2 text-lg font-semibold text-slate-900 dark:text-white">
            Data
          </h2>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700 dark:text-slate-300">
                Data Directory
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  readOnly
                  value="～/.local-db/data"
                  className="dark:border-border-dark focus:ring-primary focus:border-primary block w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-sm text-slate-900 dark:bg-[#1c1f27] dark:text-white"
                />
                <button className="dark:bg-border-dark inline-flex items-center rounded-lg bg-gray-100 px-4 py-2.5 text-center text-sm font-medium text-slate-700 hover:bg-gray-200 dark:text-white dark:hover:bg-white/10">
                  Change
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
