use std::collections::HashMap;
#[cfg(not(target_os = "macos"))]
use cache_size::{l1_cache_size, l2_cache_size, l3_cache_size};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use std::process::Command;
#[cfg(not(target_os = "macos"))]
use crate::tools::units::KILOBYTE;
use crate::tools::units::GIGABYTE;


pub struct SystemInfo;

impl SystemInfo {
    pub fn num_cores(sys: &mut System) -> usize {
        sys.cpus().len()
    }

    pub fn display_cores(sys: &mut System) -> Option<Vec<String>> {
        if sys.cpus().is_empty() {
            return None;
        }

        let mut cores = Vec::new();

        cores.push(String::new());

        for (i, cpu) in sys.cpus().iter().enumerate() {
            cores.push(format!(
                "Core {}: {:.1}%",
                i,
                cpu.cpu_usage(),
            ));
        }

        Some(cores)
    }

    pub fn display_cpu_frequency(sys: &mut System) -> Option<u64> {
        if sys.cpus().is_empty() {
            return None;
        }

        for cpu in sys.cpus() {
            if cpu.frequency().to_string().is_empty() {
                return None;
            }
            let freq = cpu.frequency();
            return Some(freq);
        }

        None
    }

    #[cfg(not(target_os = "macos"))]
    fn lscpu_l3_cache() -> Option<String> {
        let output = Command::new("lscpu").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.starts_with("L3 cache:") {
                let size = line
                    .split(':')
                    .nth(1)?
                    .trim()
                    .split_whitespace()
                    .take(2)            
                    .collect::<Vec<_>>()
                    .join(" ");

                if !size.is_empty() {
                    return Some(size);
                }
            }
        }

        None
    }

    #[cfg(not(target_os = "macos"))]
    pub fn display_cpu_cache() -> Option<HashMap<&'static str, String>> {
        let mut cache_info = HashMap::new();

        if let Some(size) = l1_cache_size() {
            cache_info.insert("L1", format!("{} KB", size / KILOBYTE as usize));
        }

        if let Some(size) = l2_cache_size() {
            cache_info.insert("L2", format!("{} KB", size / KILOBYTE as usize));
        }
        
        if let Some(size) = Self::lscpu_l3_cache() {
            cache_info.insert("L3", size);
        }

        if cache_info.is_empty() {
            None
        } else {
            Some(cache_info)
        }
    }

    pub fn display_cpu_model(sys: &mut System) -> Option<HashMap<&'static str, String>> {
        if sys.cpus().is_empty() {
            return None;
        }
        let mut info = HashMap::new();

        if let Some(cpu) = sys.cpus().first() {
            info.insert("Brand", cpu.brand().to_string());
        }

        Some(info)
    }
 
    pub fn display_memory(sys: &mut System) -> Option<Vec<String>> {
        let mut info = Vec::new();

        let total= sys.total_memory() as f64 / GIGABYTE;
        let used = sys.used_memory() as f64 / GIGABYTE;
        
        let total_swap: f64 = sys.total_swap() as f64 / GIGABYTE;
        let used_swap: f64 = sys.used_swap() as f64 / GIGABYTE;

        info.push(format!("Total: {:.2} GB    Total Swap: {:.2} GB", total, total_swap));
        info.push(format!("Used: {:.2} GB    Used Swap: ({:.2} GB)", used, used_swap));

        Some(info)
    }

    pub fn set_refresh_timer(sys: &mut System) {
        sys.refresh_cpu_usage();
        sys.refresh_cpu_frequency();
        sys.refresh_memory();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu(),
        );
    }

    pub fn get_core_usages(sys: &mut System) -> Vec<f64> {
        sys.cpus().iter().map(|cpu| cpu.cpu_usage() as f64).collect()
    } 

    pub fn display_kernel(&mut self) -> Option<String> {
        if let Some(s) = System::kernel_version() {
            Some(s)
        } else {
            None
        }
    }

    pub fn display_host_name(&mut self) -> Option<String> {
        if let Some(s) = System::host_name() {
            Some(s)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn cache_l1_format_kb() {
        let bytes: usize = 32 * KILOBYTE as usize;
        let formatted = format!("{} KB", bytes / KILOBYTE as usize);
        assert_eq!(formatted, "32 KB");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn cache_l2_format_kb() {
        let bytes: usize = 512 * KILOBYTE as usize;
        let formatted = format!("{} KB", bytes / KILOBYTE as usize);
        assert_eq!(formatted, "512 KB");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn cache_zero_bytes_formats_as_zero_kb() {
        let bytes: usize = 0;
        let formatted = format!("{} KB", bytes / KILOBYTE as usize);
        assert_eq!(formatted, "0 KB");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn display_cpu_cache_returns_some() {
        let cache = SystemInfo::display_cpu_cache();
        assert!(cache.is_some(), "expected cache info on this machine");
        let map = cache.unwrap();
        assert!(map.contains_key("L1"), "expected L1 cache entry");
        for (key, val) in &map {
            assert!(!val.is_empty(), "{} cache value should not be empty", key);
        }
    }

    #[test]
    fn memory_format_total_gb() {
        let bytes: u64 = 16 * 1024 * 1024 * 1024;
        let gb = bytes as f64 / GIGABYTE;
        let formatted = format!("{:.2} GB", gb);
        assert_eq!(formatted, "16.00 GB");
    }

    #[test]
    fn memory_format_fractional_gb() {
        let bytes: u64 = (6.5 * GIGABYTE) as u64;
        let gb = bytes as f64 / GIGABYTE;
        let formatted = format!("{:.2} GB", gb);
        assert_eq!(formatted, "6.50 GB");
    }

    #[test]
    fn memory_display_line_format() {
        let total: f64 = 16.0;
        let total_swap: f64 = 8.0;
        let line = format!("Total: {:.2} GB    Total Swap: {:.2} GB", total, total_swap);
        assert_eq!(line, "Total: 16.00 GB    Total Swap: 8.00 GB");
    }

    #[test]
    fn memory_used_line_format() {
        let used: f64 = 5.25;
        let used_swap: f64 = 1.10;
        let line = format!("Used: {:.2} GB    Used Swap: ({:.2} GB)", used, used_swap);
        assert_eq!(line, "Used: 5.25 GB    Used Swap: (1.10 GB)");
    }

    #[test]
    fn display_memory_returns_two_lines() {
        let mut sys = System::new_all();
        sys.refresh_memory();
        let info = SystemInfo::display_memory(&mut sys);
        assert!(info.is_some());
        let lines = info.unwrap();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("Total:"));
        assert!(lines[1].starts_with("Used:"));
    }

    #[test]
    fn core_format_string() {
        let formatted = format!("Core {}: {:.1}%", 0, 45.678);
        assert_eq!(formatted, "Core 0: 45.7%");
    }

    #[test]
    fn core_format_zero_usage() {
        let formatted = format!("Core {}: {:.1}%", 3, 0.0);
        assert_eq!(formatted, "Core 3: 0.0%");
    }

    #[test]
    fn core_format_hundred_percent() {
        let formatted = format!("Core {}: {:.1}%", 7, 100.0);
        assert_eq!(formatted, "Core 7: 100.0%");
    }

    #[test]
    fn display_cores_first_entry_is_empty() {
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        if let Some(cores) = SystemInfo::display_cores(&mut sys) {
            assert_eq!(cores[0], "");
            for entry in &cores[1..] {
                assert!(entry.starts_with("Core "), "unexpected format: {}", entry);
                assert!(entry.contains('%'), "missing % in: {}", entry);
            }
        }
    }
}
