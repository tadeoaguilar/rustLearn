// 37-wasm -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m37-wasm -- <1-4|bonus|all>     the library, natively
//     cargo run -p m37-wasm -- wc <dir>            Exercise 4 as a command (also builds for wasm32-wasip1)

use m37_wasm::ex02_life::Universe;
use m37_wasm::{bonus_image, ex01_text, ex03_interop, ex04_wasi};

fn ex1() {
    println!("--- 1: text utilities");
    println!(
        "word_count: {}",
        ex01_text::word_count("Rust compiles to WebAssembly")
    );
    println!(
        "slugify: {}",
        ex01_text::slugify("Hello, WASM World! (2026)")
    );
    let md = "# Title\n\nSome **bold** and *em* text with `code`.\n\n- one\n- two <script>\n";
    println!("markdown_to_html:\n{}", ex01_text::markdown_to_html(md));
}

fn ex2() {
    println!("--- 2: Game of Life (a glider)");
    let mut universe = Universe::parse(".#....\n..#...\n###...\n......\n......\n").unwrap();
    for generation in 0..3 {
        println!(
            "generation {generation} (population {}):\n{}",
            universe.population(),
            universe.render()
        );
        universe.tick();
    }
}

fn ex3() {
    println!("--- 3: across the boundary");
    println!(
        "{:?}",
        ex03_interop::stats(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])
    );
    println!("{:?}", ex03_interop::stats(&[]));
    println!(
        "normalize: {:?}",
        ex03_interop::normalize(&[10.0, 15.0, 20.0])
    );
    println!(
        "{:?}",
        ex03_interop::Signup::new("Ana@Example.COM", 30).map(|s| (s.email(), s.domain()))
    );
    println!("{:?}", ex03_interop::Signup::new("not-an-email", 30));
}

fn ex4(dir: Option<&str>) {
    println!("--- 4: WASI-ready file processing");
    let temp;
    let dir = match dir {
        Some(d) => std::path::PathBuf::from(d),
        None => {
            temp = std::env::temp_dir().join("m37-wasi-demo");
            std::fs::create_dir_all(&temp).unwrap();
            std::fs::write(temp.join("a.txt"), "one two\nthree\n").unwrap();
            std::fs::write(temp.join("b.txt"), "four five six\n").unwrap();
            temp.clone()
        }
    };
    let stats = ex04_wasi::scan_dir(&dir, ".txt").unwrap();
    let report = dir.join("report.out");
    ex04_wasi::write_report(&stats, &report).unwrap();
    print!("{}", std::fs::read_to_string(report).unwrap());
}

fn bonus() {
    println!("--- bonus: image filters on a 2x1 RGBA image");
    let mut px = vec![255, 0, 0, 255, 0, 0, 255, 255];
    bonus_image::grayscale(&mut px);
    println!("grayscale of red, blue: {px:?}");
    println!(
        "blur: {:?}",
        bonus_image::box_blur(&[0, 0, 0, 255, 200, 200, 200, 255], 2, 1)
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(None),
        Some("wc") => ex4(Some(args.get(2).map_or(".", String::as_str))),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4(None);
            bonus();
        }
        _ => println!(
            "37-wasm -- your workspace\n\n  cargo run -p m37-wasm -- <1-4|bonus|all|wc DIR>"
        ),
    }
}
