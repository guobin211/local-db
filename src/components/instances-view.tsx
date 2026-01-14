import React, { useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  FiGrid,
  FiZap,
  FiShare2,
  FiRepeat,
  FiList,
  FiAlertTriangle,
  FiChevronLeft,
  FiChevronRight,
  FiLoader,
  FiCheck
} from 'react-icons/fi';
import {
  getDatabases,
  startDatabase,
  stopDatabase,
  installDatabase,
  DatabaseInfo,
  AsyncTask
} from '../command/database';

// Task status enum for runtime comparison
const TaskStatus = {
  PENDING: 'pending',
  RUNNING: 'running',
  COMPLETED: 'completed',
  FAILED: 'failed'
} as const;

export const SUPPORTED_DATABASES = [
  { id: 'mysql', type: 'mysql', name: 'MySQL', icon: 'grid_view', colorClass: 'text-blue-500 bg-blue-500/10' },
  {
    id: 'postgresql',
    type: 'postgresql',
    name: 'PostgreSQL',
    icon: 'grid_view',
    colorClass: 'text-blue-600 bg-blue-600/10'
  },
  {
    id: 'mongodb',
    type: 'mongodb',
    name: 'MongoDB',
    icon: 'grid_view',
    colorClass: 'text-green-500 bg-green-500/10'
  },
  { id: 'redis', type: 'redis', name: 'Redis', icon: 'bolt', colorClass: 'text-red-500 bg-red-500/10' },
  { id: 'qdrant', type: 'qdrant', name: 'Qdrant', icon: 'hub', colorClass: 'text-indigo-500 bg-indigo-500/10' },
  {
    id: 'surrealdb',
    type: 'surrealdb',
    name: 'SurrealDB',
    icon: 'all_inclusive',
    colorClass: 'text-orange-500 bg-orange-500/10'
  }
];

const getIconComponent = (iconName: string) => {
  switch (iconName) {
    case 'grid_view':
      return FiGrid;
    case 'bolt':
      return FiZap;
    case 'hub':
      return FiShare2;
    case 'all_inclusive':
      return FiRepeat;
    case 'table_rows':
      return FiList;
    default:
      return FiList;
  }
};

interface InstancesViewProps {}

export const InstancesView: React.FC<InstancesViewProps> = () => {
  const [databases, setDatabases] = useState<DatabaseInfo[]>([]);
  const [loading, setLoading] = useState<string | null>(null); // Track which db type is loading
  const [installTask, setInstallTask] = useState<AsyncTask | null>(null);
  const pollingRef = useRef<number | null>(null);

  // Clear polling on unmount
  useEffect(() => {
    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
      }
    };
  }, []);

  // 加载数据库列表
  const loadDatabases = async () => {
    try {
      const dbs = await getDatabases();
      console.log('dbs', dbs);
      setDatabases(dbs);
    } catch (error) {
      console.error('Failed to load databases:', error);
    }
  };

  // 初始加载
  useEffect(() => {
    let isMounted = true;
    let unlistenProgress: (() => void) | undefined;
    let unlistenStatus: (() => void) | undefined;

    loadDatabases();

    const setupListeners = async () => {
      // 监听安装进度事件
      const u1 = await listen<AsyncTask>('install-progress', (event) => {
        if (!isMounted) return;
        console.log('Install progress:', event.payload);
        setInstallTask(event.payload);

        if (event.payload.status === TaskStatus.COMPLETED || event.payload.status === TaskStatus.FAILED) {
          setLoading(null);
          loadDatabases();
          setTimeout(() => {
            if (isMounted) setInstallTask(null);
          }, 3000);
        }
      });

      if (!isMounted) {
        u1();
      } else {
        unlistenProgress = u1;
      }

      // 监听数据库状态更新事件（来自后台同步）
      const u2 = await listen<DatabaseInfo[]>('databases-updated', (event) => {
        if (!isMounted) return;
        console.log('Databases updated from backend:', event.payload);
        setDatabases(event.payload);
      });

      if (!isMounted) {
        u2();
      } else {
        unlistenStatus = u2;
      }
    };

    setupListeners();

    return () => {
      isMounted = false;
      if (unlistenProgress) unlistenProgress();
      if (unlistenStatus) unlistenStatus();
    };
  }, []);

  // 获取数据库的显示信息（图标、颜色等）
  const getDbDisplayInfo = (dbType: string) => {
    const found = SUPPORTED_DATABASES.find((db) => db.type === dbType);
    return found || { name: dbType, icon: 'grid_view', colorClass: 'text-gray-500 bg-gray-500/10', type: dbType };
  };

  // 处理安装数据库（异步）
  const handleInstall = async (dbType: string) => {
    setLoading(dbType);
    try {
      await installDatabase({ db_type: dbType });
      // 进度更新现在通过 tauri event 监听，不再需要手动轮询
    } catch (error) {
      console.error('Failed to install database:', error);
      setLoading(null);
    }
  };

  // 处理启动数据库
  const handleStart = async (id: string, name: string) => {
    setLoading('starting');
    try {
      const result = await startDatabase(id);
      if (result.success) {
        await loadDatabases();
      } else {
        console.error(`Failed to start ${name}: ${result.message}`);
      }
    } catch (error) {
      console.error('Failed to start database:', error);
    } finally {
      setLoading(null);
    }
  };

  // 处理停止数据库
  const handleStop = async (id: string, name: string) => {
    setLoading('stopping');
    try {
      const result = await stopDatabase(id);
      if (result.success) {
        await loadDatabases();
      } else {
        console.error(`Failed to stop ${name}: ${result.message}`);
      }
    } catch (error) {
      console.error('Failed to stop database:', error);
    } finally {
      setLoading(null);
    }
  };

  // Helper to get disabled prop value
  const getDisabledValue = (isLoading: boolean): boolean | undefined => {
    return isLoading ? true : undefined;
  };

  // 合并已安装的数据库和未安装的数据库
  const installedTypes = new Set(databases.map((db) => db.type));
  const notInstalledDbs = SUPPORTED_DATABASES.filter((db) => !installedTypes.has(db.type)).map((db) => ({
    id: `not-installed-${db.type}`,
    name: db.name,
    version: 'Not Installed',
    type: db.type,
    status: 'notinstalled',
    port: '-',
    meta: 'Click to install',
    icon: db.icon,
    colorClass: db.colorClass
  }));

  const allDatabases = [...databases, ...notInstalledDbs].sort((a, b) => a.name.localeCompare(b.name));

  return (
    <div className="dark:bg-card-dark dark:border-border-dark flex h-full flex-col overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
      <div className="dark:border-border-dark dark:bg-card-dark flex shrink-0 items-center justify-between border-b border-gray-200 bg-white px-5 py-4">
        <h3 className="text-lg font-bold text-slate-900 dark:text-white">Active Instances</h3>
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-slate-500 dark:text-[#9da6b9]">Sort by:</span>
          <button className="hover:text-primary flex items-center gap-1 text-xs font-bold text-slate-900 transition-colors dark:text-white">
            Name <FiChevronRight size={14} className="rotate-90" />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-x-auto">
        <table className="w-full min-w-200 border-collapse text-left">
          <thead className="sticky top-0 z-10">
            <tr className="bg-slate-50 dark:bg-[#1c1f27]">
              <th className="px-3 py-3 pl-5 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                Database
              </th>
              <th className="px-3 py-3 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                Type
              </th>
              <th className="px-3 py-3 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                Status
              </th>
              <th className="px-3 py-3 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                Port / Meta
              </th>
              <th className="px-3 py-3 pr-5 text-right text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="dark:divide-border-dark divide-y divide-gray-200">
            {allDatabases.map((db) => {
              const IconComponent = getIconComponent(db.icon || 'grid_view');
              const displayInfo = getDbDisplayInfo(db.type);
              return (
                <tr key={db.id} className="group transition-colors hover:bg-slate-50 dark:hover:bg-white/2">
                  <td className="px-3 py-3.5 pl-5">
                    <div className="flex items-center gap-3">
                      <div className={`flex size-9 items-center justify-center rounded-lg ${displayInfo.colorClass}`}>
                        <IconComponent size={18} />
                      </div>
                      <div>
                        <p className="text-sm font-bold text-slate-900 dark:text-white">{db.name}</p>
                        <p className="text-xs font-medium text-slate-500 dark:text-[#9da6b9]">{db.version}</p>
                      </div>
                    </div>
                  </td>
                  <td className="px-3 py-3.5">
                    <span className="text-xs font-medium text-slate-500 dark:text-[#9da6b9]">{db.type}</span>
                  </td>
                  <td className="px-3 py-3.5">
                    {db.status === 'running' && (
                      <span className="inline-flex items-center gap-1.5 rounded-full bg-green-500/10 px-2 py-0.5 text-[10px] font-bold tracking-wider text-green-500 uppercase">
                        <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-green-500"></span>
                        Running
                      </span>
                    )}
                    {db.status === 'stopped' && (
                      <span className="inline-flex items-center gap-1.5 rounded-full bg-slate-100 px-2 py-0.5 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:bg-white/5 dark:text-[#9da6b9]">
                        <span className="h-1.5 w-1.5 rounded-full bg-slate-500"></span>
                        Stopped
                      </span>
                    )}
                    {db.status === 'notinstalled' && (
                      <span className="inline-flex items-center gap-1.5 rounded-full bg-amber-500/10 px-2 py-0.5 text-[10px] font-bold tracking-wider text-amber-500 uppercase">
                        <FiAlertTriangle size={14} />
                        Not Installed
                      </span>
                    )}
                  </td>
                  <td className="px-3 py-3.5">
                    {db.status === 'running' ? (
                      <p className="font-mono text-xs text-slate-500 dark:text-[#9da6b9]">{db.port}</p>
                    ) : (
                      <p className="text-[11px] text-slate-500 italic opacity-60 dark:text-[#9da6b9]">
                        {db.meta || '-'}
                      </p>
                    )}
                  </td>
                  <td className="px-3 py-3.5 pr-5 text-right">
                    <div className="flex items-center justify-end gap-2">
                      {db.status === 'running' ? (
                        <>
                          <button
                            onClick={() => handleStop(db.id, db.name)}
                            disabled={getDisabledValue(!!loading)}
                            className="rounded-lg border border-red-500/30 px-3 py-1 text-[10px] font-bold tracking-wider text-red-500 uppercase transition-all hover:bg-red-500/10 disabled:opacity-50"
                          >
                            Stop
                          </button>
                        </>
                      ) : db.status === 'stopped' ? (
                        <>
                          <button
                            onClick={() => handleStart(db.id, db.name)}
                            disabled={getDisabledValue(!!loading)}
                            className="bg-primary hover:bg-primary/90 rounded-lg px-3 py-1 text-[10px] font-bold tracking-wider text-white uppercase transition-all disabled:opacity-50"
                          >
                            Start
                          </button>
                        </>
                      ) : // Show loading state for install button
                      installTask?.db_type === db.type && installTask.status !== TaskStatus.COMPLETED ? (
                        <button
                          disabled={true}
                          className="bg-primary/80 shadow-primary/20 flex cursor-not-allowed items-center gap-2 rounded-lg px-3 py-1 text-[10px] font-bold tracking-wider text-white uppercase transition-all"
                        >
                          <FiLoader size={14} className="animate-spin" />
                          {installTask.status === TaskStatus.FAILED ? 'Failed' : `${installTask.progress}%`}
                        </button>
                      ) : installTask?.db_type === db.type && installTask.status === TaskStatus.COMPLETED ? (
                        <button
                          disabled={true}
                          className="flex items-center gap-2 rounded-lg bg-green-600 px-3 py-1 text-[10px] font-bold tracking-wider text-white uppercase transition-all"
                        >
                          <FiCheck size={14} />
                          Done
                        </button>
                      ) : (
                        <button
                          onClick={() => handleInstall(db.type)}
                          disabled={!!loading}
                          className="bg-primary hover:bg-primary/90 shadow-primary/20 rounded-lg px-3 py-1 text-[10px] font-bold tracking-wider text-white uppercase shadow-lg transition-all disabled:opacity-50"
                        >
                          Install
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div className="dark:border-border-dark flex shrink-0 items-center justify-between border-t border-gray-200 bg-slate-50 px-5 py-3 dark:bg-[#1c1f27]">
        <p className="text-xs font-medium text-slate-500 dark:text-[#9da6b9]">
          Showing {allDatabases.length} database engines ({databases.length} installed)
        </p>
        <div className="flex gap-2">
          <button
            className="rounded-md p-1.5 text-slate-500 hover:bg-slate-200 disabled:opacity-20 dark:text-[#9da6b9] dark:hover:bg-white/5"
            disabled
          >
            <FiChevronLeft size={20} />
          </button>
          <button className="rounded-md p-1.5 text-slate-500 hover:bg-slate-200 dark:text-[#9da6b9] dark:hover:bg-white/5">
            <FiChevronRight size={20} />
          </button>
        </div>
      </div>
    </div>
  );
};
