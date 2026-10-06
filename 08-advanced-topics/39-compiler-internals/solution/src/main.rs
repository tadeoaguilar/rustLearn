// Reference solution for 39-compiler-internals.
//
//     cargo run -p m39-compiler-internals-solution -- <1-4|bonus|all>
//     cargo run -p m39-compiler-internals-solution -- lint <file.rs>...
//     cargo run -p m39-compiler-internals-solution -- mir <file.rs>
//
// Exercises 2-4 run your installed rustc (offline).

use m39_compiler_internals_solution::{
    bonus_unused, ex01_lint, ex02_mir, ex03_borrowck, ex04_desugar, toolchain,
};

const LINT_SAMPLE: &str = r#"
pub fn parse_port(text: &str) -> u16 {
    let n: i64 = text.trim().parse().unwrap();
    dbg!(n);
    n as u16
}

/// Documented, and fine.
pub fn half(x: u32) -> u32 { x / 2 }

#[cfg(test)]
mod tests {
    #[test]
    fn t() { super::parse_port("80").to_string().parse::<u16>().unwrap(); }
}
"#;

const MIR_SAMPLE: &str = r#"
pub fn sum_indexed(v: &[u32]) -> u32 { let mut s = 0; for i in 0..v.len() { s += v[i]; } s }
pub fn sum_iter(v: &[u32]) -> u32 { v.iter().sum() }
pub fn add(a: i32, b: i32) -> i32 { a + b }
pub fn first(v: &[u8]) -> u8 { v[0] }
pub fn first_checked(v: &[u8]) -> Option<u8> { v.first().copied() }
"#;

const DESUGAR_SAMPLE: &str = r#"
fn parse_all(items: &[&str]) -> Result<Vec<i32>, std::num::ParseIntError> {
    let mut out = Vec::new();
    for item in items {
        out.push(item.parse::<i32>()?);
    }
    Ok(out)
}

fn main() {
    let mut total = 0;
    'outer: for row in [[1, 2], [3, 4], [5, 6]] {
        for x in row {
            if x == 5 { break 'outer; }
            total += x;
        }
    }
    println!("{:?} {:?} {}", parse_all(&["1", "22"]), parse_all(&["1", "x"]).is_err(), total);
}
"#;

fn ex1() {
    println!("--- 1: lints");
    for finding in ex01_lint::lint_source(LINT_SAMPLE, &ex01_lint::Config::default()).unwrap() {
        println!("sample.rs:{finding}");
    }
}

fn ex2() {
    println!("--- 2: MIR (debug, then -O)");
    for optimize in [false, true] {
        let mir = toolchain::emit_mir(MIR_SAMPLE, optimize).expect("rustc");
        for f in ex02_mir::parse_mir(&mir) {
            println!(
                "{:>3} {:<14} {:>2} blocks  {} bounds checks  {} overflow checks  {} calls",
                if optimize { "-O" } else { "" },
                f.name,
                f.basic_blocks,
                f.bounds_checks,
                f.overflow_checks,
                f.calls
            );
        }
    }
}

fn ex3() {
    println!("--- 3: borrow checker");
    for case in ex03_borrowck::CASES {
        let broken = ex03_borrowck::error_codes(case.broken).expect("rustc");
        let fixed =
            ex03_borrowck::error_codes(ex03_borrowck::fixed(case.code).unwrap()).expect("rustc");
        println!(
            "{} {:<42} broken -> {:?}, fixed -> {:?}",
            case.code, case.title, broken, fixed
        );
    }
    println!("longest = {}", ex03_borrowck::longest("borrow", "checker"));
    println!(
        "spawn_sum = {}",
        ex03_borrowck::spawn_sum(vec![1, 2, 3]).join().unwrap()
    );
}

fn ex4() {
    println!("--- 4: desugaring");
    let desugared = ex04_desugar::desugar(DESUGAR_SAMPLE).unwrap();
    println!(
        "sugar before: {:?}, after: {:?}",
        ex04_desugar::count_sugar(DESUGAR_SAMPLE).unwrap(),
        ex04_desugar::count_sugar(&desugared).unwrap()
    );
    println!(
        "original:  {}",
        toolchain::run_program(DESUGAR_SAMPLE)
            .expect("rustc")
            .trim()
    );
    println!(
        "desugared: {}",
        toolchain::run_program(&desugared).expect("rustc").trim()
    );
}

fn bonus() {
    println!("--- bonus: unused variables");
    let sample = "fn f(a: i32, b: i32) -> i32 { let c = 1; let _d = 2; let e = a * 2; println!(\"{e}\"); a }";
    println!("{:?}", bonus_unused::unused_variables(sample).unwrap());
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            bonus();
        }
        Some("lint") => {
            for path in &args[1..] {
                let source =
                    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
                match ex01_lint::lint_source(&source, &ex01_lint::Config::default()) {
                    Ok(findings) => findings.iter().for_each(|f| println!("{path}:{f}")),
                    Err(e) => println!("{path}: parse error: {e}"),
                }
            }
        }
        Some("mir") if args.len() == 2 => {
            let source = std::fs::read_to_string(&args[1]).expect("read file");
            for f in ex02_mir::parse_mir(&toolchain::emit_mir(&source, false).expect("rustc")) {
                println!("{f:?}");
            }
        }
        _ => println!(
            "39-compiler-internals -- reference solution\n\n  cargo run -p m39-compiler-internals-solution -- <1-4|bonus|all>\n  ... -- lint <file.rs>...\n  ... -- mir <file.rs>"
        ),
    }
}
