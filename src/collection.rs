use sysinfo::System;

use crate::data::cpu::CpuInfo;
use crate::data::disk::DiskData;
use crate::data::power::CpuPackageCap;
use crate::data::voltage::Voltage;
use crate::draw::widgets::cpu_bar::{BarColorScheme, TotalCoreBar};
use crate::draw::widgets::cpu_graph::{ColorScheme, MultiCoreGraph};
use crate::draw::widgets::cpu_power::CpuPowerUsage;
use crate::draw::widgets::network_graph::NetworkGraph;
use crate::draw::widgets::process_table::ProcessTable;
use crate::draw::widgets::voltage_bar::VoltageUsage;

pub struct TopCollection {
    pub sys: System,
    pub core_graph: MultiCoreGraph,
    pub total_cpu_bar: TotalCoreBar,
    pub cpu_power: CpuPowerUsage,
    cpu_package: Option<CpuPackageCap>,
    pub network_histogram: NetworkGraph,
    pub disk_data: DiskData,
    pub process_tree: ProcessTable,
    pub voltage_bar: VoltageUsage,
}

impl TopCollection {
    pub fn new() -> Self {
        let mut sys = System::new_all();

        let num_cores = CpuInfo::num_cores(&mut sys);
        let core_graph = MultiCoreGraph::new(num_cores, ColorScheme::Cyan);
        let total_cpu_bar = TotalCoreBar::new(BarColorScheme::Green);
        let network_histogram = NetworkGraph::new(60);
        let disk_data = DiskData::default();
        let process_tree = ProcessTable::new();
        let cpu_power = CpuPowerUsage::default();
        let cpu_package = CpuPackageCap::new().ok();
        let voltages = lm_sensors::Initializer::default()
            .initialize()
            .map(Voltage::new)
            .unwrap();
        let voltage_bar = VoltageUsage::new(voltages);

        Self {
            sys,
            core_graph,
            total_cpu_bar,
            cpu_power,
            cpu_package,
            network_histogram,
            disk_data,
            process_tree,
            voltage_bar,
        }
    }

    pub fn refresh_timer(&mut self) {
        CpuInfo::set_refresh_timer(&mut self.sys);
    }

    pub fn update_data(&mut self) {
        let core_usages = CpuInfo::get_core_usages(&mut self.sys);

        let cpu_frequency_mhz = CpuInfo::display_cpu_frequency(&mut self.sys).unwrap_or(0);
        self.core_graph.update(&core_usages, cpu_frequency_mhz);

        self.total_cpu_bar.update(&core_usages);

        let watts = if let Some(package) = self.cpu_package.as_mut() {
            package.refresh().ok().map(|()| package.curr_watt)
        } else {
            self.cpu_package = CpuPackageCap::new().ok();
            None
        };
        self.cpu_power.update(watts);

        self.network_histogram.update();

        self.disk_data.refresh(&mut self.sys);
        self.disk_data.collect_all(&mut self.sys);

        self.voltage_bar.voltages.refresh();

        self.process_tree.refresh(&mut self.sys);
    }
}
