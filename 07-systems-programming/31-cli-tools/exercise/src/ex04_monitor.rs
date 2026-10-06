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
        todo!("Exercise 4")
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
        todo!("Exercise 4")
    }
}

impl App {
    pub fn new() -> Self {
        todo!("Exercise 4")
    }

    /// Take a new snapshot -- unless paused. Keeps `selected` in range.
    pub fn update(&mut self, snapshot: Snapshot) {
        todo!("Exercise 4")
    }

    /// `q`/Esc quit, `p` pause, `c`/`m`/`i`/`n` sort by CPU/memory/pid/name,
    /// Up/Down move the selection (within the list).
    pub fn on_key(&mut self, key: Key) {
        todo!("Exercise 4")
    }

    /// Processes in display order: CPU and memory descending, pid and name ascending.
    pub fn sorted_processes(&self) -> Vec<&ProcessInfo> {
        todo!("Exercise 4")
    }
}

/// `1536` -> `"1.5 KiB"`, `0` -> `"0 B"`.
pub fn human_bytes(bytes: u64) -> String {
    todo!("Exercise 4")
}

/// Draw the whole screen: CPU and memory gauges, the CPU history, the
/// process table (sort column marked with `▼`, selection highlighted) and a
/// help line (with `PAUSED` when paused).
pub fn render(frame: &mut Frame, app: &App) {
    todo!("Exercise 4")
}
