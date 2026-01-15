import React, { useEffect, useState } from 'react';
import {
  FiArrowRight,
  FiCpu,
  FiDatabase,
  FiHardDrive,
  FiPlayCircle,
  FiRefreshCw,
  FiStopCircle,
  FiUploadCloud
} from 'react-icons/fi';
import { Bar, BarChart, Cell, ResponsiveContainer } from 'recharts';
import { DatabaseInfo, getDatabases, getSystemInfo, SystemInfo } from '../command';
import { ViewType } from '../types';

// 数据库图标映射
const dbIcons: Record<string, string> = {
  mysql: '🐬',
  postgresql: '🐘',
  mongodb: '🍃',
  redis: '◆',
  qdrant: '🔍',
  seekdb: '📊',
  surrealdb: '🌐'
};

interface DashboardProps {
  onViewChange: (view: ViewType) => void;
}

export const Dashboard: React.FC<DashboardProps> = ({ onViewChange }) => {
  const [databases, setDatabases] = useState<DatabaseInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [cpuHistory, setCpuHistory] = useState<{ val: number }[]>([
    { val: 0 },
    { val: 0 },
    { val: 0 },
    { val: 0 },
    { val: 0 },
    { val: 0 },
    { val: 0 },
    { val: 0 },
    { val: 0 },
    { val: 0 }
  ]);

  const fetchDatabases = async () => {
    try {
      setRefreshing(true);
      const dbs = await getDatabases();
      setDatabases(dbs);
      setLastUpdate(new Date());
    } catch (error) {
      console.error('Failed to fetch databases:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  const fetchSystemInfo = async () => {
    try {
      const info = await getSystemInfo();
      setSystemInfo(info);

      // 更新 CPU 历史数据
      setCpuHistory((prev) => {
        const newHistory = [...prev.slice(1), { val: info.cpu_usage }];
        return newHistory;
      });
    } catch (error) {
      console.error('Failed to fetch system info:', error);
    }
  };

  useEffect(() => {
    fetchDatabases();
    fetchSystemInfo();

    // 自动刷新数据库信息，每30秒更新一次
    const dbInterval = setInterval(fetchDatabases, 30000);
    // 自动刷新系统信息，每3秒更新一次
    const sysInterval = setInterval(fetchSystemInfo, 3000);

    return () => {
      clearInterval(dbInterval);
      clearInterval(sysInterval);
    };
  }, []);

  const runningCount = databases.filter((db) => db.status === 'running').length;
  const stoppedCount = databases.filter((db) => db.status === 'stopped').length;
  const totalCount = databases.length;

  // 获取最近使用的数据库（最多显示3个）
  const recentInstances = databases
    .sort((a, b) => new Date(b.updated_at || '').getTime() - new Date(a.updated_at || '').getTime())
    .slice(0, 3);

  const formatUptime = (db: DatabaseInfo): string => {
    if (db.status !== 'running') return '-';

    const startTime = new Date(db.updated_at || '');
    const now = new Date();
    const diffMs = now.getTime() - startTime.getTime();

    const days = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

    if (days > 0) {
      return `${days}d ${hours}h ${minutes}m`;
    } else if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  };

  const formatLastUpdate = (): string => {
    const now = new Date();
    const diffMs = now.getTime() - lastUpdate.getTime();
    const seconds = Math.floor(diffMs / 1000);

    if (seconds < 60) {
      return 'Just now';
    } else if (seconds < 120) {
      return '1 minute ago';
    } else {
      const minutes = Math.floor(seconds / 60);
      return `${minutes} minutes ago`;
    }
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="border-primary size-12 animate-spin rounded-full border-4 border-t-transparent"></div>
          <p className="text-sm text-slate-500 dark:text-[#9da6b9]">Loading dashboard...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="mb-8">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-slate-900 dark:text-white">System Overview</h2>
            <p className="mt-1 text-sm text-slate-500 dark:text-[#9da6b9]">
              Real-time status of your local database environment
            </p>
          </div>
          <button
            onClick={fetchDatabases}
            disabled={refreshing}
            className="text-primary hover:bg-primary/10 flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-colors disabled:opacity-50"
          >
            <FiRefreshCw size={16} className={refreshing ? 'animate-spin' : ''} />
            Refresh
          </button>
        </div>
        <div className="mt-2 flex items-center justify-end">
          <p className="font-mono text-[10px] tracking-widest text-slate-500 uppercase dark:text-[#9da6b9]">
            Last updated: {formatLastUpdate()}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        {[
          {
            label: 'Total Databases',
            value: totalCount.toString(),
            icon: FiDatabase,
            color: 'text-primary',
            border: 'border-transparent'
          },
          {
            label: 'Running',
            value: runningCount.toString(),
            icon: FiPlayCircle,
            color: 'text-green-500',
            border: 'border-l-green-500'
          },
          {
            label: 'Stopped',
            value: stoppedCount.toString(),
            icon: FiStopCircle,
            color: 'text-slate-400',
            border: 'border-l-slate-400'
          }
        ].map((card, i) => (
          <div
            key={i}
            className={`dark:border-border-dark dark:bg-card-dark flex flex-col gap-2 rounded-xl border border-gray-200 bg-white p-6 shadow-sm ${card.border} ${card.border !== 'border-transparent' ? 'border-l-4' : ''}`}
          >
            <div className="flex items-center justify-between">
              <p className="text-sm font-medium text-slate-500 dark:text-[#9da6b9]">{card.label}</p>
              <card.icon className={card.color} size={20} />
            </div>
            <p className="text-3xl font-bold text-slate-900 dark:text-white">{card.value}</p>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <div className="dark:bg-card-dark dark:border-border-dark rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
          <div className="mb-4 flex items-center justify-between">
            <h4 className="flex items-center gap-2 text-sm font-bold text-slate-900 dark:text-white">
              <FiCpu size={18} className="text-primary" />
              CPU Usage
            </h4>
            <span className="text-primary font-mono text-sm font-bold">
              {systemInfo ? `${systemInfo.cpu_usage.toFixed(1)}%` : '...'}
            </span>
          </div>
          <div className="h-24 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={cpuHistory}>
                <Bar dataKey="val" radius={[2, 2, 0, 0]}>
                  {cpuHistory.map((_entry, index) => (
                    <Cell
                      key={`cell-${index}`}
                      fill={index === cpuHistory.length - 1 ? '#135bec' : 'rgba(19, 91, 236, 0.3)'}
                    />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
          <p className="mt-2 text-[10px] tracking-wider text-slate-500 uppercase opacity-60 dark:text-[#9da6b9]">
            Load average: {systemInfo ? systemInfo.load_average.toFixed(2) : '...'}
          </p>
        </div>

        {/* Memory Usage */}
        <div className="dark:bg-card-dark dark:border-border-dark rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
          <div className="mb-4 flex flex-col gap-1">
            <h4 className="flex items-center gap-2 text-sm font-bold text-slate-900 dark:text-white">
              <FiHardDrive size={18} className="text-purple-500" />
              Memory
            </h4>
            <span className="self-end font-mono text-xs font-bold text-purple-500">
              {systemInfo
                ? `${(systemInfo.memory_used / 1024 / 1024 / 1024).toFixed(1)} GB / ${(systemInfo.memory_total / 1024 / 1024 / 1024).toFixed(1)} GB`
                : '...'}
            </span>
          </div>
          <div className="flex h-24 flex-col items-center justify-center">
            <div className="relative size-20">
              <svg className="size-full" viewBox="0 0 100 100">
                <circle
                  className="dark:stroke-border-dark stroke-gray-200"
                  cx="50"
                  cy="50"
                  fill="none"
                  r="45"
                  strokeDasharray="210 360"
                  strokeLinecap="round"
                  strokeWidth="8"
                  transform="rotate(150 50 50)"
                ></circle>
                <circle
                  className="stroke-purple-500"
                  cx="50"
                  cy="50"
                  fill="none"
                  r="45"
                  strokeDasharray={`${systemInfo ? systemInfo.memory_percentage * 2.1 : 0} 360`}
                  strokeLinecap="round"
                  strokeWidth="8"
                  transform="rotate(150 50 50)"
                ></circle>
              </svg>
              <div className="absolute inset-0 flex flex-col items-center justify-center pt-2">
                <span className="text-lg font-bold text-white">
                  {systemInfo ? `${Math.round(systemInfo.memory_percentage)}%` : '...'}
                </span>
                <span className="text-[8px] font-bold tracking-widest text-[#9da6b9] uppercase">Used</span>
              </div>
            </div>
          </div>
          <p className="mt-2 text-[10px] tracking-wider text-slate-500 uppercase opacity-60 dark:text-[#9da6b9]">
            Swap usage: {systemInfo ? `${(systemInfo.swap_used / 1024 / 1024).toFixed(0)}MB` : '...'}
          </p>
        </div>

        {/* Disk Usage */}
        <div className="dark:bg-card-dark dark:border-border-dark rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
          <div className="mb-4 flex items-center justify-between">
            <h4 className="flex items-center gap-2 text-sm font-bold text-slate-900 dark:text-white">
              <FiUploadCloud size={18} className="text-amber-500" />
              Disk Usage
            </h4>
            <span className="font-mono text-sm font-bold text-amber-500">
              {systemInfo && systemInfo.disks.length > 0
                ? `${Math.round(systemInfo.disks[0].usage_percentage)}%`
                : '...'}
            </span>
          </div>
          <div className="flex h-24 flex-col justify-center gap-3">
            {systemInfo &&
              systemInfo.disks.slice(0, 2).map((disk, index) => (
                <div key={index}>
                  <div className="mb-1 flex justify-between text-[10px] font-bold text-[#9da6b9]">
                    <span>{disk.mount_point.toUpperCase()}</span>
                    <span>
                      {(disk.used_space / 1024 / 1024 / 1024).toFixed(0)}GB /{' '}
                      {(disk.total_space / 1024 / 1024 / 1024).toFixed(0)}GB
                    </span>
                  </div>
                  <div className="bg-border-dark h-2 w-full overflow-hidden rounded-full">
                    <div
                      className={index === 0 ? 'h-full bg-amber-500' : 'h-full bg-amber-600'}
                      style={{ width: `${disk.usage_percentage}%` }}
                    ></div>
                  </div>
                </div>
              ))}
          </div>
          <p className="mt-2 text-[10px] tracking-wider text-slate-500 uppercase opacity-60 dark:text-[#9da6b9]">
            {systemInfo && systemInfo.disks.length > 0
              ? `${systemInfo.disks.length} disk${systemInfo.disks.length > 1 ? 's' : ''} mounted`
              : '...'}
          </p>
        </div>
      </div>

      <div className="dark:bg-card-dark dark:border-border-dark overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
        <div className="dark:border-border-dark flex items-center justify-between border-b border-gray-200 px-6 py-5">
          <h3 className="text-lg font-bold text-slate-900 dark:text-white">Recent Instances</h3>
          <button
            onClick={() => onViewChange('instances')}
            className="text-primary flex items-center gap-1 text-xs font-bold hover:underline"
          >
            View all instances <FiArrowRight size={14} />
          </button>
        </div>
        {recentInstances.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16">
            <FiDatabase size={48} className="mb-4 text-slate-300 dark:text-slate-700" />
            <p className="text-sm font-medium text-slate-500 dark:text-[#9da6b9]">No databases installed</p>
            <p className="mt-1 text-xs text-slate-400 dark:text-slate-600">Install a database to get started</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left">
              <thead>
                <tr className="bg-slate-50 dark:bg-white/5">
                  <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                    Database
                  </th>
                  <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                    Version
                  </th>
                  <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                    Port
                  </th>
                  <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                    Uptime
                  </th>
                  <th className="px-6 py-4 text-right text-[10px] font-bold tracking-wider text-slate-500 uppercase dark:text-[#9da6b9]">
                    Status
                  </th>
                </tr>
              </thead>
              <tbody className="dark:divide-border-dark divide-y divide-gray-200">
                {recentInstances.map((db) => (
                  <tr key={db.id} className="transition-colors hover:bg-slate-50 dark:hover:bg-white/5">
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-3">
                        <span className="text-lg">{dbIcons[db.type] || '💾'}</span>
                        <p className="text-sm font-bold text-slate-900 capitalize dark:text-white">{db.name}</p>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <p className="text-sm text-slate-600 dark:text-[#9da6b9]">{db.version}</p>
                    </td>
                    <td className="px-6 py-4">
                      <p className="text-sm text-slate-600 dark:text-[#9da6b9]">{db.port}</p>
                    </td>
                    <td className="px-6 py-4">
                      <p className="text-sm text-slate-600 dark:text-[#9da6b9]">{formatUptime(db)}</p>
                    </td>
                    <td className="px-6 py-4 text-right">
                      <span
                        className={`text-xs font-bold uppercase ${
                          db.status === 'running'
                            ? 'text-green-500'
                            : db.status === 'stopped'
                              ? 'text-slate-500 dark:text-[#9da6b9]'
                              : 'text-amber-500'
                        }`}
                      >
                        {db.status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
