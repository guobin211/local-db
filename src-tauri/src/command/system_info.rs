use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_percentage: f32,
    pub swap_used: u64,
    pub load_average: f64,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percentage: f32,
}

/// 获取系统信息
#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // 获取 CPU 使用率
    let cpu_usage = sys.global_cpu_usage();

    // 获取内存信息
    let memory_total = sys.total_memory();
    let memory_used = sys.used_memory();
    let memory_percentage = if memory_total > 0 {
        (memory_used as f64 / memory_total as f64 * 100.0) as f32
    } else {
        0.0
    };

    // 获取 Swap 使用量
    let swap_used = sys.used_swap();

    // 获取负载平均值
    let load_average = System::load_average().one;

    // 获取磁盘信息
    let disks = Disks::new_with_refreshed_list();
    let disk_infos: Vec<DiskInfo> = disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let usage_percentage = if total > 0 {
                (used as f64 / total as f64 * 100.0) as f32
            } else {
                0.0
            };

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percentage,
            }
        })
        .collect();

    Ok(SystemInfo {
        cpu_usage,
        memory_total,
        memory_used,
        memory_percentage,
        swap_used,
        load_average,
        disks: disk_infos,
    })
}

/// 获取 CPU 使用率
#[tauri::command]
pub fn get_cpu_usage() -> Result<f32, String> {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();

    // 等待一小段时间以获取准确的 CPU 使用率
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    Ok(sys.global_cpu_usage())
}

/// 获取内存信息
#[tauri::command]
pub fn get_memory_info() -> Result<(u64, u64, f32), String> {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let percentage = if total > 0 {
        (used as f64 / total as f64 * 100.0) as f32
    } else {
        0.0
    };

    Ok((total, used, percentage))
}

/// 获取磁盘信息
#[tauri::command]
pub fn get_disk_info() -> Result<Vec<DiskInfo>, String> {
    let disks = Disks::new_with_refreshed_list();
    let disk_infos: Vec<DiskInfo> = disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let usage_percentage = if total > 0 {
                (used as f64 / total as f64 * 100.0) as f32
            } else {
                0.0
            };

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percentage,
            }
        })
        .collect();

    Ok(disk_infos)
}
