import React, { useState, useRef } from 'react';
import { FiChevronDown, FiSearch, FiDownload, FiTrash2 } from 'react-icons/fi';
import { LogEntry } from '../types';

const MOCK_LOGS: LogEntry[] = [
  {
    timestamp: '2023-10-24 14:22:01.452',
    level: 'INFO',
    message: 'Control [main] - MongoDB starting : pid=12402 port=27017 dbpath=/data/db 64-bit'
  },
  {
    timestamp: '2023-10-24 14:22:01.453',
    level: 'INFO',
    message: 'Control [main] - targetMinOS: Windows 7/Windows Server 2008 R2'
  },
  {
    timestamp: '2023-10-24 14:22:02.110',
    level: 'WARN',
    message:
      'Storage [initandlisten] - WiredTiger message [1698150122:110731][12402:0x700003e8c000], checkpoint: process_checkpoint: 124 checkpoint took 11ms'
  },
  {
    timestamp: '2023-10-24 14:22:02.500',
    level: 'INFO',
    message: 'Network [initandlisten] - waiting for connections on port 27017'
  },
  {
    timestamp: '2023-10-24 14:25:33.001',
    level: 'ERROR',
    message: 'Network [conn42] - SocketException: remote: 127.0.0.1:54312 error: End of file'
  },
  {
    timestamp: '2023-10-24 14:26:01.212',
    level: 'INFO',
    message: 'Index [initandlisten] - Build index on: local.oplog.rs properties: { v: 2, key: { ts: 1 }, name: "ts_1" }'
  },
  {
    timestamp: '2023-10-24 14:28:15.890',
    level: 'INFO',
    message:
      'Command [conn45] - command admin.$cmd command: listDatabases { listDatabases: 1.0, $db: "admin" } numYields:0 ok:1'
  },
  {
    timestamp: '2023-10-24 14:30:00.002',
    level: 'WARN',
    message: 'Access [conn46] - Auth failed for user "root" from 127.0.0.1: Generic auth error'
  },
  {
    timestamp: '2023-10-24 14:32:44.112',
    level: 'INFO',
    message: 'Sharding [initandlisten] - autosplitter not started: not in sharded mode'
  },
  {
    timestamp: '2023-10-24 14:35:10.452',
    level: 'INFO',
    message: 'Control [main] - Periodic background task thread starting'
  },
  {
    timestamp: '2023-10-24 14:35:11.000',
    level: 'INFO',
    message: 'Connection accepted from 192.168.1.15:52001 #47 (1 connection now open)'
  }
];

export const ResourceLogs: React.FC = () => {
  const [logs, _setLogs] = useState<LogEntry[]>(MOCK_LOGS);
  const [filter, setFilter] = useState('');
  const logEndRef = useRef<HTMLDivElement>(null);

  const filteredLogs = logs.filter(
    (l) =>
      l.message.toLowerCase().includes(filter.toLowerCase()) || l.level.toLowerCase().includes(filter.toLowerCase())
  );

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'INFO':
        return 'text-blue-600 dark:text-blue-400';
      case 'WARN':
        return 'text-amber-600 dark:text-amber-400';
      case 'ERROR':
        return 'text-red-600 dark:text-red-500';
      default:
        return 'text-slate-500 dark:text-slate-400';
    }
  };

  const getBgColor = (level: string) => {
    switch (level) {
      case 'ERROR':
        return 'bg-red-500/10';
      case 'WARN':
        return 'bg-amber-500/5';
      default:
        return '';
    }
  };

  return (
    <div className="flex h-full flex-col gap-4">
      {/* Top Controls */}
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="relative">
            <select className="dark:bg-card-dark dark:border-border-dark focus:ring-primary focus:border-primary h-10 appearance-none rounded-lg border border-gray-200 bg-white px-4 pr-10 text-sm font-bold text-slate-900 transition-all focus:ring-1 dark:text-white">
              <option>MongoDB (Running)</option>
              <option>Redis (Running)</option>
              <option>Neo4j (Stopped)</option>
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
          <button className="dark:bg-card-dark dark:border-border-dark flex h-10 items-center gap-2 rounded-lg border border-gray-200 bg-white px-4 text-xs font-bold text-slate-500 transition-all hover:text-slate-900 dark:text-[#9da6b9] dark:hover:text-white">
            <FiDownload size={18} />
            Export
          </button>
          <button className="flex h-10 items-center gap-2 rounded-lg border border-red-500/20 bg-red-500/10 px-4 text-xs font-bold text-red-600 transition-all hover:bg-red-500/20 dark:text-red-500">
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
              STDOUT — MONGODB.LOG
            </p>
          </div>
          <div className="flex items-center gap-1.5 rounded bg-green-500/10 px-2 py-0.5 text-[10px] font-bold text-green-600 dark:text-green-500">
            <span className="h-1.5 w-1.5 rounded-full bg-green-600 dark:bg-green-500"></span>
            LIVE
          </div>
        </div>

        {/* Log Stream */}
        <div className="selection:bg-primary/30 flex-1 overflow-y-auto scroll-smooth p-4 font-mono text-[12px] leading-relaxed">
          {filteredLogs.map((log, i) => (
            <div
              key={i}
              className={`group flex gap-4 rounded-sm px-2 py-1.5 transition-colors ${getBgColor(log.level)}`}
            >
              <span className="shrink-0 text-slate-400 opacity-100 transition-opacity select-none group-hover:opacity-100 dark:text-slate-600 dark:opacity-40">
                {log.timestamp}
              </span>
              <span className={`w-12 shrink-0 font-bold ${getLevelColor(log.level)}`}>{log.level}</span>
              <span className="break-all text-slate-700 dark:text-slate-300">{log.message}</span>
            </div>
          ))}
          <div ref={logEndRef} />
        </div>

        {/* Terminal Footer */}
        <div className="dark:border-border-dark flex items-center justify-between border-t border-gray-200 bg-slate-50 px-5 py-2 dark:bg-[#1c1f27]">
          <div className="flex items-center gap-4 font-mono text-[10px] text-slate-500 dark:text-[#9da6b9]">
            <span>Lines: {filteredLogs.length}</span>
            <span>Size: 2.4 MB</span>
          </div>
          <div className="flex items-center gap-2 font-mono text-[10px] text-slate-500 dark:text-[#9da6b9]">
            Auto-scroll: <span className="text-primary font-bold">ON</span>
          </div>
        </div>
      </div>
    </div>
  );
};
