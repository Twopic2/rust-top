use std::collections::HashMap;
use sysinfo::System;

use crate::data::info::SystemInfo;

impl SystemInfo {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        if let Some(cores) = SystemInfo::display_cores(&mut sys) {
            assert_eq!(cores[0], "");
            for entry in &cores[1..] {
                assert!(entry.starts_with("Core "), "unexpected format: {}", entry);
                assert!(entry.contains('%'), "missing % in: {}", entry);
            }
        }
    }
}
