# rust-top

A terminal-based system monitor written in Rust. Displays real-time CPU, memory, disk, network, temperature, and process information in a graphical TUI.

## Features

- **CPU** — per-core usage graphs, total usage bar, model/cache info
- **Memory** — RAM and swap usage
- **Temperatures** — CPU, NVMe, and NIC sensors (Linux: lm-sensors, macOS: sysinfo)
- **Disk** — usage, filesystem, mount point, and live I/O rates
- **Network** — RX/TX histogram with total throughput and local IP
- **Processes** — live process list sorted by CPU usage with memory %
- Configurable refresh rate (1000–5000ms)

## Platform Support

| WSL | Linux | macOS |
|---------|-------|-------|
| CPU / Memory | ✓ | ✓ |
| Temperatures | ✓ (lm-sensors) | ✓ (sysinfo) |
| CPU Cache Info | ✓ (cache-size) | ✓ (sysctl P/E cores) |
| Disk I/O | ✓ | ✓ |
| Network | ✓ | ✓ |
| Processes | ✓ | ✓ |

## Requirements

**Linux:**
- [`lm-sensors`](https://github.com/groeck/lm-sensors) installed and configured (`sensors-detect`)

```sh
# Debian/Ubuntu
sudo apt install lm-sensors
sudo sensors-detect

# Arch
sudo pacman -S lm_sensors
sudo sensors-detect
```

**macOS:** No additional dependencies.

## Build & Run

```sh
git clone https://github.com/yourname/rust-top
cd rust-top
cargo run --release
```

## Keybindings

| Key | Action |
|-----|--------|
| `Q` / `Esc` | Quit |
| `+` / `=` | Increase refresh rate (faster) |
| `-` / `_` | Decrease refresh rate (slower) |
| Mouse click | Adjust refresh rate via on-screen button |

## Layout

```
┌─────────────────────────────────────────────────────────────────┐
│                      rust-top  [hostname]                       │
├────────────────────────────┬────────────────────────────────────┤
│ CPU Model │ Cache │ Memory │ Disk usage / I/O table             │
├────────────────────────────┤                                    │
│ Per-core CPU graphs        ├────────────────────────────────────┤
├────────────────────────────┤ Process list (sorted by CPU%)      │
│ Total CPU bar              │                                    │
├────────────────────────────│                                    │
│ Temperatures │ Temp bars   │                                    │
├────────────────────────────│                                    │
│ Network RX/TX histogram    │                                    │
└────────────────────────────┴────────────────────────────────────┘
```

## Project Structure

```
src/
├── main.rs                  # Entry point, terminal init
├── app.rs                   # App state, main loop, layout
├── event.rs                 # Input handling
├── data/
│   ├── info.rs              # CPU, memory, kernel info
│   ├── temp.rs              # Temperature sensors (platform-specific)
│   ├── disk.rs              # Disk usage and I/O
│   ├── network.rs           # Network RX/TX
│   └── darwin/cache.rs      # macOS CPU cache via sysctl
└── draw/
    ├── bar.rs               # CPU and temperature bars
    ├── graph.rs             # Multi-core graphs, disk table
    ├── histogram.rs         # Network activity chart
    ├── widget.rs            # Temperature sensor widget
    ├── misc.rs              # Tick rate button
    └── process_tree.rs      # Process list renderer
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI framework |
| `crossterm` | Terminal input/output |
| `sysinfo` | CPU, memory, process, component data |
| `lm-sensors` | Linux hardware temperature sensors |
| `sysctl` | macOS sysctl calls (cache info) |
| `cache-size` | Linux CPU cache sizes |
| `libc` | Unix system calls |
| `local-ip-address` | Local network IP |
| `color-eyre` | Error reporting |
