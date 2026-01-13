export interface LogEntry {
  timestamp: string;
  level: 'INFO' | 'WARN' | 'ERROR';
  message: string;
}

export type ViewType = 'dashboard' | 'instances' | 'logs' | 'backup' | 'settings';
