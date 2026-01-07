use sysinfo::{System, CpuRefreshKind, MemoryRefreshKind, RefreshKind};

pub struct SystemInfo {
    sys: System,
}

