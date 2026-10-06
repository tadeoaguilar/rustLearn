use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::sut::ex01_supervisor::{self as supervisor, Outcome, Policy, Restart, Supervisor};
use crate::sut::ex02_shell::{self as shell, Control, Pipeline, Shell, ShellError, Token};
use crate::sut::ex03_ipc::{self as ipc, KvClient};
use crate::sut::ex04_shared_memory::{CacheError, SharedCache};
use crate::sut::{bonus_jobs, ex05_signals as signals};

const MS: Duration = Duration::from_millis(1);

fn sh(script: &str) -> Outcome {
    Outcome::from_status(Command::new("sh").args(["-c", script]).status().unwrap())
}

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_outcomes() {
    assert_eq!(sh("exit 0"), Outcome::Exited(0));
    assert_eq!(sh("exit 3"), Outcome::Exited(3));
    assert_eq!(sh("kill -9 $$"), Outcome::Signaled(9));
    assert!(
        Outcome::Exited(0).success()
            && !Outcome::Exited(1).success()
            && !Outcome::Signaled(15).success()
    );
}

#[test]
fn ex1_backoff() {
    let (base, max) = (10 * MS, 100 * MS);
    let delays: Vec<Duration> = (0..6).map(|a| supervisor::backoff(a, base, max)).collect();
    assert_eq!(
        delays,
        [10 * MS, 20 * MS, 40 * MS, 80 * MS, 100 * MS, 100 * MS]
    );
    assert_eq!(supervisor::backoff(40, base, max), max, "no overflow");
    assert_eq!(
        supervisor::backoff(
            u32::MAX,
            Duration::from_secs(u64::MAX / 4),
            Duration::from_secs(5)
        ),
        Duration::from_secs(5)
    );
}

fn policy(restart: Restart, max_restarts: u32) -> Policy {
    Policy {
        restart,
        max_restarts,
        backoff_base: 5 * MS,
        backoff_max: 20 * MS,
    }
}

#[test]
fn ex1_should_restart() {
    let fail = Outcome::Exited(1);
    let ok = Outcome::Exited(0);
    assert!(!supervisor::should_restart(
        &policy(Restart::Never, 9),
        fail,
        0
    ));
    assert!(supervisor::should_restart(
        &policy(Restart::OnFailure, 9),
        fail,
        0
    ));
    assert!(supervisor::should_restart(
        &policy(Restart::OnFailure, 9),
        Outcome::Signaled(9),
        0
    ));
    assert!(!supervisor::should_restart(
        &policy(Restart::OnFailure, 9),
        ok,
        0
    ));
    assert!(supervisor::should_restart(
        &policy(Restart::Always, 9),
        ok,
        8
    ));
    assert!(
        !supervisor::should_restart(&policy(Restart::Always, 9), ok, 9),
        "out of restarts"
    );
}

#[test]
fn ex1_supervisor_restarts_until_success() {
    let dir = tempfile::tempdir().unwrap();
    let counter = dir.path().join("runs");
    // fails twice, then succeeds
    let script = format!(
        "n=$(cat {0} 2>/dev/null || echo 0); n=$((n+1)); echo $n > {0}; [ $n -ge 3 ]",
        counter.display()
    );
    let sup = Supervisor::new("sh", &["-c", &script], policy(Restart::OnFailure, 10));
    let mut slept = Vec::new();
    let history = sup.run(&mut |d| slept.push(d)).unwrap();
    let outcomes: Vec<Outcome> = history.iter().map(|r| r.outcome).collect();
    assert_eq!(
        outcomes,
        [Outcome::Exited(1), Outcome::Exited(1), Outcome::Exited(0)]
    );
    assert_eq!(slept, [5 * MS, 10 * MS]);
    assert_eq!(history[2].delay_before_restart, None);
}

#[test]
fn ex1_supervisor_limits_and_stop() {
    let mut never_sleep = |_| {};
    let always = Supervisor::new("true", &[], policy(Restart::Always, 2));
    assert_eq!(
        always.run(&mut never_sleep).unwrap().len(),
        3,
        "the first run and 2 restarts"
    );
    let never = Supervisor::new("false", &[], policy(Restart::Never, 5));
    assert_eq!(never.run(&mut never_sleep).unwrap().len(), 1);
    let stopped = Supervisor::new("false", &[], policy(Restart::Always, 5));
    stopped.stop.store(true, Ordering::SeqCst);
    assert_eq!(
        stopped.run(&mut never_sleep).unwrap().len(),
        1,
        "stop is honoured"
    );
    assert!(
        Supervisor::new("/no/such/program", &[], policy(Restart::Always, 5))
            .run(&mut never_sleep)
            .is_err()
    );
}

// ---------------------------------------------------------------- Exercise 2

fn words(tokens: &[&str]) -> Vec<Token> {
    tokens.iter().map(|w| Token::Word(w.to_string())).collect()
}

#[test]
fn ex2_tokenize() {
    let vars: HashMap<String, String> = [("NAME", "ana"), ("?", "3"), ("EMPTY", "")]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
    let t = |line: &str| shell::tokenize(line, &vars).unwrap();
    assert_eq!(t("ls  -l\t/tmp"), words(&["ls", "-l", "/tmp"]));
    assert_eq!(t("echo 'a  b' \"c d\""), words(&["echo", "a  b", "c d"]));
    assert_eq!(
        t("echo $NAME '$NAME' \"$NAME!\" $? x$EMPTY"),
        words(&["echo", "ana", "$NAME", "ana!", "3", "x"])
    );
    assert_eq!(
        t("echo a\\ b \"q\\\"uote\""),
        words(&["echo", "a b", "q\"uote"])
    );
    assert_eq!(
        t("echo ''"),
        words(&["echo", ""]),
        "an empty quoted word is a word"
    );
    assert_eq!(
        t("a|b>c>>d<e"),
        [
            Token::Word("a".into()),
            Token::Pipe,
            Token::Word("b".into()),
            Token::Out,
            Token::Word("c".into()),
            Token::Append,
            Token::Word("d".into()),
            Token::In,
            Token::Word("e".into())
        ]
    );
    assert_eq!(
        t("cost $5"),
        words(&["cost", ""]),
        "unset variables are empty"
    );
    assert_eq!(t("a $ b"), words(&["a", "$", "b"]));
    assert_eq!(
        shell::tokenize("echo 'oops", &vars),
        Err(ShellError::UnterminatedQuote)
    );
    assert_eq!(
        shell::tokenize("echo \"oops", &vars),
        Err(ShellError::UnterminatedQuote)
    );
}

#[test]
fn ex2_parse() {
    let vars = HashMap::new();
    let p = |line: &str| shell::parse(&shell::tokenize(line, &vars).unwrap());
    assert_eq!(
        p("sort < in.txt | uniq -c >> out.txt").unwrap(),
        Pipeline {
            stages: vec![vec!["sort".into()], vec!["uniq".into(), "-c".into()]],
            input: Some("in.txt".into()),
            output: Some(("out.txt".into(), true)),
        }
    );
    assert_eq!(p("").unwrap(), Pipeline::default());
    for bad in [
        "| a",
        "a |",
        "a || b",
        "a >",
        "a < ",
        "a | b < c",
        "a > f | b",
    ] {
        assert!(matches!(p(bad), Err(ShellError::Syntax(_))), "{bad:?}");
    }
}

fn shell_in(dir: &tempfile::TempDir) -> Shell {
    Shell::new(dir.path().canonicalize().unwrap())
}

fn run(shell: &mut Shell, line: &str) -> (Control, String) {
    let mut out = Vec::new();
    let control = shell.run_line(line, &mut out).unwrap();
    (control, String::from_utf8(out).unwrap())
}

#[test]
fn ex2_pipelines_and_redirection() {
    let dir = tempfile::tempdir().unwrap();
    let mut sh = shell_in(&dir);
    assert_eq!(
        run(&mut sh, "echo hello world"),
        (Control::Continue(0), "hello world\n".into())
    );
    assert_eq!(
        run(&mut sh, "printf 'b\\na\\nb\\n' | sort | uniq").1,
        "a\nb\n"
    );
    assert_eq!(
        run(&mut sh, "echo first > f.txt").1,
        "",
        "redirected output isn't captured"
    );
    run(&mut sh, "echo second >> f.txt");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("f.txt")).unwrap(),
        "first\nsecond\n"
    );
    assert_eq!(run(&mut sh, "tr a-z A-Z < f.txt").1, "FIRST\nSECOND\n");
    run(&mut sh, "echo replaced > f.txt");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("f.txt")).unwrap(),
        "replaced\n",
        "> truncates"
    );
    // a big pipeline output doesn't deadlock
    assert_eq!(run(&mut sh, "seq 1 50000 | tail -n 1").1, "50000\n");
}

#[test]
fn ex2_status_variables_and_builtins() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();
    let mut sh = shell_in(&dir);
    assert_eq!(run(&mut sh, "sh -c 'exit 5'").0, Control::Continue(5));
    assert_eq!(run(&mut sh, "echo $?").1, "5\n");
    assert_eq!(
        run(&mut sh, "this-command-does-not-exist").0,
        Control::Continue(127)
    );
    run(&mut sh, "export WHO=rustacean");
    assert_eq!(
        run(&mut sh, "sh -c 'echo hi $WHO'").1,
        "hi rustacean\n",
        "exported to children"
    );
    run(&mut sh, "cd sub");
    assert!(run(&mut sh, "pwd").1.trim_end().ends_with("/sub"));
    assert_eq!(
        run(&mut sh, "sh -c pwd").1,
        run(&mut sh, "pwd").1,
        "children start in the shell's directory"
    );
    assert_eq!(run(&mut sh, "cd nowhere").0, Control::Continue(1));
    assert_eq!(run(&mut sh, "exit 4").0, Control::Exit(4));
    assert!(matches!(
        sh.run_line("cd sub | cat", &mut Vec::new()),
        Err(ShellError::Syntax(_))
    ));
}

// ---------------------------------------------------------------- Exercise 3

#[test]
fn ex3_pipes_to_a_child() {
    assert_eq!(
        ipc::sort_via_child(&["pear", "apple", "Fig"]).unwrap(),
        ["Fig", "apple", "pear"]
    );
    assert!(ipc::sort_via_child(&[]).unwrap().is_empty());
}

#[test]
fn ex3_message_framing() {
    let mut buf = Vec::new();
    ipc::write_message(&mut buf, b"hello").unwrap();
    ipc::write_message(&mut buf, b"").unwrap();
    assert_eq!(&buf[..9], b"\0\0\0\x05hello");
    let mut r = &buf[..];
    assert_eq!(ipc::read_message(&mut r).unwrap(), Some(b"hello".to_vec()));
    assert_eq!(ipc::read_message(&mut r).unwrap(), Some(vec![]));
    assert_eq!(ipc::read_message(&mut r).unwrap(), None);
    let huge = ((ipc::MAX_MESSAGE + 1) as u32).to_be_bytes();
    assert!(ipc::read_message(&mut &huge[..]).is_err());
}

#[test]
fn ex3_kv_commands() {
    let store = Mutex::new(HashMap::new());
    assert_eq!(ipc::handle_command(&store, "GET a"), "NOT_FOUND");
    assert_eq!(ipc::handle_command(&store, "SET a hello world"), "OK");
    assert_eq!(
        ipc::handle_command(&store, "GET a"),
        "VALUE hello world",
        "values may contain spaces"
    );
    assert_eq!(ipc::handle_command(&store, "DEL a"), "OK");
    assert_eq!(ipc::handle_command(&store, "DEL a"), "NOT_FOUND");
    assert!(ipc::handle_command(&store, "FLY away").starts_with("ERROR"));
    assert!(ipc::handle_command(&store, "SET onlykey").starts_with("ERROR"));
}

#[test]
fn ex3_unix_socket_server_shares_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kv.sock");
    ipc::spawn_kv_server(&path).unwrap();
    let mut first = KvClient::connect(&path).unwrap();
    first.set("lang", "rust").unwrap();
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let path = path.clone();
            std::thread::spawn(move || {
                let mut c = KvClient::connect(&path).unwrap();
                c.set(&format!("k{i}"), &i.to_string()).unwrap();
                c.get("lang").unwrap()
            })
        })
        .collect();
    for h in handles {
        assert_eq!(h.join().unwrap().as_deref(), Some("rust"));
    }
    assert_eq!(first.get("k5").unwrap().as_deref(), Some("5"));
    assert!(first.delete("k5").unwrap());
    assert_eq!(first.get("k5").unwrap(), None);
}

#[test]
fn ex3_socketpair() {
    assert_eq!(
        ipc::double_via_socketpair(&[0, 1, 50, 1000]).unwrap(),
        [0, 2, 100, 2000]
    );
}

// ---------------------------------------------------------------- Exercise 4

#[test]
fn ex4_put_get_across_mappings() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("c.bin");
    let a = SharedCache::create(&path, 8).unwrap();
    a.put(42, b"answer").unwrap();
    a.put(50, b"fifty").unwrap(); // 50 % 8 == 42 % 8: probes to the next slot
    let b = SharedCache::open(&path).unwrap();
    assert_eq!(b.get(42).as_deref(), Some(&b"answer"[..]));
    assert_eq!(b.get(50).as_deref(), Some(&b"fifty"[..]));
    b.put(42, b"changed").unwrap();
    assert_eq!(
        a.get(42).as_deref(),
        Some(&b"changed"[..]),
        "a write in one mapping is visible in the other"
    );
    assert_eq!(a.get(7), None);
    assert_eq!(a.get(0), None);
}

#[test]
fn ex4_errors() {
    let dir = tempfile::tempdir().unwrap();
    let cache = SharedCache::create(&dir.path().join("c.bin"), 2).unwrap();
    assert!(matches!(cache.put(0, b"x"), Err(CacheError::ZeroKey)));
    assert!(matches!(
        cache.put(1, &[0; 56]),
        Err(CacheError::ValueTooLong)
    ));
    cache.put(1, &[1; 55]).unwrap();
    cache.put(2, b"b").unwrap();
    assert!(matches!(cache.put(3, b"c"), Err(CacheError::Full)));
    cache.put(2, b"replaced").unwrap();
    std::fs::write(dir.path().join("junk.bin"), vec![1u8; 100]).unwrap();
    assert!(matches!(
        SharedCache::open(&dir.path().join("junk.bin")),
        Err(CacheError::BadMagic)
    ));
}

#[test]
fn ex4_atomic_counter_from_many_mappings() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("c.bin");
    let cache = SharedCache::create(&path, 4).unwrap();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let path = path.clone();
            std::thread::spawn(move || {
                let mine = SharedCache::open(&path).unwrap(); // its own mapping
                for _ in 0..5_000 {
                    mine.increment();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(cache.count(), 40_000);
}

#[test]
fn ex4_concurrent_puts_from_other_mappings_all_land() {
    // Without the lock, two writers probing for a free slot at the same time
    // can both claim it, and one entry overwrites the other.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("c.bin");
    let cache = SharedCache::create(&path, 128).unwrap();
    let handles: Vec<_> = (0..8u64)
        .map(|t| {
            let path = path.clone();
            std::thread::spawn(move || {
                let mine = SharedCache::open(&path).unwrap();
                for i in 0..12u64 {
                    // all keys collide on the same starting slot (multiples of 128)
                    let key = (t * 12 + i + 1) * 128;
                    mine.put(key, &key.to_le_bytes()).unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    for key in (1..=96u64).map(|k| k * 128) {
        assert_eq!(
            cache.get(key),
            Some(key.to_le_bytes().to_vec()),
            "key {key}"
        );
    }
    let start = Instant::now();
    drop(cache.lock());
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "the lock is free afterwards"
    );
}

// ---------------------------------------------------------------- Exercise 5

#[test]
fn ex5_signal_flags() {
    let flag = signals::flag_on(signal_hook::consts::SIGUSR2).unwrap();
    assert!(!signals::wait_for(&flag, 20 * MS));
    signal_hook::low_level::raise(signal_hook::consts::SIGUSR2).unwrap();
    assert!(signals::wait_for(&flag, Duration::from_secs(2)));
}

#[test]
fn ex5_terminate_politely_then_forcefully() {
    let mut polite = Command::new("sleep").arg("30").spawn().unwrap();
    assert_eq!(
        signals::terminate(&mut polite, Duration::from_secs(5)).unwrap(),
        Outcome::Signaled(15)
    );
    let mut stubborn = Command::new("sh")
        .args(["-c", "trap '' TERM; sleep 30 & wait"])
        .spawn()
        .unwrap();
    std::thread::sleep(100 * MS); // let the trap be installed
    let start = Instant::now();
    assert_eq!(
        signals::terminate(&mut stubborn, 300 * MS).unwrap(),
        Outcome::Signaled(9)
    );
    assert!(start.elapsed() >= 290 * MS, "waited the grace period first");
}

#[test]
fn ex5_resource_limits() {
    let (soft, hard) = signals::open_files_limit().unwrap();
    assert!(soft >= 3 && soft <= hard);
    let out =
        signals::run_with_file_limit(Command::new("sh").args(["-c", "ulimit -n"]), 64).unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "64");
    assert_eq!(
        signals::open_files_limit().unwrap().0,
        soft,
        "the parent's limit is unchanged"
    );
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_parallel_jobs() {
    let jobs: Vec<Vec<String>> = vec![
        vec!["sh".into(), "-c".into(), "sleep 0.3; exit 1".into()],
        vec!["sh".into(), "-c".into(), "exit 2".into()],
        vec!["/no/such/program".into()],
        vec!["sh".into(), "-c".into(), "sleep 0.3; exit 3".into()],
    ];
    let start = Instant::now();
    let results = bonus_jobs::run_parallel(&jobs, 2).unwrap();
    assert_eq!(
        results,
        [
            Outcome::Exited(1),
            Outcome::Exited(2),
            Outcome::Exited(127),
            Outcome::Exited(3)
        ],
        "in input order"
    );
    assert!(start.elapsed() < Duration::from_secs(3));

    let sleepers: Vec<Vec<String>> = (0..4).map(|_| vec!["sleep".into(), "0.3".into()]).collect();
    let start = Instant::now();
    bonus_jobs::run_parallel(&sleepers, 2).unwrap();
    assert!(
        start.elapsed() >= 550 * MS,
        "never more than 2 at once: {:?}",
        start.elapsed()
    );
    let start = Instant::now();
    bonus_jobs::run_parallel(&sleepers, 4).unwrap();
    assert!(
        start.elapsed() < 550 * MS,
        "4 at once: {:?}",
        start.elapsed()
    );
}
