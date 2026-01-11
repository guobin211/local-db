export enum DBStatus {
  RUNNING = 'Running',
  STOPPED = 'Stopped',
  NOT_INSTALLED = 'Not Installed'
}

export interface DBInstance {
  id: string;
  name: string;
  version: string;
  type: string;
  status: DBStatus;
  port: string;
  meta: string;
  icon: string;
  colorClass: string;
}

export interface LogEntry {
  timestamp: string;
  level: 'INFO' | 'WARN' | 'ERROR';
  message: string;
}

export type ViewType = 'dashboard' | 'instances' | 'logs' | 'backup';
