// Reference solution for 34-os-concepts (Unix).
//
//     cargo run -p m34-os-concepts-solution -- <1-5|bonus|all>     demos
//     cargo run -p m34-os-concepts-solution -- shell               an interactive shell (exit to leave)

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use m34_os_concepts_solution::ex01_supervisor::{Policy, Restart, Supervisor};
use m34_os_concepts_solution::ex02_shell::{Control, Shell};
use m34_os_concepts_solution::ex03_ipc::{self, KvClient};
use m34_os_concepts_solution::ex04_shared_memory::SharedCache;
use m34_os_concepts_solution::{bonus_jobs, ex05_signals};

fn ex1() {
    println!("--- 1: a supervisor restarting a crashing program");
    let policy = Policy {
        restart: Restart::OnFailure,
        max_restarts: 4,
        backoff_base: Duration::from_millis(10),
        backoff_max: Duration::from_millis(50),
    };
    let supervisor = Supervisor::new("sh", &["-c", "echo '  child running'; exit 3"], policy);
    let history = supervisor.run(&mut |d| std::thread::sleep(d)).unwrap();
    for (i, run) in history.iter().enumerate() {
        println!(
            "run {i}: {:?}, then wait {:?}",
            run.outcome, run.delay_before_restart
        );
    }
}

fn ex2() {
    println!("--- 2: a shell");
    let dir = tempfile::tempdir().unwrap();
    let mut shell = Shell::new(dir.path().to_path_buf());
    for line in [
        "echo hello 'quoted   spaces' \"and $HOME-ish vars\"",
        "printf 'banana\\napple\\ncherry\\n' | sort | tr a-z A-Z",
        "echo one > out.txt",
        "echo two >> out.txt",
        "wc -l < out.txt",
        "export GREETING=hi",
        "sh -c 'echo $GREETING from a child; exit 7'",
        "echo last status was $?",
        "no-such-command",
        "pwd",
    ] {
        let mut out = Vec::new();
        let control = shell.run_line(line, &mut out);
        println!(
            "$ {line}\n{}  -> {control:?}",
            String::from_utf8_lossy(&out).replace(&*dir.path().to_string_lossy(), "TMP")
        );
    }
}

fn ex3() {
    println!("--- 3: IPC");
    println!(
        "sorted by a child through pipes: {:?}",
        ex03_ipc::sort_via_child(&["pear", "apple", "fig"]).unwrap()
    );
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("kv.sock");
    ex03_ipc::spawn_kv_server(&socket).unwrap();
    let mut a = KvClient::connect(&socket).unwrap();
    let mut b = KvClient::connect(&socket).unwrap();
    a.set("color", "teal").unwrap();
    println!(
        "client B reads what client A set over {}: {:?}",
        socket.file_name().unwrap().to_string_lossy(),
        b.get("color").unwrap()
    );
    println!(
        "doubled through a socketpair: {:?}",
        ex03_ipc::double_via_socketpair(&[1, 2, 21]).unwrap()
    );
}

fn ex4() {
    println!("--- 4: shared memory across processes");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.bin");
    let cache = SharedCache::create(&path, 32).unwrap();
    cache.put(7, b"seven").unwrap();
    let me = std::env::current_exe().unwrap();
    let start = Instant::now();
    let workers: Vec<_> = (0..4)
        .map(|_| {
            Command::new(&me)
                .args(["worker", path.to_str().unwrap(), "10000"])
                .spawn()
                .unwrap()
        })
        .collect();
    for mut w in workers {
        w.wait().unwrap();
    }
    println!(
        "4 processes x 10000 increments of one mapped counter: {} (in {:?})",
        cache.count(),
        start.elapsed()
    );
    println!(
        "a value a worker wrote: {:?}",
        cache.get(99).map(String::from_utf8)
    );
}

/// A worker process: open the same file, increment `n` times.
fn worker(path: &str, n: u64) {
    let cache = SharedCache::open(&PathBuf::from(path)).unwrap();
    for _ in 0..n {
        cache.increment();
    }
    cache
        .put(99, format!("from pid {}", std::process::id()).as_bytes())
        .unwrap();
}

fn ex5() {
    println!("--- 5: signals and limits");
    let flag = ex05_signals::flag_on(signal_hook::consts::SIGUSR1).unwrap();
    signal_hook::low_level::raise(signal_hook::consts::SIGUSR1).unwrap();
    println!(
        "SIGUSR1 to ourselves, seen by the flag: {}",
        ex05_signals::wait_for(&flag, Duration::from_secs(1))
    );
    let mut sleeper = Command::new("sleep").arg("30").spawn().unwrap();
    println!(
        "`sleep 30` sent SIGTERM: {:?}",
        ex05_signals::terminate(&mut sleeper, Duration::from_secs(2)).unwrap()
    );
    let (soft, hard) = ex05_signals::open_files_limit().unwrap();
    println!("open files limit: soft {soft}, hard {hard}");
    let out = ex05_signals::run_with_file_limit(Command::new("sh").args(["-c", "ulimit -n"]), 64)
        .unwrap();
    println!(
        "a child started with the limit lowered reports: {}",
        String::from_utf8_lossy(&out.stdout).trim()
    );
}

fn bonus() {
    println!("--- bonus: 6 jobs of 0.2 s, 3 at a time");
    let jobs: Vec<Vec<String>> = (0..6)
        .map(|i| vec!["sh".into(), "-c".into(), format!("sleep 0.2; exit {i}")])
        .collect();
    let start = Instant::now();
    let results = bonus_jobs::run_parallel(&jobs, 3).unwrap();
    println!("{results:?} in {:?}", start.elapsed());
}

fn repl() {
    let mut shell = Shell::new(std::env::current_dir().unwrap());
    let stdin = io::stdin();
    loop {
        print!("rsh {}$ ", shell.cwd.display());
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 {
            break;
        }
        let mut out = io::stdout();
        match shell.run_line(line.trim_end(), &mut out) {
            Ok(Control::Exit(code)) => std::process::exit(code),
            Ok(Control::Continue(_)) => {}
            Err(e) => eprintln!("rsh: {e}"),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(),
        Some("5") => ex5(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            ex5();
            bonus();
        }
        Some("worker") => worker(&args[2], args[3].parse().unwrap()),
        Some("shell") => repl(),
        _ => println!(
            "34-os-concepts -- reference solution\n\n  cargo run -p m34-os-concepts-solution -- <1-5|bonus|all|shell>"
        ),
    }
}
