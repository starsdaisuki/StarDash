use serde::{Deserialize, Serialize};
use sysinfo::{Components, Disks, Networks, System};

// ========== 数据结构 ==========

#[derive(Serialize)]
pub struct CpuInfo {
    name: String,
    usage: f32,
    cores: usize,
    core_usages: Vec<f32>,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    total_gb: f64,
    used_gb: f64,
    usage_percent: f64,
}

#[derive(Serialize)]
pub struct DiskInfo {
    name: String,
    mount_point: String,
    total_gb: f64,
    used_gb: f64,
    available_gb: f64,
    usage_percent: f64,
    fs_type: String,
}

#[derive(Serialize)]
pub struct NetworkInterface {
    name: String,
    received_bytes: u64,
    transmitted_bytes: u64,
    mac_address: String,
    ip_addresses: Vec<String>,
}

#[derive(Serialize)]
pub struct TempInfo {
    label: String,
    temperature: f32,
}

#[derive(Serialize)]
pub struct ProcessInfo {
    name: String,
    pid: u32,
    cpu_usage: f32,
    memory_mb: f64,
}

#[derive(Serialize)]
pub struct BatteryInfo {
    percentage: f32,
    is_charging: bool,
    state: String,
    health_percent: f32,
    time_to_empty_mins: Option<f64>,
    time_to_full_mins: Option<f64>,
    cycle_count: Option<u32>,
}

#[derive(Serialize)]
pub struct SystemOverview {
    os_name: String,
    host_name: String,
    uptime: u64,
}

#[derive(Serialize)]
pub struct FullSystemInfo {
    overview: SystemOverview,
    cpu: CpuInfo,
    memory: MemoryInfo,
    disks: Vec<DiskInfo>,
    networks: Vec<NetworkInterface>,
    temperatures: Vec<TempInfo>,
    top_processes: Vec<ProcessInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct PublicIpInfo {
    ip: String,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    org: Option<String>,
}

// ========== 命令 ==========

#[tauri::command]
fn get_system_info() -> FullSystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_usage();

    let cpus = sys.cpus();

    // 磁盘
    let disks = Disks::new_with_refreshed_list();
    let disk_list: Vec<DiskInfo> = disks
        .iter()
        .map(|d| {
            let total = d.total_space() as f64 / 1_073_741_824.0;
            let available = d.available_space() as f64 / 1_073_741_824.0;
            let used = total - available;
            DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                total_gb: total,
                used_gb: used,
                available_gb: available,
                usage_percent: if total > 0.0 { used / total * 100.0 } else { 0.0 },
                fs_type: d.file_system().to_string_lossy().to_string(),
            }
        })
        .collect();

    // 网络接口
    let networks = Networks::new_with_refreshed_list();
    let network_list: Vec<NetworkInterface> = networks
        .iter()
        .map(|(name, data)| {
            let ips: Vec<String> = data.ip_networks().iter().map(|ip| ip.addr.to_string()).collect();
            NetworkInterface {
                name: name.clone(),
                received_bytes: data.total_received(),
                transmitted_bytes: data.total_transmitted(),
                mac_address: data.mac_address().to_string(),
                ip_addresses: ips,
            }
        })
        .collect();

    // 温度
    let components = Components::new_with_refreshed_list();
    let temp_list: Vec<TempInfo> = components
        .iter()
        .filter(|c| c.temperature().unwrap_or(0.0) > 0.0)
        .map(|c| TempInfo {
            label: c.label().to_string(),
            temperature: c.temperature().unwrap_or(0.0),
        })
        .collect();

    // 进程 Top 10 (按 CPU 使用率排序)
    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|p| ProcessInfo {
            name: p.name().to_string_lossy().to_string(),
            pid: p.pid().as_u32(),
            cpu_usage: p.cpu_usage(),
            memory_mb: p.memory() as f64 / 1_048_576.0,
        })
        .collect();
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
    processes.truncate(10);

    FullSystemInfo {
        overview: SystemOverview {
            os_name: System::long_os_version().unwrap_or_default(),
            host_name: System::host_name().unwrap_or_default(),
            uptime: System::uptime(),
        },
        cpu: CpuInfo {
            name: cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
            usage: sys.global_cpu_usage(),
            cores: cpus.len(),
            core_usages: cpus.iter().map(|c| c.cpu_usage()).collect(),
        },
        memory: MemoryInfo {
            total_gb: sys.total_memory() as f64 / 1_073_741_824.0,
            used_gb: sys.used_memory() as f64 / 1_073_741_824.0,
            usage_percent: sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0,
        },
        disks: disk_list,
        networks: network_list,
        temperatures: temp_list,
        top_processes: processes,
    }
}

#[tauri::command]
async fn get_public_ip() -> Result<PublicIpInfo, String> {
    let resp = reqwest::get("https://ipinfo.io/json")
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    resp.json::<PublicIpInfo>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

#[tauri::command]
fn get_battery_info() -> Option<BatteryInfo> {
    let manager = battery::Manager::new().ok()?;
    let mut batteries = manager.batteries().ok()?;
    let battery = batteries.next()?.ok()?;

    let state = match battery.state() {
        battery::State::Charging => "充电中",
        battery::State::Discharging => "放电中",
        battery::State::Full => "已充满",
        _ => "未知",
    };

    Some(BatteryInfo {
        percentage: battery.state_of_charge().value * 100.0,
        is_charging: battery.state() == battery::State::Charging,
        state: state.to_string(),
        health_percent: battery.state_of_health().value * 100.0,
        time_to_empty_mins: battery.time_to_empty().map(|t| t.value as f64 / 60.0),
        time_to_full_mins: battery.time_to_full().map(|t| t.value as f64 / 60.0),
        cycle_count: battery.cycle_count(),
    })
}

// ========== 入口 ==========

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            get_public_ip,
            get_battery_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}