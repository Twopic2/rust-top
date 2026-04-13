#![cfg(not(target_os = "macos"))]

use std::collections::HashMap;
use std::process::Command;

use cache_size::{l1_cache_size, l2_cache_size};

use crate::data::info::SystemInfo;
use crate::tools::units::KILOBYTE;

impl SystemInfo {
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
    fn cache_l1_format_kb() {
        let bytes: usize = 32 * KILOBYTE as usize;
        assert_eq!(format!("{} KB", bytes / KILOBYTE as usize), "32 KB");
    }

    #[test]
    fn cache_l2_format_kb() {
        let bytes: usize = 512 * KILOBYTE as usize;
        assert_eq!(format!("{} KB", bytes / KILOBYTE as usize), "512 KB");
    }

    #[test]
    fn cache_zero_bytes_formats_as_zero_kb() {
        let bytes: usize = 0;
        assert_eq!(format!("{} KB", bytes / KILOBYTE as usize), "0 KB");
    }

    #[test]
    fn display_cpu_cache_returns_some() {
        let cache = SystemInfo::display_cpu_cache();
        assert!(cache.is_some(), "expected cache info on this machine");
        let map = cache.unwrap();
        assert!(map.contains_key("L1"), "expected L1 cache entry");
        for (key, val) in &map {
            assert!(!val.is_empty(), "{} cache value should not be empty", key);
        }
    }
}
