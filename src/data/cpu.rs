use std::collections::HashMap;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[cfg(not(target_os = "macos"))]
use std::process::Command;
#[cfg(not(target_os = "macos"))]
use cache_size::{l1_cache_size, l2_cache_size};
#[cfg(not(target_os = "macos"))]
use crate::tools::units::KILOBYTE;

pub struct CpuInfo;

impl CpuInfo {
    pub fn num_cores(sys: &mut System) -> usize {
        sys.cpus().len()
    }

    pub fn display_cores(sys: &mut System) -> Option<Vec<String>> {
        let cpus = sys.cpus();
        if cpus.is_empty() {
            return None;
        }
        let mut cores = Vec::with_capacity(cpus.len() + 1);
        cores.push(String::new());
        cores.extend(
            cpus.iter()
                .enumerate()
                .map(|(i, cpu)| format!("Core {}: {:.1}%", i, cpu.cpu_usage())),
        );
        Some(cores)
    }

    pub fn display_cpu_frequency(sys: &mut System) -> Option<u64> {
        sys.cpus().first().map(|cpu| cpu.frequency())
    }

    pub fn display_cpu_model(sys: &mut System) -> Option<HashMap<&'static str, String>> {
        let brand = sys.cpus().first()?.brand().to_string();
        let mut info = HashMap::new();
        info.insert("Brand", brand);
        Some(info)
    }

    pub fn get_core_usages(sys: &mut System) -> Vec<f64> {
        sys.cpus().iter().map(|cpu| cpu.cpu_usage() as f64).collect()
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

    #[cfg(not(target_os = "macos"))]
    fn lscpu_l3_cache() -> Option<String> {
        let output = Command::new("lscpu").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find_map(|line| line.strip_prefix("L3 cache:"))
            .map(|rest| {
                rest.trim()
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|s| !s.is_empty())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn display_cpu_cache() -> Option<HashMap<&'static str, String>> {
        let mut cache_info = HashMap::new();
        let kb = |bytes: usize| format!("{} KB", bytes / KILOBYTE as usize);
        if let Some(size) = l1_cache_size() {
            cache_info.insert("L1", kb(size));
        }
        if let Some(size) = l2_cache_size() {
            cache_info.insert("L2", kb(size));
        }
        if let Some(size) = Self::lscpu_l3_cache() {
            cache_info.insert("L3", size);
        }
        (!cache_info.is_empty()).then_some(cache_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn cache_l1_format_kb() {
        let bytes: usize = 32 * KILOBYTE as usize;
        assert_eq!(format!("{} KB", bytes / KILOBYTE as usize), "32 KB");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn cache_l2_format_kb() {
        let bytes: usize = 512 * KILOBYTE as usize;
        assert_eq!(format!("{} KB", bytes / KILOBYTE as usize), "512 KB");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn cache_zero_bytes_formats_as_zero_kb() {
        let bytes: usize = 0;
        assert_eq!(format!("{} KB", bytes / KILOBYTE as usize), "0 KB");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn display_cpu_cache_returns_some() {
        let cache = CpuInfo::display_cpu_cache();
        assert!(cache.is_some(), "expected cache info on this machine");
        let map = cache.unwrap();
        assert!(map.contains_key("L1"), "expected L1 cache entry");
        for (key, val) in &map {
            assert!(!val.is_empty(), "{} cache value should not be empty", key);
        }
    }

    #[test]
    fn core_format_string() {
        assert_eq!(format!("Core {}: {:.1}%", 0, 45.678), "Core 0: 45.7%");
    }

    #[test]
    fn core_format_zero_usage() {
        assert_eq!(format!("Core {}: {:.1}%", 3, 0.0), "Core 3: 0.0%");
    }

    #[test]
    fn core_format_hundred_percent() {
        assert_eq!(format!("Core {}: {:.1}%", 7, 100.0), "Core 7: 100.0%");
    }

    #[test]
    fn display_cores_first_entry_is_empty() {
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        if let Some(cores) = CpuInfo::display_cores(&mut sys) {
            assert_eq!(cores[0], "");
            for entry in &cores[1..] {
                assert!(entry.starts_with("Core "), "unexpected format: {}", entry);
                assert!(entry.contains('%'), "missing % in: {}", entry);
            }
        }
    }
}
