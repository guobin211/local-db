import React from 'react';
import { DBInstance, DBStatus } from '../types';

const MOCK_DATABASES: DBInstance[] = [
  {
    id: '1',
    name: 'MongoDB',
    version: 'v6.0.5 community',
    type: 'NoSQL / Document',
    status: DBStatus.RUNNING,
    port: '27017',
    meta: '',
    icon: 'grid_view',
    colorClass: 'text-green-500 bg-green-500/10'
  },
  {
    id: '2',
    name: 'Redis',
    version: 'v7.0.10 stable',
    type: 'Key-Value / Cache',
    status: DBStatus.RUNNING,
    port: '6379',
    meta: '',
    icon: 'bolt',
    colorClass: 'text-red-500 bg-red-500/10'
  },
  {
    id: '3',
    name: 'Neo4j',
    version: 'v5.8.0 enterprise',
    type: 'Graph / Cypher',
    status: DBStatus.STOPPED,
    port: '7474',
    meta: 'Last active 2h ago',
    icon: 'account_tree',
    colorClass: 'text-blue-500 bg-blue-500/10'
  },
  {
    id: '4',
    name: 'Qdrant',
    version: 'v1.1.0 rust-native',
    type: 'Vector / AI',
    status: DBStatus.RUNNING,
    port: '6333',
    meta: '',
    icon: 'hub',
    colorClass: 'text-indigo-500 bg-indigo-500/10'
  },
  {
    id: '5',
    name: 'SurrealDB',
    version: 'Cloud-Native',
    type: 'Multi-model',
    status: DBStatus.NOT_INSTALLED,
    port: '8000',
    meta: 'Requires 150MB',
    icon: 'all_inclusive',
    colorClass: 'text-orange-500 bg-orange-500/10'
  },
  {
    id: '6',
    name: 'SeekDB',
    version: 'v0.4.2 beta',
    type: 'Relational',
    status: DBStatus.STOPPED,
    port: '-',
    meta: '-',
    icon: 'table_rows',
    colorClass: 'text-slate-400 bg-slate-400/10'
  }
];

interface InstancesViewProps {
  searchQuery: string;
}

export const InstancesView: React.FC<InstancesViewProps> = ({ searchQuery }) => {
  const filtered = MOCK_DATABASES.filter(
    (db) =>
      db.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      db.type.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="bg-card-dark border-border-dark flex h-full flex-col overflow-hidden rounded-xl border shadow-sm">
      <div className="border-border-dark bg-card-dark flex shrink-0 items-center justify-between border-b px-6 py-5">
        <h3 className="text-xl font-bold text-white">Active Instances</h3>
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-[#9da6b9]">Sort by:</span>
          <button className="hover:text-primary flex items-center gap-1 text-xs font-bold text-white transition-colors">
            Name <span className="material-symbols-outlined text-sm">keyboard_arrow_down</span>
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-x-auto">
        <table className="w-full min-w-[800px] border-collapse text-left">
          <thead className="sticky top-0 z-10">
            <tr className="bg-[#1c1f27]">
              <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-[#9da6b9] uppercase">Database</th>
              <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-[#9da6b9] uppercase">Type</th>
              <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-[#9da6b9] uppercase">Status</th>
              <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-[#9da6b9] uppercase">Port / Meta</th>
              <th className="px-6 py-4 text-right text-[10px] font-bold tracking-wider text-[#9da6b9] uppercase">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="divide-border-dark divide-y">
            {filtered.map((db) => (
              <tr key={db.id} className="group transition-colors hover:bg-white/[0.02]">
                <td className="px-6 py-5">
                  <div className="flex items-center gap-3">
                    <div className={`flex size-10 items-center justify-center rounded-lg ${db.colorClass}`}>
                      <span className="material-symbols-outlined">{db.icon}</span>
                    </div>
                    <div>
                      <p className="text-sm font-bold text-white">{db.name}</p>
                      <p className="text-xs font-medium text-[#9da6b9]">{db.version}</p>
                    </div>
                  </div>
                </td>
                <td className="px-6 py-5">
                  <span className="text-xs font-medium text-[#9da6b9]">{db.type}</span>
                </td>
                <td className="px-6 py-5">
                  {db.status === DBStatus.RUNNING && (
                    <span className="inline-flex items-center gap-1.5 rounded-full bg-green-500/10 px-2.5 py-1 text-[10px] font-bold tracking-wider text-green-500 uppercase">
                      <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-green-500"></span>
                      Running
                    </span>
                  )}
                  {db.status === DBStatus.STOPPED && (
                    <span className="inline-flex items-center gap-1.5 rounded-full bg-white/5 px-2.5 py-1 text-[10px] font-bold tracking-wider text-[#9da6b9] uppercase">
                      <span className="h-1.5 w-1.5 rounded-full bg-slate-500"></span>
                      Stopped
                    </span>
                  )}
                  {db.status === DBStatus.NOT_INSTALLED && (
                    <span className="inline-flex items-center gap-1.5 rounded-full bg-amber-500/10 px-2.5 py-1 text-[10px] font-bold tracking-wider text-amber-500 uppercase">
                      <span className="material-symbols-outlined text-[14px]">warning</span>
                      Not Installed
                    </span>
                  )}
                </td>
                <td className="px-6 py-5">
                  {db.status === DBStatus.RUNNING ? (
                    <p className="font-mono text-xs text-[#9da6b9]">{db.port}</p>
                  ) : (
                    <p className="text-[11px] text-[#9da6b9] italic opacity-60">{db.meta || '-'}</p>
                  )}
                </td>
                <td className="px-6 py-5 text-right">
                  {db.status === DBStatus.RUNNING ? (
                    <button className="rounded-lg border border-red-500/30 px-4 py-1.5 text-[11px] font-bold tracking-wider text-red-500 uppercase transition-all hover:bg-red-500/10">
                      Stop
                    </button>
                  ) : db.status === DBStatus.STOPPED ? (
                    <button className="bg-primary hover:bg-primary/90 rounded-lg px-4 py-1.5 text-[11px] font-bold tracking-wider text-white uppercase transition-all">
                      Start
                    </button>
                  ) : (
                    <button className="bg-primary hover:bg-primary/90 shadow-primary/20 rounded-lg px-4 py-1.5 text-[11px] font-bold tracking-wider text-white uppercase shadow-lg transition-all">
                      Install & Start
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="border-border-dark flex shrink-0 items-center justify-between border-t bg-[#1c1f27] px-6 py-4">
        <p className="text-xs font-medium text-[#9da6b9]">Showing {filtered.length} supported database engines</p>
        <div className="flex gap-2">
          <button className="rounded-md p-1.5 text-[#9da6b9] hover:bg-white/5 disabled:opacity-20" disabled>
            <span className="material-symbols-outlined text-[20px]">chevron_left</span>
          </button>
          <button className="rounded-md p-1.5 text-[#9da6b9] hover:bg-white/5">
            <span className="material-symbols-outlined text-[20px]">chevron_right</span>
          </button>
        </div>
      </div>
    </div>
  );
};
