import React from 'react';
import { BarChart, Bar, ResponsiveContainer, Cell } from 'recharts';
import { ViewType } from '../types';

const data = [
  { val: 10 },
  { val: 15 },
  { val: 12 },
  { val: 18 },
  { val: 22 },
  { val: 35 },
  { val: 28 },
  { val: 20 },
  { val: 12 },
  { val: 15 }
];

interface DashboardProps {
  onViewChange: (view: ViewType) => void;
}

export const Dashboard: React.FC<DashboardProps> = ({ onViewChange }) => {
  return (
    <div className="space-y-6">
      {/* Top Cards */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        {[
          {
            label: 'Total Databases',
            value: '6',
            icon: 'inventory_2',
            color: 'text-primary',
            border: 'border-transparent'
          },
          { label: 'Running', value: '3', icon: 'play_circle', color: 'text-green-500', border: 'border-l-green-500' },
          { label: 'Stopped', value: '3', icon: 'stop_circle', color: 'text-slate-400', border: 'border-l-slate-400' }
        ].map((card, i) => (
          <div
            key={i}
            className={`border-border-dark bg-card-dark flex flex-col gap-2 rounded-xl border p-6 shadow-sm ${card.border} ${card.border !== 'border-transparent' ? 'border-l-4' : ''}`}
          >
            <div className="flex items-center justify-between">
              <p className="text-sm font-medium text-[#9da6b9]">{card.label}</p>
              <span className={`material-symbols-outlined ${card.color}`}>{card.icon}</span>
            </div>
            <p className="text-3xl font-bold text-white">{card.value}</p>
          </div>
        ))}
      </div>

      {/* Charts Grid */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        {/* CPU Usage */}
        <div className="bg-card-dark border-border-dark rounded-xl border p-6 shadow-sm">
          <div className="mb-6 flex items-center justify-between">
            <h4 className="flex items-center gap-2 text-sm font-bold text-white">
              <span className="material-symbols-outlined text-primary text-[20px]">memory</span>
              CPU Usage
            </h4>
            <span className="text-primary font-mono text-sm font-bold">12.4%</span>
          </div>
          <div className="h-32 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={data}>
                <Bar dataKey="val" radius={[2, 2, 0, 0]}>
                  {data.map((_entry, index) => (
                    <Cell key={`cell-${index}`} fill={index === 5 ? '#135bec' : 'rgba(19, 91, 236, 0.3)'} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
          <p className="mt-3 text-[10px] tracking-wider text-[#9da6b9] uppercase opacity-60">
            Real-time load average: 0.85
          </p>
        </div>

        {/* Memory Usage */}
        <div className="bg-card-dark border-border-dark rounded-xl border p-6 shadow-sm">
          <div className="mb-6 flex items-center justify-between">
            <h4 className="flex items-center gap-2 text-sm font-bold text-white">
              <span className="material-symbols-outlined text-[20px] text-purple-500">memory_alt</span>
              Memory
            </h4>
            <span className="font-mono text-sm font-bold text-purple-500">4.2 GB / 16 GB</span>
          </div>
          <div className="flex h-32 flex-col items-center justify-center">
            <div className="relative size-28">
              <svg className="size-full" viewBox="0 0 100 100">
                <circle
                  className="stroke-border-dark"
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
                  strokeDasharray="54.6 360"
                  strokeLinecap="round"
                  strokeWidth="8"
                  transform="rotate(150 50 50)"
                ></circle>
              </svg>
              <div className="absolute inset-0 flex flex-col items-center justify-center pt-2">
                <span className="text-xl font-bold text-white">26%</span>
                <span className="text-[8px] font-bold tracking-widest text-[#9da6b9] uppercase">Used</span>
              </div>
            </div>
          </div>
          <p className="mt-3 text-[10px] tracking-wider text-[#9da6b9] uppercase opacity-60">Swap usage: 124MB</p>
        </div>

        {/* Disk Usage */}
        <div className="bg-card-dark border-border-dark rounded-xl border p-6 shadow-sm">
          <div className="mb-6 flex items-center justify-between">
            <h4 className="flex items-center gap-2 text-sm font-bold text-white">
              <span className="material-symbols-outlined text-[20px] text-amber-500">database_upload</span>
              Disk Usage
            </h4>
            <span className="font-mono text-sm font-bold text-amber-500">78%</span>
          </div>
          <div className="flex h-32 flex-col justify-center gap-4">
            <div>
              <div className="mb-1 flex justify-between text-[10px] font-bold text-[#9da6b9]">
                <span>SYSTEM SSD (C:)</span>
                <span>234GB / 512GB</span>
              </div>
              <div className="bg-border-dark h-2 w-full overflow-hidden rounded-full">
                <div className="h-full w-[45%] bg-amber-500"></div>
              </div>
            </div>
            <div>
              <div className="mb-1 flex justify-between text-[10px] font-bold text-[#9da6b9]">
                <span>DB DATA VOLUME (D:)</span>
                <span>780GB / 1TB</span>
              </div>
              <div className="bg-border-dark h-2 w-full overflow-hidden rounded-full">
                <div className="h-full w-[78%] bg-amber-600"></div>
              </div>
            </div>
          </div>
          <p className="mt-3 text-[10px] tracking-wider text-[#9da6b9] uppercase opacity-60">
            IOPS: 12.4k read / 4.1k write
          </p>
        </div>
      </div>

      {/* Mini Active Instances Table */}
      <div className="bg-card-dark border-border-dark overflow-hidden rounded-xl border shadow-sm">
        <div className="border-border-dark flex items-center justify-between border-b px-6 py-5">
          <h3 className="text-lg font-bold text-white">Recent Instances</h3>
          <button
            onClick={() => onViewChange('instances')}
            className="text-primary flex items-center gap-1 text-xs font-bold hover:underline"
          >
            View all instances <span className="material-symbols-outlined text-[14px]">arrow_forward</span>
          </button>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full border-collapse text-left">
            <thead>
              <tr className="bg-white/5">
                <th className="px-6 py-4 text-[10px] font-bold tracking-wider text-[#9da6b9] uppercase">Database</th>
                <th className="px-6 py-4 text-right text-[10px] font-bold tracking-wider text-[#9da6b9] uppercase">
                  Status
                </th>
              </tr>
            </thead>
            <tbody className="divide-border-dark divide-y">
              {[
                { name: 'MongoDB', status: 'Running', color: 'text-green-500' },
                { name: 'Redis', status: 'Running', color: 'text-green-500' },
                { name: 'Neo4j', status: 'Stopped', color: 'text-[#9da6b9]' }
              ].map((db, i) => (
                <tr key={i} className="transition-colors hover:bg-white/5">
                  <td className="px-6 py-4">
                    <p className="text-sm font-bold text-white">{db.name}</p>
                  </td>
                  <td className="px-6 py-4 text-right">
                    <span className={`text-xs font-bold ${db.color}`}>{db.status}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
