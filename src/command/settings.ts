import { invoke } from '@tauri-apps/api/core';
import { OperationResult } from './database';

// 全局设置接口
export interface GlobalSettings {
  default_storage_path: string;
  auto_start: boolean;
  theme: string;
  language: string;
  auto_backup: boolean;
  backup_frequency: string;
  backup_retention_days: number;
  log_level: string;
  log_retention_days: number;
}

// 获取全局设置
export async function getSettings(): Promise<GlobalSettings> {
  return invoke('get_settings');
}

// 更新全局设置
export async function updateSettings(settings: GlobalSettings): Promise<OperationResult> {
  return invoke('update_settings', { settings });
}
