import { invoke } from '@tauri-apps/api/core';

// 数据库类型
export type DatabaseType = 'mysql' | 'postgresql' | 'mongodb' | 'redis' | 'qdrant' | 'seekdb' | 'surrealdb';

// 数据库状态
export type DatabaseStatus = 'running' | 'stopped' | 'notinstalled';

// 任务状态
export type TaskStatus = 'pending' | 'running' | 'completed' | 'failed';

// 异步任务信息接口
export interface AsyncTask {
  id: string;
  task_type: string;
  db_type: string;
  status: TaskStatus;
  progress: number;
  message: string;
  error?: string;
  created_at: string;
  updated_at: string;
}

// 数据库信息接口
export interface DatabaseInfo {
  id: string;
  name: string;
  type: string;
  version?: string;
  install_path?: string;
  data_path?: string;
  log_path?: string;
  port?: number;
  username?: string;
  password?: string;
  config?: string;
  status?: DatabaseStatus;
  auto_start?: boolean;
  pid?: number;
  created_at?: string;
  updated_at?: string;
  icon?: string;
  meta?: string;
}

// 操作结果接口
export interface OperationResult<T = void> {
  success: boolean;
  message: string;
  data?: T;
}

// 安装数据库参数
export interface InstallDatabaseParams {
  db_type: string; // 改为 string 以支持所有数据库类型
  version?: string;
  port?: number;
  username?: string;
  password?: string;
}

// 获取所有数据库列表
export async function getDatabases(): Promise<DatabaseInfo[]> {
  return invoke('get_databases');
}

// 根据ID获取数据库信息
export async function getDatabase(id: string): Promise<DatabaseInfo | null> {
  return invoke('get_database', { id });
}

// 根据数据库类型获取数据库
export async function getDatabaseByType(dbType: DatabaseType): Promise<DatabaseInfo | null> {
  return invoke('get_database_by_type', { dbType });
}

// 启动数据库
export async function startDatabase(id: string): Promise<OperationResult> {
  return invoke('start_database', { id });
}

// 停止数据库
export async function stopDatabase(id: string): Promise<OperationResult> {
  return invoke('stop_database', { id });
}

// 重启数据库
export async function restartDatabase(id: string): Promise<OperationResult> {
  return invoke('restart_database', { id });
}

// 获取数据库状态
export async function getDatabaseStatus(id: string): Promise<DatabaseStatus | null> {
  return invoke('get_database_status', { id });
}

// 删除数据库
export async function deleteDatabase(id: string, withData: boolean = false): Promise<OperationResult> {
  return invoke('delete_database', { id, withData });
}

// 安装数据库（异步，返回任务ID）
export async function installDatabase(params: InstallDatabaseParams): Promise<string> {
  return invoke('install_database', { params });
}

// 获取任务状态（用于轮询）
export async function getTaskStatus(taskId: string): Promise<AsyncTask | null> {
  return invoke('get_task_status', { taskId });
}

// 更新数据库自启动设置
export async function updateDatabaseAutostart(id: string, autoStart: boolean): Promise<OperationResult> {
  return invoke('update_database_autostart', { id, autoStart });
}

// 同步所有数据库的运行状态
// 此函数用于在页面加载完成后调用，检查并更新所有数据库的实际运行状态
export async function syncDatabasesStatus(): Promise<DatabaseInfo[]> {
  return invoke('sync_databases_status');
}
