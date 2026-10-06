// 31-cli-tools -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m31-cli-tools -- <1-4|bonus|all>          scripted demos
//     cargo run -p m31-cli-tools -- grep -n fn src/           the real tools
//     cargo run -p m31-cli-tools -- logview stats app.log
//     cargo run -p m31-cli-tools -- tasks add "Write tests"
//     cargo run -p m31-cli-tools -- monitor                   (interactive; q quits)

use std::io::{self, BufReader, IsTerminal, Write};
use std::time::Duration;

use clap::Parser;
use m31_cli_tools::bonus_completions;
use m31_cli_tools::ex01_grep::{self, GrepArgs};
use m31_cli_tools::ex02_logview::{self, LogviewCli};
use m31_cli_tools::ex03_tasks::{self, TasksCli};
use m31_cli_tools::ex04_monitor::{self, App, Key, Source, SysinfoSource};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

const SAMPLE_LOG: &str = "\
2026-10-05T12:00:00Z INFO  app::http: listening on :8080
2026-10-05T12:00:03Z DEBUG app::db::pool: connection opened
2026-10-05T12:00:04Z WARN  app::db: slow query took 812 ms
2026-10-05T12:00:09Z ERROR app::http: request failed: timeout
this line is not a log line
2026-10-05T12:01:00Z INFO  app::http: GET /health 200
";

/// Parse `args` as the given clap CLI (with `name` as argv[0]).
fn parse<T: Parser>(name: &str, args: &[&str]) -> T {
    T::parse_from(std::iter::once(name).chain(args.iter().copied()))
}

fn ex1() {
    println!("--- 1: minigrep");
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("poem.txt"),
        "I'm nobody! Who are you?\nAre you nobody, too?\nThen there's a pair of us!\n",
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();
    std::fs::write(
        dir.path().join("sub/more.txt"),
        "How dreary to be somebody!\n",
    )
    .unwrap();
    let path = dir.path().to_str().unwrap();
    for args in [
        vec!["-n", "-i", "NOBODY", &format!("{path}/poem.txt")],
        vec!["-c", "you", &format!("{path}/poem.txt")],
        vec!["-r", "body", path],
        vec!["zebra", &format!("{path}/poem.txt")],
        vec!["-n", "body", path],
    ] {
        let grep_args: GrepArgs = parse("minigrep", &args);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = ex01_grep::run(&grep_args, &mut out, &mut err, false);
        println!(
            "$ minigrep {}   (exit {code})",
            args.join(" ").replace(path, "DIR")
        );
        print!(
            "{}{}",
            String::from_utf8_lossy(&out).replace(path, "DIR"),
            String::from_utf8_lossy(&err).replace(path, "DIR")
        );
    }
}

fn ex2() {
    println!("--- 2: logview");
    for args in [
        vec!["filter", "--level", "warn"],
        vec!["filter", "--target", "app::db", "--json"],
        vec!["stats"],
        vec!["tail", "-n", "2"],
    ] {
        let cli: LogviewCli = parse("logview", &args);
        let mut out = Vec::new();
        ex02_logview::run(&cli.command, SAMPLE_LOG.as_bytes(), &mut out).unwrap();
        println!(
            "$ logview {}\n{}",
            args.join(" "),
            String::from_utf8_lossy(&out)
        );
    }
}

fn ex3() {
    println!("--- 3: tasks");
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tasks.json");
    let file = file.to_str().unwrap();
    for args in [
        vec!["add", "Write the README", "-p", "low"],
        vec![
            "add",
            "Fix the parser",
            "-p",
            "high",
            "-t",
            "bug",
            "--due",
            "2026-10-31",
        ],
        vec!["add", "Review PR", "-t", "work"],
        vec!["done", "3"],
        vec!["list"],
        vec!["list", "--all"],
        vec!["remove", "9"],
    ] {
        let mut full = vec!["--file", file];
        full.extend(&args);
        let cli: TasksCli = parse("tasks", &full);
        let mut out = Vec::new();
        let result = ex03_tasks::run(&cli, &mut out);
        println!("$ tasks {}", args.join(" "));
        print!("{}", String::from_utf8_lossy(&out));
        if let Err(e) = result {
            println!("error: {e}");
        }
    }
    println!(
        "the store on disk:\n{}",
        std::fs::read_to_string(file).unwrap()
    );
}

fn ex4() {
    println!("--- 4: the monitor, one frame of this machine rendered off-screen");
    let mut source = SysinfoSource::new();
    let mut app = App::new();
    app.update(source.snapshot());
    std::thread::sleep(Duration::from_millis(300)); // CPU usage needs two samples
    app.update(source.snapshot());
    app.on_key(Key::Char('m'));
    let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
    terminal.draw(|f| ex04_monitor::render(f, &app)).unwrap();
    let buffer = terminal.backend().buffer();
    for y in 0..buffer.area.height {
        let line: String = (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect();
        println!("{line}");
    }
}

fn bonus() {
    println!("--- bonus: completions");
    let script = bonus_completions::completions(clap_complete::Shell::Bash);
    println!(
        "bash completion script: {} lines, starts with {:?}",
        script.lines().count(),
        script.lines().next().unwrap_or("")
    );
    println!(
        "colour on a terminal with NO_COLOR=1: {}",
        bonus_completions::should_color(ex01_grep::ColorChoice::Auto, true, Some("1"))
    );
}

/// The real interactive monitor.
fn monitor() -> io::Result<()> {
    use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
    let mut terminal = ratatui::init();
    let mut source = SysinfoSource::new();
    let mut app = App::new();
    let result = (|| -> io::Result<()> {
        while !app.quit {
            app.update(source.snapshot());
            terminal.draw(|f| ex04_monitor::render(f, &app))?;
            if event::poll(Duration::from_millis(1000))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let key = match key.code {
                            KeyCode::Char(c) => Some(Key::Char(c)),
                            KeyCode::Up => Some(Key::Up),
                            KeyCode::Down => Some(Key::Down),
                            KeyCode::Esc => Some(Key::Esc),
                            _ => None,
                        };
                        if let Some(key) = key {
                            app.on_key(key);
                        }
                    }
                }
            }
        }
        Ok(())
    })();
    ratatui::restore();
    result
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rest: Vec<&str> = args.iter().skip(2).map(String::as_str).collect();
    let code = match args.get(1).map(String::as_str) {
        Some("grep") => {
            let grep_args: GrepArgs = parse("minigrep", &rest);
            ex01_grep::run(
                &grep_args,
                &mut io::stdout(),
                &mut io::stderr(),
                io::stdout().is_terminal(),
            )
        }
        Some("logview") => {
            let cli: LogviewCli = parse("logview", &rest);
            let result = match cli.command.file() {
                Some(path) => std::fs::File::open(path).and_then(|f| {
                    ex02_logview::run(&cli.command, BufReader::new(f), &mut io::stdout())
                }),
                None => ex02_logview::run(&cli.command, io::stdin().lock(), &mut io::stdout()),
            };
            result.map_or_else(
                |e| {
                    eprintln!("logview: {e}");
                    2
                },
                |()| 0,
            )
        }
        Some("tasks") => {
            let cli: TasksCli = parse("tasks", &rest);
            ex03_tasks::run(&cli, &mut io::stdout()).map_or_else(
                |e| {
                    eprintln!("tasks: {e}");
                    1
                },
                |()| 0,
            )
        }
        Some("monitor") => monitor().map_or_else(
            |e| {
                eprintln!("monitor: {e}");
                1
            },
            |()| 0,
        ),
        Some("completions") => {
            let shell = rest
                .first()
                .and_then(|s| s.parse().ok())
                .unwrap_or(clap_complete::Shell::Bash);
            print!("{}", bonus_completions::completions(shell));
            0
        }
        Some("1") => {
            ex1();
            0
        }
        Some("2") => {
            ex2();
            0
        }
        Some("3") => {
            ex3();
            0
        }
        Some("4") => {
            ex4();
            0
        }
        Some("bonus") => {
            bonus();
            0
        }
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            bonus();
            0
        }
        _ => {
            println!("31-cli-tools -- your workspace\n");
            println!("  cargo run -p m31-cli-tools -- <1-4|bonus|all>      demos");
            println!("  cargo run -p m31-cli-tools -- grep|logview|tasks|monitor|completions ...");
            0
        }
    };
    let _ = io::stdout().flush();
    std::process::exit(code);
}
