use std::{collections::HashMap};

use sysinfo::{System, RefreshKind};

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

        let total = self.sys.total_memory() as f64 / 1_073_741_824.0; 
        let used = self.sys.used_memory() as f64 / 1_073_741_824.0;
        let available = self.sys.available_memory() as f64 / 1_073_741_824.0;
        let usage_percent = (used / total) * 100.0;

        info.push(format!("Total: {:.2} GB", total));
        info.push(format!("Used: {:.2} GB ({:.1}%)", used, usage_percent));
        info.push(format!("Available: {:.2} GB", available));

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

/* 
Make sure to add Networking and DiskInfo

pub struct NetworkInfo {
    sys: System,
};

impl NetworkInfo {
    pub fn new() -> Self {

    }
} */
