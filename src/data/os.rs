use sysinfo::System;

pub struct OsInfo;

impl OsInfo {
    pub fn display_kernel() -> Option<String> {
        System::kernel_version()
    }

    pub fn display_host_name() -> Option<String> {
        System::host_name()
    }
}
