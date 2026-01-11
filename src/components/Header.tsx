import React from 'react';

interface HeaderProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
}

export const Header: React.FC<HeaderProps> = ({ searchQuery, onSearchChange }) => {
  return (
    <header className="border-border-dark bg-card-dark sticky top-0 z-50 flex items-center justify-between border-b px-8 py-3.5">
      <div className="flex flex-1 items-center gap-10">
        <div className="group relative w-full max-w-[400px]">
          <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3.5">
            <span className="material-symbols-outlined group-focus-within:text-primary text-[20px] text-[#9da6b9] transition-colors">
              search
            </span>
          </div>
          <input
            type="text"
            className="bg-border-dark focus:ring-primary focus:bg-background-dark block h-10 w-full rounded-lg border-none pr-4 pl-11 text-sm text-white placeholder-[#9da6b9] transition-all focus:ring-1"
            placeholder="Search databases, instances, logs..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
          />
        </div>
      </div>

      <div className="ml-4 flex items-center gap-3">
        <button className="bg-primary hover:bg-primary/90 shadow-primary/10 flex h-10 items-center justify-center gap-2 rounded-lg px-5 text-sm font-bold whitespace-nowrap text-white shadow-lg transition-all active:scale-95">
          <span className="material-symbols-outlined text-[20px]">add_circle</span>
          <span>Add Database</span>
        </button>
        <button
          className="bg-border-dark flex h-10 w-10 items-center justify-center rounded-lg text-white transition-all hover:bg-white/10 active:scale-95"
          title="Settings"
        >
          <span className="material-symbols-outlined">settings</span>
        </button>
      </div>
    </header>
  );
};
