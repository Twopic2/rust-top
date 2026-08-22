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

**WSL:** No additional dependencies.

## Layout

<img width="3727" height="1918" alt="Screenshot_20260222_225826" src="https://github.com/user-attachments/assets/340abb8f-67b6-46ff-a696-c3adb0672a93" />

## Building

**Debug**
- cargo build
- cargo run

**Release**
- cargo build --release
- cargo run --release


## Benchmarking

**d-hat memory allocations**
- cargo run --release --features dhat-heap
which produces a dhat-heap.json which shows the data inside https://nnethercote.github.io/dh_view/dh_view.html. 