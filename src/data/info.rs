use std::collections::HashMap;
#[cfg(not(target_os = "macos"))]
use cache_size::{l1_cache_size, l2_cache_size, l3_cache_size};
use sysinfo::System;

#[cfg(not(target_os = "macos"))]
const KILOBYTE: usize = 1024;
#[cfg(not(target_os = "macos"))]
const MEGABYTE: usize = 10000;
const GIGABYTE: f64 = 1024.0 * 1024.0 * 1024.0;


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
    pub fn display_cpu_cache() -> Option<HashMap<&'static str, String>> {
        let mut cache_info = HashMap::new();

        if let Some(size) = l1_cache_size() {
            cache_info.insert("L1", format!("{} KB", size / KILOBYTE));
        }

        if let Some(size) = l2_cache_size() {
            cache_info.insert("L2", format!("{} KB", size / KILOBYTE));
        }

        if let Some(size) = l3_cache_size() {
            cache_info.insert("L3", format!("{} MB", size / MEGABYTE));
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
 
    pub fn display_memory(sys: &mut System) -> Vec<String> {
        let mut info = Vec::new();

        let total= sys.total_memory() as f64 / GIGABYTE;
        let used = sys.used_memory() as f64 / GIGABYTE;
        
        let total_swap: f64 = sys.total_swap() as f64 / GIGABYTE;
        let used_swap: f64 = sys.used_swap() as f64 / GIGABYTE;

        info.push(format!("Total: {:.2} GB    Total Swap: {:.2} GB", total, total_swap));
        info.push(format!("Used: {:.2} GB    Used Swap: ({:.2} GB)", used, used_swap));

        info
    }

    pub fn set_refresh_timer(sys: &mut System) {
        sys.refresh_cpu_usage();
        sys.refresh_cpu_frequency();
        sys.refresh_memory();
    }

    pub fn get_core_usages(sys: &mut System) -> Vec<f64> {
        sys.cpus().iter().map(|cpu| cpu.cpu_usage() as f64).collect()
    } 
}

pub trait OsInfo {
    fn display_kernel(&self) -> String;
    fn display_host_name(&self) -> String;
}

impl OsInfo for SystemInfo {
    fn display_kernel(&self) -> String {
        System::kernel_version().unwrap_or_else(|| String::from("No Kernel data available"))
    }

    fn display_host_name(&self) -> String {
        System::host_name().unwrap_or_else(|| String::from("No Os data available"))
    }
}
