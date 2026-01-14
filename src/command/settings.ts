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

const THEME_KEY = 'local-db-theme';

// 获取存储的主题
export function getStoredTheme(): string {
  return localStorage.getItem(THEME_KEY) || 'light';
}

// 应用主题到文档
export function applyTheme(theme: string): void {
  if (theme === 'dark') {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
}

// 设置存储的主题
export function setStoredTheme(theme: string): void {
  localStorage.setItem(THEME_KEY, theme);
  applyTheme(theme);
}

// 获取全局设置
export async function getSettings(): Promise<GlobalSettings> {
  const settings: GlobalSettings = await invoke('get_settings');
  // 使用 localStorage 中的主题覆盖后端设置
  settings.theme = getStoredTheme();
  return settings;
}

// 更新全局设置
export async function updateSettings(settings: GlobalSettings): Promise<OperationResult> {
  // 同时更新 localStorage
  setStoredTheme(settings.theme);
  return invoke('update_settings', { settings });
}
