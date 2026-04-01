use crate::processes::processdata::CollectProcessData;
use std::collections::BTreeMap;
use std::cmp::Ordering;
use sysinfo::System;
use ratatui::{
    Frame,
    style::{Color, Style, Modifier},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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

pub enum SortOrder {
    Descending,
}

#[derive(PartialEq)]
pub enum SearchState {
    NoSearch,
    Searching,
    FilterApplied,
    ClearSearch,
}

pub struct ProcessTable {
    pub selected_pid: u32,
    sort_column: ProcessColumn,
    sort_order: SortOrder,
    pub search_state: SearchState,
    pub search_input: String,
    collector: CollectProcessData,
    proc_table: Vec<CollectProcessData>,
    pub filtered_table: Vec<CollectProcessData>,
    sorted_proc: Vec<CollectProcessData>,
    render_area: Rect,
}

impl ProcessTable {
    pub fn new() -> Self {
        Self {
            selected_pid: 0,
            sort_column: ProcessColumn::CpuUsage,
            sort_order: SortOrder::Descending,
            search_state: SearchState::NoSearch,
            search_input: String::new(),
            collector: CollectProcessData::default(),
            proc_table: Vec::new(),
            filtered_table: Vec::new(),
            sorted_proc: Vec::new(),
            render_area: Rect::default(),
        }
    }

    pub fn refresh(&mut self, sys: &mut System) {
        self.proc_table = self.collector.process_data(sys);
        if self.is_searching() {
            self.apply_filter();
        }
    }

    pub fn is_searching(&self) -> bool {
        matches!(self.search_state, SearchState::Searching | SearchState::FilterApplied)
    }

    pub fn is_filter_input_active(&self) -> bool {
        self.search_state == SearchState::Searching
    }

    pub fn delete_table_entry(&mut self, pid: u32) {
        self.proc_table.retain(|p| p.pid != pid);
        self.filtered_table.retain(|p| p.pid != pid);
        self.sorted_proc.retain(|p| p.pid != pid);
        self.selected_pid = 0;
    }

    pub fn apply_filter(&mut self) {
        let query = self.search_input.to_lowercase();
        self.filtered_table = self.proc_table.iter()
            .filter(|p| {
                p.program.to_lowercase().starts_with(&query)
                    || p.command.to_lowercase().starts_with(&query)
            })
            .cloned()
            .collect();

        self.proc_table.clear();
    }

    pub fn get_sorted_processes(&self, processes: Vec<CollectProcessData>) -> Vec<CollectProcessData> {
        let mut indexed_processes: BTreeMap<ProcessKey, CollectProcessData> = BTreeMap::new();

        for process in processes {
            let key = ProcessKey::new(&process, &self.sort_column, &self.sort_order);
            indexed_processes.insert(key, process);
        }

        indexed_processes.into_values().collect()
    }

    fn create_filter_bar(&self, frame: &mut Frame, area: Rect) {
        let filter_width = 30u16.min(area.width);
        let filter_area = Rect {
            x: area.x + area.width.saturating_sub(filter_width),
            y: area.y,
            width: filter_width,
            height: 3,
        };
        let search_text = format!("/{}", self.search_input);
        let search_box = Paragraph::new(Line::from(search_text))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Filter")
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            )
            .style(Style::default().fg(Color::White).bold());
        frame.render_widget(search_box, filter_area);
    }

    pub fn handle_click(&mut self, col: u16, row: u16) {
        let area = self.render_area;
        if col < area.x || col >= area.x + area.width { return; }
        if row < area.y || row >= area.y + area.height { return; }

        let offset = row.saturating_sub(area.y + 2) as usize;
        if let Some(p) = self.sorted_proc.get(offset) {
            self.selected_pid = p.pid;
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.render_area = area;

        let source = if self.is_searching() {
            &self.filtered_table
        } else {
            &self.proc_table
        };

        if self.is_filter_input_active() {
            self.create_filter_bar(frame, area);
        }

        self.sorted_proc = self.get_sorted_processes(source.to_vec());

        let header = format!(
            "{:>7} {:>7} {:>7} {:>15} {}",
            ProcessColumn::display_header(ProcessColumn::Pid),
            ProcessColumn::display_header(ProcessColumn::CpuUsage),
            ProcessColumn::display_header(ProcessColumn::MemUsage),
            ProcessColumn::display_header(ProcessColumn::Program),
            ProcessColumn::display_header(ProcessColumn::Command)
        );

        let mut process_items: Vec<ListItem> = vec![
            ListItem::new(header).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ];

        let data_items: Vec<ListItem> = self.sorted_proc
            .iter()
            .map(|p| {
                let line = format!(
                    "{:>7} {:>6.1}% {:>6.1}% {:>15} {}",
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

        let process_list = List::new(process_items)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

        let list_selected = self.sorted_proc
            .iter()
            .position(|p| p.pid == self.selected_pid)
            .map(|i| i + 1);
        let mut list_state = ListState::default().with_selected(list_selected);
        frame.render_stateful_widget(process_list, area, &mut list_state);
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
