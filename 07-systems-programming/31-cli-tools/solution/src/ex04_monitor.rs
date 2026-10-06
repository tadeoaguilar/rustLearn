//! Exercise 4: a terminal system monitor with ratatui.
//!
//! A TUI is three parts: **state** (`App`), **input** that changes it
//! (`on_key`), and a pure **render** function from state to a frame. Keeping
//! them apart makes the whole thing testable without a terminal: tests feed
//! `App` fake snapshots and keys, render into ratatui's `TestBackend`, and
//! read the resulting buffer. `main.rs` adds the real terminal and the real
//! system data (sysinfo).

use std::collections::VecDeque;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table};

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    /// Percent of one core.
    pub cpu: f32,
    /// Bytes.
    pub memory: u64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Snapshot {
    /// Usage per core, percent.
    pub cpus: Vec<f32>,
    pub memory_used: u64,
    pub memory_total: u64,
    pub processes: Vec<ProcessInfo>,
}

impl Snapshot {
    /// Average over all cores (0 with none).
    pub fn cpu_average(&self) -> f32 {
        if self.cpus.is_empty() {
            0.0
        } else {
            self.cpus.iter().sum::<f32>() / self.cpus.len() as f32
        }
    }
}

/// Where snapshots come from: the real system, or a fake in tests.
pub trait Source {
    fn snapshot(&mut self) -> Snapshot;
}

/// The real system, through the sysinfo crate (provided).
pub struct SysinfoSource {
    system: sysinfo::System,
}

impl Default for SysinfoSource {
    fn default() -> Self {
        Self::new()
    }
}

impl SysinfoSource {
    pub fn new() -> Self {
        SysinfoSource {
            system: sysinfo::System::new_all(),
        }
    }
}

impl Source for SysinfoSource {
    fn snapshot(&mut self) -> Snapshot {
        let s = &mut self.system;
        s.refresh_cpu_usage();
        s.refresh_memory();
        s.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        Snapshot {
            cpus: s.cpus().iter().map(|c| c.cpu_usage()).collect(),
            memory_used: s.used_memory(),
            memory_total: s.total_memory(),
            processes: s
                .processes()
                .iter()
                .map(|(pid, p)| ProcessInfo {
                    pid: pid.as_u32(),
                    name: p.name().to_string_lossy().into_owned(),
                    cpu: p.cpu_usage(),
                    memory: p.memory(),
                })
                .collect(),
        }
    }
}

/// The keys the monitor understands (independent of the terminal library).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Esc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

/// How many CPU averages the sparkline remembers.
pub const HISTORY: usize = 60;

#[derive(Debug, Clone)]
pub struct App {
    pub snapshot: Snapshot,
    pub sort: SortBy,
    /// Index into the sorted process list.
    pub selected: usize,
    pub paused: bool,
    pub quit: bool,
    /// Recent CPU averages, oldest first, at most `HISTORY`.
    pub history: VecDeque<u64>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            snapshot: Snapshot::default(),
            sort: SortBy::Cpu,
            selected: 0,
            paused: false,
            quit: false,
            history: VecDeque::new(),
        }
    }

    /// Take a new snapshot -- unless paused. Keeps `selected` in range.
    pub fn update(&mut self, snapshot: Snapshot) {
        if self.paused {
            return;
        }
        if self.history.len() == HISTORY {
            self.history.pop_front();
        }
        self.history
            .push_back(snapshot.cpu_average().round() as u64);
        self.snapshot = snapshot;
        self.selected = self
            .selected
            .min(self.snapshot.processes.len().saturating_sub(1));
    }

    /// `q`/Esc quit, `p` pause, `c`/`m`/`i`/`n` sort by CPU/memory/pid/name,
    /// Up/Down move the selection (within the list).
    pub fn on_key(&mut self, key: Key) {
        match key {
            Key::Char('q') | Key::Esc => self.quit = true,
            Key::Char('p') => self.paused = !self.paused,
            Key::Char('c') => self.sort = SortBy::Cpu,
            Key::Char('m') => self.sort = SortBy::Memory,
            Key::Char('i') => self.sort = SortBy::Pid,
            Key::Char('n') => self.sort = SortBy::Name,
            Key::Up => self.selected = self.selected.saturating_sub(1),
            Key::Down => {
                let last = self.snapshot.processes.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(last);
            }
            Key::Char(_) => {}
        }
    }

    /// Processes in display order: CPU and memory descending, pid and name ascending.
    pub fn sorted_processes(&self) -> Vec<&ProcessInfo> {
        let mut list: Vec<&ProcessInfo> = self.snapshot.processes.iter().collect();
        match self.sort {
            SortBy::Cpu => list.sort_by(|a, b| b.cpu.total_cmp(&a.cpu).then(a.pid.cmp(&b.pid))),
            SortBy::Memory => list.sort_by(|a, b| b.memory.cmp(&a.memory).then(a.pid.cmp(&b.pid))),
            SortBy::Pid => list.sort_by_key(|p| p.pid),
            SortBy::Name => list.sort_by(|a, b| {
                a.name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then(a.pid.cmp(&b.pid))
            }),
        }
        list
    }
}

/// `1536` -> `"1.5 KiB"`, `0` -> `"0 B"`.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Draw the whole screen: CPU and memory gauges, the CPU history, the
/// process table (sort column marked with `▼`, selection highlighted) and a
/// help line (with `PAUSED` when paused).
pub fn render(frame: &mut Frame, app: &App) {
    let [cpu_area, mem_area, history_area, table_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let cpu = app.snapshot.cpu_average();
    frame.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("CPU ({} cores)", app.snapshot.cpus.len())),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .percent(cpu.clamp(0.0, 100.0) as u16)
            .label(format!("{cpu:.1}%")),
        cpu_area,
    );

    let (used, total) = (app.snapshot.memory_used, app.snapshot.memory_total);
    let ratio = if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64).clamp(0.0, 1.0)
    };
    frame.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Memory"))
            .gauge_style(Style::default().fg(Color::Blue))
            .ratio(ratio)
            .label(format!("{} / {}", human_bytes(used), human_bytes(total))),
        mem_area,
    );

    let history: Vec<u64> = app.history.iter().copied().collect();
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("CPU history"))
            .data(&history)
            .max(100),
        history_area,
    );

    let mark = |label: &str, by: SortBy| {
        if app.sort == by {
            format!("{label}▼")
        } else {
            label.to_string()
        }
    };
    let header = Row::new(vec![
        Cell::from(mark("PID", SortBy::Pid)),
        Cell::from(mark("NAME", SortBy::Name)),
        Cell::from(mark("CPU%", SortBy::Cpu)),
        Cell::from(mark("MEM", SortBy::Memory)),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));
    let rows = app
        .sorted_processes()
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            let row = Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(format!("{:.1}", p.cpu)),
                Cell::from(human_bytes(p.memory)),
            ]);
            if i == app.selected {
                row.style(Style::default().add_modifier(Modifier::REVERSED))
            } else {
                row
            }
        });
    let widths = [
        Constraint::Length(8),
        Constraint::Min(10),
        Constraint::Length(7),
        Constraint::Length(10),
    ];
    frame.render_widget(
        Table::new(rows, widths).header(header).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Processes ({})", app.snapshot.processes.len())),
        ),
        table_area,
    );

    let mut help = "q quit  p pause  c/m/i/n sort  ↑/↓ select".to_string();
    if app.paused {
        help.push_str("  PAUSED");
    }
    frame.render_widget(Paragraph::new(Line::from(help)), help_area);
}
