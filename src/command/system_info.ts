import { invoke } from '@tauri-apps/api/core';

export interface DiskInfo {
  name: string;
  mount_point: string;
  total_space: number;
  available_space: number;
  used_space: number;
  usage_percentage: number;
}

export interface SystemInfo {
  cpu_usage: number;
  memory_total: number;
  memory_used: number;
  memory_percentage: number;
  swap_used: number;
  load_average: number;
  disks: DiskInfo[];
}

/**
 * 获取系统信息（CPU、内存、磁盘等）
 */
export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke('get_system_info');
}

/**
 * 获取 CPU 使用率
 */
export async function getCpuUsage(): Promise<number> {
  return invoke('get_cpu_usage');
}

/**
 * 获取内存信息
 * @returns [total, used, percentage]
 */
export async function getMemoryInfo(): Promise<[number, number, number]> {
  return invoke('get_memory_info');
}

/**
 * 获取磁盘信息
 */
export async function getDiskInfo(): Promise<DiskInfo[]> {
  return invoke('get_disk_info');
}
