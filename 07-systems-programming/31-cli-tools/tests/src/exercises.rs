use std::fs;
use std::path::{Path, PathBuf};

use crate::sut::bonus_completions;
use crate::sut::ex01_grep::{self as grep, ColorChoice, GrepArgs, GrepError};
use crate::sut::ex02_logview::{
    self as logview, Level, LineFilter, LogCommand, LogviewCli, ParseError,
};
use crate::sut::ex03_tasks::{
    self as tasks, ListFilter, Priority, Store, TaskCommand, TaskError, TasksCli,
};
use crate::sut::ex04_monitor::{self as monitor, App, Key, ProcessInfo, Snapshot, SortBy};
use clap::{CommandFactory, Parser};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use regex::Regex;

fn args(line: &[&str]) -> GrepArgs {
    GrepArgs::try_parse_from(std::iter::once("minigrep").chain(line.iter().copied()))
        .expect("valid arguments")
}

/// A directory: a.txt, b.txt, sub/c.txt, .hidden/d.txt
fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "apple\nBanana\ncherry apple\n").unwrap();
    fs::write(dir.path().join("b.txt"), "date\nelderberry\n").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/c.txt"), "apple pie\n").unwrap();
    fs::create_dir(dir.path().join(".hidden")).unwrap();
    fs::write(dir.path().join(".hidden/d.txt"), "apple\n").unwrap();
    dir
}

fn run_grep(line: &[&str]) -> (i32, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = grep::run(&args(line), &mut out, &mut err, false);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

fn p(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_parses_flags() {
    let a = args(&[
        "-i", "-n", "-v", "-c", "-r", "-F", "-m", "3", "--color", "always", "pat", "x", "y",
    ]);
    assert_eq!(a.pattern, "pat");
    assert_eq!(a.paths, [PathBuf::from("x"), PathBuf::from("y")]);
    assert!(
        a.ignore_case
            && a.line_number
            && a.invert_match
            && a.count
            && a.recursive
            && a.fixed_strings
    );
    assert_eq!((a.max_count, a.color), (Some(3), ColorChoice::Always));
    let long = args(&[
        "--ignore-case",
        "--line-number",
        "--invert-match",
        "--count",
        "--recursive",
        "--fixed-strings",
        "--max-count",
        "1",
        "p",
        "f",
    ]);
    assert!(
        long.ignore_case
            && long.line_number
            && long.invert_match
            && long.count
            && long.recursive
            && long.fixed_strings
    );
    let defaults = args(&["p", "f"]);
    assert!(!defaults.ignore_case && defaults.max_count.is_none());
    assert_eq!(defaults.color, ColorChoice::Never);
    assert!(
        GrepArgs::try_parse_from(["minigrep", "pattern"]).is_err(),
        "a path is required"
    );
    assert!(GrepArgs::try_parse_from(["minigrep", "--color", "sometimes", "p", "f"]).is_err());
}

#[test]
fn ex1_build_regex() {
    let re = grep::build_regex(&args(&["a.c", "f"])).unwrap();
    assert!(re.is_match("abc"));
    let fixed = grep::build_regex(&args(&["-F", "a.c", "f"])).unwrap();
    assert!(!fixed.is_match("abc") && fixed.is_match("a.c"));
    let ci = grep::build_regex(&args(&["-i", "hello", "f"])).unwrap();
    assert!(ci.is_match("HeLLo"));
    assert!(matches!(
        grep::build_regex(&args(&["(", "f"])),
        Err(GrepError::Pattern(_))
    ));
}

#[test]
fn ex1_search_lines() {
    let re = Regex::new("an").unwrap();
    let text = "banana\napple\nmango\n";
    assert_eq!(
        grep::search_lines(text.as_bytes(), &re, false, None).unwrap(),
        [(1, "banana".to_string()), (3, "mango".to_string())]
    );
    assert_eq!(
        grep::search_lines(text.as_bytes(), &re, true, None).unwrap(),
        [(2, "apple".to_string())]
    );
    assert_eq!(
        grep::search_lines(text.as_bytes(), &re, false, Some(1))
            .unwrap()
            .len(),
        1
    );
    assert!(
        grep::search_lines(text.as_bytes(), &re, false, Some(0))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn ex1_collect_files() {
    let dir = tree();
    let d = dir.path().to_path_buf();
    assert!(matches!(
        grep::collect_files(std::slice::from_ref(&d), false),
        Err(GrepError::IsADirectory(_))
    ));
    let files = grep::collect_files(std::slice::from_ref(&d), true).unwrap();
    assert_eq!(
        files,
        [d.join("a.txt"), d.join("b.txt"), d.join("sub/c.txt")],
        "sorted, hidden skipped"
    );
    let files = grep::collect_files(&[d.join("b.txt"), d.join("a.txt")], false).unwrap();
    assert_eq!(files, [d.join("b.txt"), d.join("a.txt")], "files as given");
}

#[test]
fn ex1_highlight() {
    let re = Regex::new("ap+").unwrap();
    assert_eq!(
        grep::highlight("an apple, a map", &re),
        "an \x1b[1;31mapp\x1b[0mle, a m\x1b[1;31map\x1b[0m"
    );
}

#[test]
fn ex1_run_output_and_exit_codes() {
    let dir = tree();
    let a = p(&dir.path().join("a.txt"));
    assert_eq!(
        run_grep(&["apple", &a]),
        (0, "apple\ncherry apple\n".into(), String::new())
    );
    assert_eq!(run_grep(&["-n", "-i", "banana", &a]).1, "2:Banana\n");
    assert_eq!(run_grep(&["-c", "apple", &a]).1, "2\n");
    assert_eq!(run_grep(&["zebra", &a]), (1, String::new(), String::new()));

    let (code, out, _) = run_grep(&["-r", "apple", &p(dir.path())]);
    assert_eq!(code, 0);
    let expected = format!(
        "{a}:apple\n{a}:cherry apple\n{}:apple pie\n",
        p(&dir.path().join("sub/c.txt"))
    );
    assert_eq!(out, expected, "file names when searching several");

    let (code, out, err) = run_grep(&["apple", &a, &p(&dir.path().join("missing.txt"))]);
    assert_eq!(code, 2, "an error wins over a match");
    assert!(out.contains("apple") && err.contains("missing.txt"));
    let (code, _, err) = run_grep(&["x", &p(dir.path())]);
    assert_eq!(code, 2);
    assert!(err.starts_with("minigrep:") && err.contains("is a directory"));
}

#[test]
fn ex1_color() {
    let dir = tree();
    let a = p(&dir.path().join("a.txt"));
    assert!(
        run_grep(&["--color", "always", "apple", &a])
            .1
            .contains("\x1b[1;31mapple\x1b[0m")
    );
    assert!(
        !run_grep(&["--color", "auto", "apple", &a])
            .1
            .contains('\x1b'),
        "not a terminal"
    );
    let (mut out, mut err) = (Vec::new(), Vec::new());
    grep::run(
        &args(&["--color", "auto", "apple", &a]),
        &mut out,
        &mut err,
        true,
    );
    assert!(String::from_utf8(out).unwrap().contains('\x1b'));
}

// ---------------------------------------------------------------- Exercise 2

const LOG: &str = "\
2026-10-05T12:00:00Z INFO  app::http: listening on :8080
2026-10-05T12:00:03Z DEBUG app::db::pool: connection opened
2026-10-05T12:00:04Z WARN  app::db: slow query took 812 ms
not a log line
2026-10-05T12:00:09Z ERROR app::http: request failed: timeout
2026-10-05T12:01:00Z INFO  app::dbx: GET /health 200
";

fn line(ts: &str, level: Level, target: &str, message: &str) -> logview::LogLine {
    logview::LogLine {
        timestamp: ts.into(),
        level,
        target: target.into(),
        message: message.into(),
    }
}

#[test]
fn ex2_levels() {
    assert_eq!("warn".parse::<Level>(), Ok(Level::Warn));
    assert_eq!("WARNING".parse::<Level>(), Ok(Level::Warn));
    assert_eq!("Error".parse::<Level>(), Ok(Level::Error));
    assert_eq!(
        "fatal".parse::<Level>(),
        Err(ParseError::Level("fatal".into()))
    );
    assert!(Level::Trace < Level::Debug && Level::Warn < Level::Error);
    assert_eq!(Level::Info.to_string(), "INFO");
    assert_eq!(format!("[{:<5}]", Level::Info), "[INFO ]");
}

#[test]
fn ex2_parse_line() {
    assert_eq!(
        logview::parse_line("2026-10-05T12:00:04Z WARN  app::db: slow: 812 ms"),
        Ok(line(
            "2026-10-05T12:00:04Z",
            Level::Warn,
            "app::db",
            "slow: 812 ms"
        ))
    );
    assert!(logview::is_timestamp("2026-10-05T12:00:04Z"));
    assert!(!logview::is_timestamp("2026-10-05 12:00:04"));
    assert!(matches!(
        logview::parse_line("yesterday INFO a: b"),
        Err(ParseError::Timestamp(_))
    ));
    assert!(matches!(
        logview::parse_line("2026-10-05T12:00:04Z LOUD a: b"),
        Err(ParseError::Level(_))
    ));
    assert_eq!(
        logview::parse_line("2026-10-05T12:00:04Z INFO no colon here"),
        Err(ParseError::Missing)
    );
    assert_eq!(logview::parse_line(""), Err(ParseError::Missing));
}

#[test]
fn ex2_filters() {
    let l = line(
        "2026-10-05T12:00:04Z",
        Level::Warn,
        "app::db::pool",
        "Slow Query",
    );
    assert!(LineFilter::default().matches(&l));
    assert!(
        LineFilter {
            level: Some(Level::Warn),
            ..Default::default()
        }
        .matches(&l)
    );
    assert!(
        !LineFilter {
            level: Some(Level::Error),
            ..Default::default()
        }
        .matches(&l)
    );
    assert!(
        LineFilter {
            target: Some("app::db".into()),
            ..Default::default()
        }
        .matches(&l)
    );
    assert!(
        LineFilter {
            target: Some("app::db::pool".into()),
            ..Default::default()
        }
        .matches(&l)
    );
    assert!(
        !LineFilter {
            target: Some("app::d".into()),
            ..Default::default()
        }
        .matches(&l),
        "whole path segments"
    );
    assert!(
        LineFilter {
            since: Some("2026-10-05T12:00:04Z".into()),
            ..Default::default()
        }
        .matches(&l),
        "since is inclusive"
    );
    assert!(
        !LineFilter {
            until: Some("2026-10-05T12:00:04Z".into()),
            ..Default::default()
        }
        .matches(&l),
        "until is exclusive"
    );
    assert!(
        LineFilter {
            contains: Some("slow query".into()),
            ..Default::default()
        }
        .matches(&l),
        "case-insensitive"
    );
    assert!(
        !LineFilter {
            contains: Some("fast".into()),
            ..Default::default()
        }
        .matches(&l)
    );
}

#[test]
fn ex2_stats() {
    let s = logview::stats(LOG.lines());
    assert_eq!(s.by_level.get(&Level::Info), Some(&2));
    assert_eq!(s.by_level.get(&Level::Error), Some(&1));
    assert_eq!(s.by_target.get("app::http"), Some(&2));
    assert_eq!(s.malformed, 1);
    assert_eq!(
        (s.first.as_deref(), s.last.as_deref()),
        (Some("2026-10-05T12:00:00Z"), Some("2026-10-05T12:01:00Z"))
    );
    assert_eq!(
        logview::stats(["", "  "]).malformed,
        0,
        "blank lines aren't malformed"
    );
}

fn logview(cli: &[&str]) -> String {
    let cli = LogviewCli::try_parse_from(std::iter::once("logview").chain(cli.iter().copied()))
        .expect("valid arguments");
    let mut out = Vec::new();
    logview::run(&cli.command, LOG.as_bytes(), &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn ex2_cli() {
    let cli = LogviewCli::try_parse_from([
        "logview", "filter", "-l", "warn", "-t", "app", "--since", "a", "--until", "b", "-g", "x",
        "--json", "f.log",
    ])
    .unwrap();
    let LogCommand::Filter { filter, json, file } = cli.command else {
        panic!("filter")
    };
    assert_eq!(filter.level, Some(Level::Warn));
    assert_eq!(
        (filter.target.as_deref(), filter.contains.as_deref(), json),
        (Some("app"), Some("x"), true)
    );
    assert_eq!(file, Some(PathBuf::from("f.log")));
    let cli = LogviewCli::try_parse_from(["logview", "tail", "-n", "5"]).unwrap();
    assert_eq!(
        cli.command,
        LogCommand::Tail {
            lines: 5,
            level: None,
            file: None
        }
    );
    assert!(LogviewCli::try_parse_from(["logview", "frobnicate"]).is_err());
}

#[test]
fn ex2_run() {
    assert_eq!(logview(&["filter", "--level", "warn"]).lines().count(), 2);
    let json = logview(&["filter", "--target", "app::db", "--json"]);
    let values: Vec<serde_json::Value> = json
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(values.len(), 2, "app::db and app::db::pool, not app::dbx");
    assert_eq!(values[1]["level"], "WARN");
    assert_eq!(values[1]["message"], "slow query took 812 ms");
    let stats = logview(&["stats"]);
    assert!(
        stats.contains("ERROR 1") && stats.contains("malformed: 1"),
        "{stats}"
    );
    assert_eq!(
        logview(&["tail", "-n", "2"]).lines().collect::<Vec<_>>(),
        [LOG.lines().nth(4).unwrap(), LOG.lines().nth(5).unwrap()]
    );
    assert_eq!(
        logview(&["tail", "-n", "1", "--level", "warn"]).trim(),
        LOG.lines().nth(4).unwrap()
    );
}

// ---------------------------------------------------------------- Exercise 3

fn store_in(dir: &tempfile::TempDir) -> Store {
    Store::load(dir.path().join("tasks.json")).unwrap()
}

#[test]
fn ex3_dates() {
    assert!(tasks::valid_date("2026-10-31"));
    for bad in [
        "2026-13-01",
        "2026-00-10",
        "2026-10-32",
        "26-10-31",
        "2026/10/31",
        "abcd-ef-gh",
        "",
    ] {
        assert!(!tasks::valid_date(bad), "{bad}");
    }
}

#[test]
fn ex3_add_list_complete_remove() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = store_in(&dir);
    assert!(store.tasks().is_empty(), "a missing file is an empty store");
    let a = store
        .add("write docs", Priority::Low, vec![], None)
        .unwrap();
    let b = store
        .add(
            "  fix bug  ",
            Priority::High,
            vec!["bug".into()],
            Some("2026-10-31".into()),
        )
        .unwrap();
    let c = store
        .add("review", Priority::Medium, vec!["work".into()], None)
        .unwrap();
    assert_eq!((a, b, c), (1, 2, 3));
    assert_eq!(store.tasks()[1].title, "fix bug", "trimmed");
    store.complete(b).unwrap();
    let open: Vec<u32> = store
        .list(&ListFilter::default())
        .iter()
        .map(|t| t.id)
        .collect();
    assert_eq!(open, [c, a], "open only, by priority");
    let all: Vec<u32> = store
        .list(&ListFilter {
            include_done: true,
            ..Default::default()
        })
        .iter()
        .map(|t| t.id)
        .collect();
    assert_eq!(all, [c, a, b], "done last");
    let tagged = store.list(&ListFilter {
        include_done: true,
        tag: Some("work".into()),
        ..Default::default()
    });
    assert_eq!(tagged.len(), 1);
    let high = store.list(&ListFilter {
        include_done: true,
        priority: Some(Priority::High),
        ..Default::default()
    });
    assert_eq!(high[0].id, b);

    assert_eq!(store.remove(a).unwrap().title, "write docs");
    assert!(matches!(store.remove(a), Err(TaskError::NotFound(1))));
    assert_eq!(
        store.add("new", Priority::Low, vec![], None).unwrap(),
        4,
        "ids are never reused"
    );
}

#[test]
fn ex3_validation_and_edit() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = store_in(&dir);
    assert!(matches!(
        store.add("  ", Priority::Low, vec![], None),
        Err(TaskError::EmptyTitle)
    ));
    assert!(matches!(
        store.add("t", Priority::Low, vec![], Some("tomorrow".into())),
        Err(TaskError::InvalidDate(_))
    ));
    let id = store.add("old", Priority::Low, vec![], None).unwrap();
    store.edit(id, Some("new"), Some(Priority::High)).unwrap();
    assert_eq!(
        (store.tasks()[0].title.as_str(), store.tasks()[0].priority),
        ("new", Priority::High)
    );
    assert!(matches!(
        store.edit(id, Some(""), None),
        Err(TaskError::EmptyTitle)
    ));
    assert!(matches!(
        store.edit(99, None, None),
        Err(TaskError::NotFound(99))
    ));
    assert!(matches!(store.complete(99), Err(TaskError::NotFound(99))));
}

#[test]
fn ex3_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = store_in(&dir);
    store
        .add(
            "persist me",
            Priority::High,
            vec!["x".into()],
            Some("2026-01-02".into()),
        )
        .unwrap();
    store.save().unwrap();
    let again = store_in(&dir);
    assert_eq!(again.tasks(), store.tasks());
    let names: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, ["tasks.json"], "no temporary files left behind");
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("tasks.json")).unwrap()).unwrap();
    assert_eq!(json["tasks"][0]["priority"], "high");
    fs::write(dir.path().join("tasks.json"), "{ not json").unwrap();
    assert!(matches!(
        Store::load(dir.path().join("tasks.json")),
        Err(TaskError::Json(_))
    ));
}

#[test]
fn ex3_cli() {
    let cli = TasksCli::try_parse_from([
        "tasks",
        "--file",
        "t.json",
        "add",
        "Buy milk",
        "-p",
        "high",
        "-t",
        "home",
        "-t",
        "today",
        "--due",
        "2026-10-06",
    ])
    .unwrap();
    assert_eq!(cli.file, PathBuf::from("t.json"));
    assert_eq!(
        cli.command,
        TaskCommand::Add {
            title: "Buy milk".into(),
            priority: Priority::High,
            tags: vec!["home".into(), "today".into()],
            due: Some("2026-10-06".into())
        }
    );
    let cli = TasksCli::try_parse_from(["tasks", "--file", "t.json", "list", "--all"]).unwrap();
    assert_eq!(
        cli.command,
        TaskCommand::List {
            all: true,
            tag: None,
            priority: None
        }
    );
    let command = TasksCli::command();
    let file = command
        .get_arguments()
        .find(|a| a.get_id() == "file")
        .expect("a --file argument");
    assert_eq!(
        file.get_env().and_then(|e| e.to_str()),
        Some("TASKS_FILE"),
        "--file falls back to $TASKS_FILE"
    );
    assert_eq!(file.get_default_values(), ["tasks.json"]);
}

#[test]
fn ex3_run_and_table() {
    let dir = tempfile::tempdir().unwrap();
    let file = p(&dir.path().join("t.json"));
    let run = |line: &[&str]| -> (Result<(), TaskError>, String) {
        let cli = TasksCli::try_parse_from(
            ["tasks", "--file", &file]
                .into_iter()
                .chain(line.iter().copied()),
        )
        .unwrap();
        let mut out = Vec::new();
        let result = tasks::run(&cli, &mut out);
        (result, String::from_utf8(out).unwrap())
    };
    assert_eq!(
        run(&[
            "add",
            "Fix the parser",
            "-p",
            "high",
            "-t",
            "bug",
            "--due",
            "2026-10-31"
        ])
        .1,
        "added task 1\n"
    );
    run(&["add", "Review"]).0.unwrap();
    run(&["done", "2"]).0.unwrap();
    assert_eq!(
        run(&["list"]).1,
        "  1 [ ] high   Fix the parser  #bug  (due 2026-10-31)\n"
    );
    assert_eq!(
        run(&["list", "--all"]).1.lines().nth(1),
        Some("  2 [x] medium Review")
    );
    assert!(matches!(
        run(&["remove", "7"]).0,
        Err(TaskError::NotFound(7))
    ));
}

// ---------------------------------------------------------------- Exercise 4

fn process(pid: u32, name: &str, cpu: f32, memory: u64) -> ProcessInfo {
    ProcessInfo {
        pid,
        name: name.into(),
        cpu,
        memory,
    }
}

fn snapshot() -> Snapshot {
    Snapshot {
        cpus: vec![10.0, 30.0],
        memory_used: 6 * 1024 * 1024 * 1024,
        memory_total: 16 * 1024 * 1024 * 1024,
        processes: vec![
            process(30, "zsh", 0.5, 4_000_000),
            process(10, "cargo", 80.0, 900_000_000),
            process(20, "Browser", 12.5, 2_000_000_000),
        ],
    }
}

fn screen(app: &App) -> String {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|f| monitor::render(f, app)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn ex4_helpers() {
    assert_eq!(snapshot().cpu_average(), 20.0);
    assert_eq!(Snapshot::default().cpu_average(), 0.0);
    assert_eq!(monitor::human_bytes(0), "0 B");
    assert_eq!(monitor::human_bytes(1023), "1023 B");
    assert_eq!(monitor::human_bytes(1536), "1.5 KiB");
    assert_eq!(monitor::human_bytes(6 * 1024 * 1024 * 1024), "6.0 GiB");
}

#[test]
fn ex4_update_pause_and_history() {
    let mut app = App::new();
    assert_eq!(
        (app.sort, app.selected, app.paused, app.quit),
        (SortBy::Cpu, 0, false, false)
    );
    app.update(snapshot());
    assert_eq!(app.history, [20]);
    app.on_key(Key::Char('p'));
    let mut changed = snapshot();
    changed.cpus = vec![90.0];
    app.update(changed.clone());
    assert_eq!(app.snapshot, snapshot(), "paused: no updates");
    app.on_key(Key::Char('p'));
    for _ in 0..100 {
        app.update(changed.clone());
    }
    assert_eq!(app.history.len(), monitor::HISTORY);
    assert_eq!(app.history.back(), Some(&90));
}

#[test]
fn ex4_keys_and_sorting() {
    let mut app = App::new();
    app.update(snapshot());
    let pids = |app: &App| {
        app.sorted_processes()
            .iter()
            .map(|p| p.pid)
            .collect::<Vec<_>>()
    };
    assert_eq!(pids(&app), [10, 20, 30], "CPU descending");
    app.on_key(Key::Char('m'));
    assert_eq!(pids(&app), [20, 10, 30], "memory descending");
    app.on_key(Key::Char('i'));
    assert_eq!(pids(&app), [10, 20, 30]);
    app.on_key(Key::Char('n'));
    assert_eq!(pids(&app), [20, 10, 30], "names, case-insensitive");
    app.on_key(Key::Up);
    assert_eq!(app.selected, 0);
    for _ in 0..5 {
        app.on_key(Key::Down);
    }
    assert_eq!(app.selected, 2, "stays within the list");
    let mut fewer = snapshot();
    fewer.processes.truncate(1);
    app.update(fewer);
    assert_eq!(app.selected, 0, "clamped when the list shrinks");
    app.on_key(Key::Char('x'));
    assert!(!app.quit);
    app.on_key(Key::Char('q'));
    assert!(app.quit);
    let mut esc = App::new();
    esc.on_key(Key::Esc);
    assert!(esc.quit);
}

#[test]
fn ex4_render() {
    let mut app = App::new();
    app.update(snapshot());
    let text = screen(&app);
    assert!(
        text.contains("CPU (2 cores)") && text.contains("20.0%"),
        "{text}"
    );
    assert!(
        text.contains("Memory") && text.contains("6.0 GiB / 16.0 GiB"),
        "{text}"
    );
    assert!(
        text.contains("CPU history") && text.contains("Processes (3)"),
        "{text}"
    );
    assert!(text.contains("CPU%▼"), "the sort column is marked: {text}");
    let (cargo, browser, zsh) = (
        text.find("cargo").unwrap(),
        text.find("Browser").unwrap(),
        text.find("zsh").unwrap(),
    );
    assert!(cargo < browser && browser < zsh, "rows in sort order");
    assert!(!text.contains("PAUSED"));
    app.on_key(Key::Char('p'));
    app.on_key(Key::Char('m'));
    let text = screen(&app);
    assert!(text.contains("PAUSED") && text.contains("MEM▼"), "{text}");
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_completions_for_every_shell() {
    for shell in [
        clap_complete::Shell::Bash,
        clap_complete::Shell::Zsh,
        clap_complete::Shell::Fish,
    ] {
        let script = bonus_completions::completions(shell);
        assert!(script.contains("minigrep"), "{shell}");
        assert!(script.contains("ignore-case"), "{shell}: knows the flags");
    }
}

#[test]
fn bonus_should_color() {
    use ColorChoice::*;
    assert!(bonus_completions::should_color(Always, false, Some("1")));
    assert!(!bonus_completions::should_color(Never, true, None));
    assert!(bonus_completions::should_color(Auto, true, None));
    assert!(!bonus_completions::should_color(Auto, false, None));
    assert!(!bonus_completions::should_color(Auto, true, Some("1")));
    assert!(
        bonus_completions::should_color(Auto, true, Some("")),
        "an empty NO_COLOR doesn't count"
    );
}
