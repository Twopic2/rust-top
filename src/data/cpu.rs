use std::{collections::HashMap, format};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use crate::tools::units::format_kb;

#[cfg(not(target_os = "macos"))]
use std::process::Command;
#[cfg(not(target_os = "macos"))]
use cache_size::{l1_cache_size, l2_cache_size};
#[cfg(not(target_os = "macos"))]
use crate::tools::units::KILOBYTE;

pub struct CpuInfo;

/* 
    ! Important Fixes

    Why is there so many useless hashmaps this could be all implamented with a Vector. 
*/

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

    pub fn display_cpu_model(sys: &mut System) -> Option<String> {
        let brand = sys.cpus().first()?.brand().to_string();
        Some(format!("Brand: {}", brand))
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
    pub fn display_cpu_cache() -> Vec<(&'static str, f64)> {
        let mut cache_info = Vec::new();

        if let Some(size) = l1_cache_size() {
            let kb = format_kb(size);
            cache_info.push(("L1", kb));
        }

        if let Some(size) = l2_cache_size() {
            let kb = format_kb(size);
            cache_info.push(("L2", kb));
        }

        cache_info
    }
}
