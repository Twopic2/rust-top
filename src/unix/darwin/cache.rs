use sysctl::Sysctl;

const KILOBYTE: usize = 1024;
const MEGABYTE: usize = 1024 * 1024;

#[cfg(target_os = "macos")]
pub struct CacheMac;

fn get_cache_size(name: &str) -> Option<usize> {
    sysctl::Ctl::new(name).ok()?.value().ok().and_then(|v| {
        match v {
            sysctl::CtlValue::U64(n) => Some(n as usize),
            sysctl::CtlValue::Int(n) => Some(n as usize),
            sysctl::CtlValue::Uint(n) => Some(n as usize),
            _ => None,
        }
    })
}

#[cfg(target_os = "macos")]
impl CacheMac {
    pub fn cache_levels() -> Vec<String> {
        let mut levels = Vec::new();

        if let Some(size) = get_cache_size("hw.perflevel0.l1icachesize") {
            levels.push(format!("P-L1i: {} KB", size / KILOBYTE));
        }
        if let Some(size) = get_cache_size("hw.perflevel0.l1dcachesize") {
            levels.push(format!("P-L1d: {} KB", size / KILOBYTE));
        }
        if let Some(size) = get_cache_size("hw.perflevel0.l2cachesize") {
            levels.push(format!("P-L2: {} MB", size / MEGABYTE));
        }
        if let Some(size) = get_cache_size("hw.perflevel1.l1icachesize") {
            levels.push(format!("E-L1i: {} KB", size / KILOBYTE));
        }
        if let Some(size) = get_cache_size("hw.perflevel1.l1dcachesize") {
            levels.push(format!("E-L1d: {} KB", size / KILOBYTE));
        }
        if let Some(size) = get_cache_size("hw.perflevel1.l2cachesize") {
            levels.push(format!("E-L2: {} MB", size / MEGABYTE));
        }

        levels
    }   
} 
