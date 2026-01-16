import React, { useState, useRef, useEffect } from 'react';
import { FiChevronDown, FiSearch, FiDownload, FiTrash2 } from 'react-icons/fi';
import { getDatabases, DatabaseInfo, readDatabaseLogs, clearDatabaseLogs } from '../command/database';

interface ParsedLogEntry {
  timestamp: string;
  level: string;
  message: string;
  raw: string;
}

export const ResourceLogs: React.FC = () => {
  const [logs, setLogs] = useState<ParsedLogEntry[]>([]);
  const [filter, setFilter] = useState('');
  const [databases, setDatabases] = useState<DatabaseInfo[]>([]);
  const [selectedDbId, setSelectedDbId] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [autoScroll, setAutoScroll] = useState(true);
  const logEndRef = useRef<HTMLDivElement>(null);

  // 解析日志行，提取时间戳、级别和消息
  const parseLogLine = (line: string): ParsedLogEntry => {
    // 尝试匹配常见的日志格式
    // 格式1: [2024-01-16 14:22:01] INFO message
    const pattern1 = /^\[([^\]]+)\]\s+(\w+)\s+(.+)$/;
    // 格式2: 2024-01-16 14:22:01.452 INFO message
    const pattern2 = /^(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}(?:\.\d+)?)\s+(\w+)\s+(.+)$/;
    // 格式3: INFO: message (timestamp might be elsewhere or missing)
    const pattern3 = /^(\w+):\s*(.+)$/;

    let match = line.match(pattern1);
    if (match) {
      return {
        timestamp: match[1],
        level: match[2].toUpperCase(),
        message: match[3],
        raw: line,
      };
    }

    match = line.match(pattern2);
    if (match) {
      return {
        timestamp: match[1],
        level: match[2].toUpperCase(),
        message: match[3],
        raw: line,
      };
    }

    match = line.match(pattern3);
    if (match) {
      return {
        timestamp: new Date().toISOString().replace('T', ' ').substring(0, 23),
        level: match[1].toUpperCase(),
        message: match[2],
        raw: line,
      };
    }

    // 如果无法解析，将整行作为消息
    return {
      timestamp: '',
      level: 'INFO',
      message: line,
      raw: line,
    };
  };

  // 加载数据库列表
  useEffect(() => {
    const loadDatabases = async () => {
      try {
        const dbs = await getDatabases();
        // 过滤出已安装的数据库
        const installedDbs = dbs.filter((db) => db.status !== 'notinstalled');
        setDatabases(installedDbs);

        if (installedDbs.length > 0 && !selectedDbId) {
          setSelectedDbId(installedDbs[0].id);
        }
      } catch (error) {
        console.error('Failed to load databases:', error);
      }
    };

    loadDatabases();
  }, []);

  // 当选择的数据库改变时，加载日志
  useEffect(() => {
    if (!selectedDbId) {
      setLogs([]);
      return;
    }

    const loadLogs = async () => {
      setIsLoading(true);
      try {
        // 读取最后 1000 行日志
        const logLines = await readDatabaseLogs(selectedDbId, 1000);
        const parsedLogs = logLines.map(parseLogLine);
        setLogs(parsedLogs);
      } catch (error) {
        console.error('Failed to load logs:', error);
        setLogs([]);
      } finally {
        setIsLoading(false);
      }
    };

    loadLogs();
  }, [selectedDbId]);

  // 自动滚动到底部
  useEffect(() => {
    if (autoScroll && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  // 定时刷新日志（每5秒）
  useEffect(() => {
    if (!selectedDbId) return;

    const interval = setInterval(async () => {
      try {
        const logLines = await readDatabaseLogs(selectedDbId, 1000);
        const parsedLogs = logLines.map(parseLogLine);
        setLogs(parsedLogs);
      } catch (error) {
        console.error('Failed to refresh logs:', error);
      }
    }, 5000);

    return () => clearInterval(interval);
  }, [selectedDbId]);

  const filteredLogs = logs.filter(
    (l) =>
      l.message.toLowerCase().includes(filter.toLowerCase()) ||
      l.level.toLowerCase().includes(filter.toLowerCase())
  );

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'INFO':
        return 'text-blue-600 dark:text-blue-400';
      case 'WARN':
      case 'WARNING':
        return 'text-amber-600 dark:text-amber-400';
      case 'ERROR':
      case 'FATAL':
        return 'text-red-600 dark:text-red-500';
      case 'DEBUG':
        return 'text-slate-500 dark:text-slate-400';
      default:
        return 'text-slate-500 dark:text-slate-400';
    }
  };

  const getBgColor = (level: string) => {
    switch (level) {
      case 'ERROR':
      case 'FATAL':
        return 'bg-red-500/10';
      case 'WARN':
      case 'WARNING':
        return 'bg-amber-500/5';
      default:
        return '';
    }
  };

  // 导出日志功能
  const handleExportLogs = async () => {
    try {
      const selectedDb = databases.find((db) => db.id === selectedDbId);
      const dbName = selectedDb?.name || 'database';
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
      const fileName = `${dbName}-logs-${timestamp}.txt`;

      // 使用浏览器的下载功能
      const content = logs.map((log) => log.raw).join('\n');
      const blob = new Blob([content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      console.log('Logs exported successfully');
    } catch (error) {
      console.error('Failed to export logs:', error);
    }
  };

  // 清除日志功能
  const handleClearLogs = async () => {
    if (!selectedDbId) return;

    if (confirm('Are you sure you want to clear all logs? This action cannot be undone.')) {
      try {
        await clearDatabaseLogs(selectedDbId);
        setLogs([]);
        console.log('Logs cleared successfully');
      } catch (error) {
        console.error('Failed to clear logs:', error);
      }
    }
  };

  // 获取选中的数据库信息
  const selectedDb = databases.find((db) => db.id === selectedDbId);
  const dbLogFileName = selectedDb ? `${selectedDb.name.toLowerCase()}.log` : 'database.log';

  // 计算日志文件大小（估算）
  const logSizeKB = logs.reduce((acc, log) => acc + log.raw.length, 0) / 1024;
  const logSizeStr =
    logSizeKB < 1024 ? `${logSizeKB.toFixed(1)} KB` : `${(logSizeKB / 1024).toFixed(1)} MB`;

  return (
    <div className="flex h-full flex-col gap-4">
      {/* Top Controls */}
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="relative">
            <select
              value={selectedDbId}
              onChange={(e) => setSelectedDbId(e.target.value)}
              className="dark:bg-card-dark dark:border-border-dark focus:ring-primary focus:border-primary h-10 appearance-none rounded-lg border border-gray-200 bg-white px-4 pr-10 text-sm font-bold text-slate-900 transition-all focus:ring-1 dark:text-white"
            >
              {databases.length === 0 ? (
                <option value="">No databases installed</option>
              ) : (
                databases.map((db) => (
                  <option key={db.id} value={db.id}>
                    {db.name} ({db.status === 'running' ? 'Running' : 'Stopped'})
                  </option>
                ))
              )}
            </select>
            <FiChevronDown
              size={20}
              className="pointer-events-none absolute top-2.5 right-3 text-slate-500 dark:text-[#9da6b9]"
            />
          </div>

          <div className="group relative w-75">
            <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3.5">
              <FiSearch size={18} className="text-slate-400 dark:text-[#9da6b9]" />
            </div>
            <input
              type="text"
              className="dark:bg-card-dark dark:border-border-dark focus:ring-primary block h-10 w-full rounded-lg border border-gray-200 bg-white pr-4 pl-10 text-xs text-slate-900 placeholder-slate-400 transition-all focus:ring-1 dark:text-white dark:placeholder-[#9da6b9]"
              placeholder="Search logs..."
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
            />
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleExportLogs}
            disabled={logs.length === 0}
            className="dark:bg-card-dark dark:border-border-dark flex h-10 items-center gap-2 rounded-lg border border-gray-200 bg-white px-4 text-xs font-bold text-slate-500 transition-all hover:text-slate-900 disabled:opacity-50 disabled:cursor-not-allowed dark:text-[#9da6b9] dark:hover:text-white"
          >
            <FiDownload size={18} />
            Export
          </button>
          <button
            onClick={handleClearLogs}
            disabled={logs.length === 0}
            className="flex h-10 items-center gap-2 rounded-lg border border-red-500/20 bg-red-500/10 px-4 text-xs font-bold text-red-600 transition-all hover:bg-red-500/20 disabled:opacity-50 disabled:cursor-not-allowed dark:text-red-500"
          >
            <FiTrash2 size={18} />
            Clear Logs
          </button>
        </div>
      </div>

      {/* Terminal View */}
      <div className="dark:bg-card-dark dark:border-border-dark flex flex-1 flex-col overflow-hidden rounded-xl border border-gray-200 bg-white shadow-2xl">
        {/* Terminal Header */}
        <div className="dark:border-border-dark flex items-center justify-between border-b border-gray-200 bg-slate-50 px-5 py-3 dark:bg-[#1c1f27]">
          <div className="flex items-center gap-2">
            <div className="mr-4 flex gap-1.5">
              <div className="size-3 rounded-full bg-red-500/60"></div>
              <div className="size-3 rounded-full bg-amber-500/60"></div>
              <div className="size-3 rounded-full bg-green-500/60"></div>
            </div>
            <p className="font-mono text-[10px] font-bold tracking-widest text-[#9da6b9] uppercase">
              STDOUT — {dbLogFileName.toUpperCase()}
            </p>
          </div>
          <div className="flex items-center gap-1.5 rounded bg-green-500/10 px-2 py-0.5 text-[10px] font-bold text-green-600 dark:text-green-500">
            <span className="h-1.5 w-1.5 rounded-full bg-green-600 dark:bg-green-500"></span>
            LIVE
          </div>
        </div>

        {/* Log Stream */}
        <div className="selection:bg-primary/30 flex-1 overflow-y-auto scroll-smooth p-4 font-mono text-[12px] leading-relaxed">
          {isLoading ? (
            <div className="flex h-full items-center justify-center text-slate-500 dark:text-slate-400">
              Loading logs...
            </div>
          ) : filteredLogs.length === 0 ? (
            <div className="flex h-full items-center justify-center text-slate-500 dark:text-slate-400">
              {logs.length === 0 ? 'No logs available' : 'No logs match your search'}
            </div>
          ) : (
            <>
              {filteredLogs.map((log, i) => (
                <div
                  key={i}
                  className={`group flex gap-4 rounded-sm px-2 py-1.5 transition-colors ${getBgColor(log.level)}`}
                >
                  {log.timestamp && (
                    <span className="shrink-0 text-slate-400 opacity-100 transition-opacity select-none group-hover:opacity-100 dark:text-slate-600 dark:opacity-40">
                      {log.timestamp}
                    </span>
                  )}
                  <span className={`w-12 shrink-0 font-bold ${getLevelColor(log.level)}`}>{log.level}</span>
                  <span className="break-all text-slate-700 dark:text-slate-300">{log.message}</span>
                </div>
              ))}
              <div ref={logEndRef} />
            </>
          )}
        </div>

        {/* Terminal Footer */}
        <div className="dark:border-border-dark flex items-center justify-between border-t border-gray-200 bg-slate-50 px-5 py-2 dark:bg-[#1c1f27]">
          <div className="flex items-center gap-4 font-mono text-[10px] text-slate-500 dark:text-[#9da6b9]">
            <span>Lines: {filteredLogs.length}</span>
            <span>Size: {logSizeStr}</span>
          </div>
          <div className="flex items-center gap-2 font-mono text-[10px] text-slate-500 dark:text-[#9da6b9]">
            Auto-scroll:{' '}
            <button
              onClick={() => setAutoScroll(!autoScroll)}
              className={`font-bold ${autoScroll ? 'text-primary' : 'text-slate-500 dark:text-[#9da6b9]'}`}
            >
              {autoScroll ? 'ON' : 'OFF'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
