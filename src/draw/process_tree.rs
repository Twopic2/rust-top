use crate::processes::processdata::CollectProcessData;
use std::collections::BTreeMap;
use std::cmp::Ordering;
use ratatui::{
    Frame,
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, List, ListItem},
    layout::Rect,
};

#[derive(Clone, Copy)]
pub enum ProcessColumn {
    Pid,
    Command,
    Program,
    MemUsage,
    CpuUsage,
}

impl ProcessColumn {
    fn display_header(process: ProcessColumn) -> &'static str {
        match process {
            ProcessColumn::Pid => "PID",
            ProcessColumn::CpuUsage => "CPU",
            ProcessColumn::MemUsage => "MEM",
            ProcessColumn::Command => "Command",
            ProcessColumn::Program => "Program",
        }
    } 
}

#[derive(Clone, Copy)]
pub enum SortOrder {
    Descending,
}

pub struct ProcessTree {
    sort_column: ProcessColumn,
    sort_order: SortOrder,
}

impl ProcessTree {
    pub fn new() -> Self {
        Self {
            sort_column: ProcessColumn::CpuUsage,
            sort_order: SortOrder::Descending,
        }
    }

    pub fn get_sorted_processes(&self, processes: Vec<CollectProcessData>) -> Vec<CollectProcessData> {
        let mut indexed_processes: BTreeMap<ProcessKey, CollectProcessData> = BTreeMap::new();

        for process in processes {
            let key = ProcessKey::new(&process, &self.sort_column, &self.sort_order);
            indexed_processes.insert(key, process);
        }

        indexed_processes.into_values().collect()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, processes: Vec<CollectProcessData>) {
        let top_processes = self.get_sorted_processes(processes);

        let header = format!(
            "{:>3}  {:>7} {:>7} {:>7} {:>7} {:>7}",
            "#",
            ProcessColumn::display_header(ProcessColumn::Pid),
            ProcessColumn::display_header(ProcessColumn::CpuUsage),
            ProcessColumn::display_header(ProcessColumn::MemUsage),
            ProcessColumn::display_header(ProcessColumn::Program),
            ProcessColumn::display_header(ProcessColumn::Command)
        );

        let mut process_items: Vec<ListItem> = vec![
            ListItem::new(header).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ];

        let data_items: Vec<ListItem> = top_processes
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let line = format!(
                    "{:>3}. {:>7} {:>6.1}% {:>6.1}% {:>15} {}",
                    i + 1,
                    p.pid,
                    p.cpu_usage_percent,
                    p.mem_usage_percent,
                    p.program,
                    p.command
                );
                ListItem::new(line)
            })
            .collect();

        process_items.extend(data_items);

        let title = format!("Processes");

        let process_list = List::new(process_items)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(process_list, area);
    }
}

#[derive(Debug, Clone)]
struct ProcessKey {
    primary: OrderedFloat,
    secondary: u32,
}

impl ProcessKey {
    fn new(process: &CollectProcessData, column: &ProcessColumn, order: &SortOrder) -> Self {
        let value = match column {
            ProcessColumn::Pid => process.pid as f32,
            ProcessColumn::CpuUsage => process.cpu_usage_percent,
            ProcessColumn::MemUsage => process.mem_usage_percent,
            ProcessColumn::Command => process.command.len() as f32,
            ProcessColumn::Program => process.program.len() as f32,
        };

        let primary = OrderedFloat::new(value, order);

        Self {
            primary,
            secondary: process.pid,
        }
    }
}

impl PartialEq for ProcessKey {
    fn eq(&self, other: &Self) -> bool {
        self.primary == other.primary && self.secondary == other.secondary
    }
}

impl Eq for ProcessKey {}

impl PartialOrd for ProcessKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProcessKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.primary.cmp(&other.primary)
            .then_with(|| self.secondary.cmp(&other.secondary))
    }
}

#[derive(Debug, Clone)]
struct OrderedFloat {
    value: f32,
    inverted: bool,
}

impl OrderedFloat {
    fn new(value: f32, order: &SortOrder) -> Self {
        let inverted = matches!(order, SortOrder::Descending);
        Self { value, inverted }
    }
}

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        let a = if self.value.is_nan() { 0.0 } else { self.value };
        let b = if other.value.is_nan() { 0.0 } else { other.value };
        a == b
    }
}

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = if self.value.is_nan() { 0.0 } else { self.value };
        let b = if other.value.is_nan() { 0.0 } else { other.value };

        let ordering = a.partial_cmp(&b).unwrap_or(Ordering::Equal);

        if self.inverted {
            ordering.reverse()
        } else {
            ordering
        }
    }
}