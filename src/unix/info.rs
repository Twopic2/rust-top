use std::collections::HashMap;
#[cfg(not(target_os = "macos"))]
use cache_size::{l1_cache_size, l2_cache_size, l3_cache_size};
use sysinfo::{System, RefreshKind};

#[cfg(not(target_os = "macos"))]
const KILOBYTE: usize = 1024;
// ToDo: Make sure to fix MEGABYTE to match the proper value 
#[cfg(not(target_os = "macos"))]
const MEGABYTE: usize = 1024 * 1024;
const GIGABYTE: f64 = 1024.0 * 1024.0 * 1024.0;


pub struct SystemInfo {
    sys: System,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::everything()
        );
        sys.refresh_all();
        Self { sys }
    }

    pub fn num_cores(&self) -> usize {
        self.sys.cpus().len()
    }

    pub fn display_cores(&mut self) -> Option<Vec<String>> {
        if self.sys.cpus().is_empty() {
            return None;
        }

        let mut cores = Vec::new();

        cores.push(String::new());

        for (i, cpu) in self.sys.cpus().iter().enumerate() {
            cores.push(format!(
                "Core {}: {:.1}%",
                i,
                cpu.cpu_usage(),
            ));
        }

        Some(cores)
    }

    pub fn display_cpu_frequency(&mut self) -> Option<Vec<String>> {
        if self.sys.cpus().is_empty() {
            return None;
        }

        let mut cpu_frequency = Vec::new();
        cpu_frequency.push(String::new());

        for cpu in self.sys.cpus() {
            if cpu.frequency().to_string().is_empty() {
                return None;
            }

            cpu_frequency.push(cpu.frequency().to_string());
        }

        Some(cpu_frequency)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn display_cpu_cache(&self) -> Option<HashMap<&str, String>> {
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

    pub fn display_cpu_model(&self) -> Option<HashMap<&str, String>> {
        if self.sys.cpus().is_empty() {
            return None;
        }
        let mut info = HashMap::new();

        if let Some(cpu) = self.sys.cpus().first() {
            info.insert("Brand", cpu.brand().to_string());
        }

        Some(info)
    }
 
    pub fn display_memory(&self) -> Vec<String> {
        let mut info = Vec::new();

        let total= self.sys.total_memory() as f64 / GIGABYTE;
        let used = self.sys.used_memory() as f64 / GIGABYTE;
        let usage_percent = (used / total) * 100.0;

        info.push(format!("Total: {:.2} GB", total));
        info.push(format!("Used: {:.2} GB ({:.1}%)", used, usage_percent));

        info
    }

    pub fn set_refresh_timer(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_cpu_frequency();
        self.sys.refresh_memory();
    }

    pub fn get_core_usages(&self) -> Vec<f64> {
        self.sys.cpus().iter().map(|cpu| cpu.cpu_usage() as f64).collect()
    } 
}

pub trait OsInfo {
    fn display_kernel(&self) -> Vec<String>;
    fn display_host_name(&self) -> Vec<String>;
}

impl OsInfo for SystemInfo {
    fn display_kernel(&self) -> Vec<String> {
        let mut v = Vec::new();
        v.push(String::new());

        let kernel = System::kernel_version();

        v.push(format!("{}", kernel.unwrap_or_else(|| String::from("No Kernel data available"))));
        v
    }

    fn display_host_name(&self) -> Vec<String> {
        let mut v = Vec::new();
        v.push(String::new());

        let os = System::host_name();

        v.push(format!("{}", os.unwrap_or_else(|| String::from("No Os data available"))));
        v
    }
}
