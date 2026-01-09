use std::{collections::HashMap, thread, time::Duration};

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

    pub fn display_cores(&mut self) -> Option<Vec<String>> {
        if self.sys.cpus().is_empty() {
            return None;
        }

        let mut cores = Vec::new();

        cores.push(format!("Global CPU: {:.1}%", self.sys.global_cpu_usage()));
        cores.push(String::new());

        for (i, cpu) in self.sys.cpus().iter().enumerate() {
            cores.push(format!(
                "Core {}: {:.1}% @ {} MHz",
                i,
                cpu.cpu_usage(),
                cpu.frequency()
            ));
        }

        Some(cores)
    }

    pub fn display_cpu_model(&self) -> Option<HashMap<String, String>> {
        if self.sys.cpus().is_empty() {
            return None;
        }
        let mut info = HashMap::new();

        if let Some(cpu) = self.sys.cpus().first() {
            info.insert("CPU Name".to_string(), cpu.name().to_string());
            info.insert("Brand".to_string(), cpu.brand().to_string());
            info.insert("Vendor ID".to_string(), cpu.vendor_id().to_string());
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

    /* pub fn set_refresh_timer(&mut self) {} */
}
