pub const KILOBYTE: f64 = 1024.0;
pub const MEGABYTE: f64 = 1024.0 * 1024.0;
pub const GIGABYTE: f64 = 1024.0 * 1024.0 * 1024.0;
pub const TERABYTE: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;

pub fn format_rate(bytes: f64) -> String {
    if bytes >= GIGABYTE {
        format!("{:.1}Gb/s", bytes / GIGABYTE)
    } else if bytes >= MEGABYTE {
        format!("{:.1}Mb/s", bytes / MEGABYTE)
    } else if bytes >= KILOBYTE {
        format!("{:.1}Kb/s", bytes / KILOBYTE)
    } else {
        format!("{:.0}b/s", bytes)
    }
}

pub fn format_total(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= GIGABYTE {
        format!("{:.1}GB", b / GIGABYTE)
    } else if b >= MEGABYTE {
        format!("{:.1}MB", b / MEGABYTE)
    } else if b >= KILOBYTE {
        format!("{:.1}KB", b / KILOBYTE)
    } else {
        format!("{}B", bytes)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= TERABYTE {
        format!("{:.2} TB", b / TERABYTE)
    } else if b >= GIGABYTE {
        format!("{:.2} GB", b / GIGABYTE)
    } else if b >= MEGABYTE {
        format!("{:.2} MB", b / MEGABYTE)
    } else if b >= KILOBYTE {
        format!("{:.2} KB", b / KILOBYTE)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_axis_label(bytes: f64) -> String {
    if bytes >= GIGABYTE {
        format!("{:.1}G", bytes / GIGABYTE)
    } else if bytes >= MEGABYTE {
        format!("{:.1}M", bytes / MEGABYTE)
    } else if bytes >= KILOBYTE {
        format!("{:.1}K", bytes / KILOBYTE)
    } else if bytes > 0.0 {
        format!("{:.0}", bytes)
    } else {
        "0".to_string()
    }
}
