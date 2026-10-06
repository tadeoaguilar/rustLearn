use crate::sut::ex02_life::Universe;
use crate::sut::ex03_interop::{self as interop, Signup};
use crate::sut::{bonus_image as image, ex01_text as text, ex04_wasi as wasi};

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_word_count_and_slugify() {
    assert_eq!(text::word_count(""), 0);
    assert_eq!(text::word_count("  one\ttwo\nthree  "), 3);
    assert_eq!(text::slugify("Hello, WASM World!"), "hello-wasm-world");
    assert_eq!(
        text::slugify("  --Rust & WebAssembly 2026--  "),
        "rust-webassembly-2026"
    );
    assert_eq!(
        text::slugify("Ünïcode café"),
        "n-code-caf",
        "non-ASCII is a separator"
    );
    assert_eq!(text::slugify("!!!"), "");
}

#[test]
fn ex1_escaping() {
    assert_eq!(
        text::escape_html(r#"<a href="x">Tom & 'Jerry'</a>"#),
        "&lt;a href=&quot;x&quot;&gt;Tom &amp; &#39;Jerry&#39;&lt;/a&gt;"
    );
}

#[test]
fn ex1_markdown_blocks() {
    let html = text::markdown_to_html("# Title\n## Sub\n\nline one\nline two\n\n- a\n- b\n\nafter");
    assert_eq!(
        html,
        "<h1>Title</h1>\n<h2>Sub</h2>\n<p>line one line two</p>\n<ul>\n<li>a</li>\n<li>b</li>\n</ul>\n<p>after</p>\n"
    );
    assert_eq!(
        text::markdown_to_html("####### seven"),
        "<p>####### seven</p>\n",
        "at most six levels"
    );
    assert_eq!(text::markdown_to_html("#nospace"), "<p>#nospace</p>\n");
    assert_eq!(
        text::markdown_to_html("- last item"),
        "<ul>\n<li>last item</li>\n</ul>\n",
        "a list at the end is closed"
    );
    assert_eq!(text::markdown_to_html(""), "");
}

#[test]
fn ex1_markdown_inline_and_safety() {
    assert_eq!(
        text::markdown_to_html("**b** *i* `c`"),
        "<p><strong>b</strong> <em>i</em> <code>c</code></p>\n"
    );
    assert_eq!(
        text::markdown_to_html("**bold *nested* text**"),
        "<p><strong>bold <em>nested</em> text</strong></p>\n"
    );
    assert_eq!(
        text::markdown_to_html("`**not bold**`"),
        "<p><code>**not bold**</code></p>\n",
        "no markup inside code"
    );
    assert_eq!(
        text::markdown_to_html("2 * 3 = 6"),
        "<p>2 * 3 = 6</p>\n",
        "an unmatched marker stays"
    );
    assert_eq!(
        text::markdown_to_html("<script>alert(1)</script>"),
        "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>\n",
        "input can never inject HTML"
    );
}

// ---------------------------------------------------------------- Exercise 2

#[test]
fn ex2_cells_and_neighbours() {
    let mut u = Universe::new(4, 3);
    assert_eq!((u.width(), u.height(), u.population()), (4, 3, 0));
    u.set(0, 0, true);
    u.toggle(2, 3);
    assert!(u.is_alive(0, 0) && u.is_alive(2, 3) && !u.is_alive(1, 1));
    assert_eq!(u.population(), 2);
    assert_eq!(u.live_neighbours(1, 1), 1);
    assert_eq!(
        u.live_neighbours(0, 0),
        1,
        "(2,3) wraps around to touch (0,0)"
    );
    u.toggle(0, 0);
    assert!(!u.is_alive(0, 0));
}

#[test]
fn ex2_oscillators_and_still_lifes() {
    let blinker = Universe::parse(".....\n..#..\n..#..\n..#..\n.....").unwrap();
    let mut u = blinker.clone();
    u.tick();
    assert_eq!(u.render(), ".....\n.....\n.###.\n.....\n.....\n");
    u.tick();
    assert_eq!(u, blinker, "period 2");
    let block = Universe::parse("....\n.##.\n.##.\n....").unwrap();
    let mut b = block.clone();
    b.tick();
    assert_eq!(b, block, "a still life");
}

#[test]
fn ex2_glider_travels_across_the_wrapping_edge() {
    let start = Universe::parse(".#....\n..#...\n###...\n......\n......\n......").unwrap();
    let mut u = start.clone();
    for _ in 0..24 {
        u.tick();
        assert_eq!(u.population(), 5);
    }
    // every 4 generations a glider moves one cell diagonally; 24 = once
    // around a 6x6 torus
    assert_eq!(u, start);
}

#[test]
fn ex2_cells_ptr_points_at_the_cells() {
    let mut u = Universe::new(3, 2);
    u.set(1, 2, true);
    let ptr = u.cells_ptr();
    // SAFETY: `ptr` points to width * height bytes owned by `u`, which is
    // alive and not mutated while we read -- exactly how JS uses it.
    let cells = unsafe { std::slice::from_raw_parts(ptr, 6) };
    assert_eq!(cells, [0, 0, 0, 0, 0, 1]);
    assert!(Universe::parse("#.\n#").is_err());
    assert!(Universe::parse("#x").is_err());
}

// ---------------------------------------------------------------- Exercise 3

#[test]
fn ex3_stats() {
    let s = interop::stats(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).unwrap();
    assert_eq!(
        (s.count, s.mean, s.median, s.min, s.max, s.std_dev),
        (8, 5.0, 4.5, 2.0, 9.0, 2.0)
    );
    assert_eq!(
        interop::stats(&[3.0, 1.0, 2.0]).unwrap().median,
        2.0,
        "odd count: the middle value"
    );
    assert!(interop::stats(&[]).is_err());
    assert!(interop::stats(&[1.0, f64::NAN]).is_err());
    assert_eq!(interop::stats(&[-1.0]).unwrap().std_dev, 0.0);
}

#[test]
fn ex3_normalize() {
    assert_eq!(interop::normalize(&[10.0, 15.0, 20.0]), [0.0, 0.5, 1.0]);
    assert_eq!(interop::normalize(&[7.0, 7.0]), [0.0, 0.0]);
    assert!(interop::normalize(&[]).is_empty());
}

#[test]
fn ex3_signup_validation() {
    let s = Signup::new("  Ana@Example.COM ", 30).unwrap();
    assert_eq!(
        (s.email().as_str(), s.age(), s.domain().as_str()),
        ("ana@example.com", 30, "example.com")
    );
    for bad in [
        "no-at.com",
        "@example.com",
        "a@b",
        "a@b@c.com",
        "a@.com",
        "a@com.",
    ] {
        assert!(Signup::new(bad, 30).is_err(), "{bad}");
    }
    assert!(Signup::new("a@b.co", 12).is_err());
    assert!(Signup::new("a@b.co", 121).is_err());
    assert!(Signup::new("a@b.co", 13).is_ok() && Signup::new("a@b.co", 120).is_ok());
}

// ---------------------------------------------------------------- Exercise 4

#[test]
fn ex4_scan_and_report() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("b.txt"), "four five six\n").unwrap();
    std::fs::write(dir.path().join("a.txt"), "one two\nthree\n").unwrap();
    std::fs::write(dir.path().join("skip.md"), "not counted").unwrap();
    std::fs::write(dir.path().join("binary.txt"), [0xff, 0xfe, 0x00]).unwrap();
    std::fs::create_dir(dir.path().join("dir.txt")).unwrap();
    let stats = wasi::scan_dir(dir.path(), ".txt").unwrap();
    assert_eq!(
        stats,
        [
            wasi::stats_of("a.txt", "one two\nthree\n"),
            wasi::stats_of("b.txt", "four five six\n")
        ]
    );
    assert_eq!((stats[0].lines, stats[0].words, stats[0].bytes), (2, 3, 14));
    let report = dir.path().join("report.out");
    wasi::write_report(&stats, &report).unwrap();
    let text = std::fs::read_to_string(report).unwrap();
    assert_eq!(
        text,
        "     2      3       14 a.txt\n     1      3       14 b.txt\n     3      6       28 total\n"
    );
    assert!(wasi::scan_dir(&dir.path().join("missing"), ".txt").is_err());
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_filters() {
    let mut px = vec![
        255, 0, 0, 200, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 0,
    ];
    image::grayscale(&mut px);
    assert_eq!(
        px,
        [
            76, 76, 76, 200, 150, 150, 150, 255, 29, 29, 29, 255, 255, 255, 255, 0
        ],
        "alpha untouched"
    );
    image::invert(&mut px);
    assert_eq!(&px[..4], [179, 179, 179, 200]);
    let flat = vec![10u8; 3 * 3 * 4];
    assert_eq!(
        image::box_blur(&flat, 3, 3),
        flat,
        "a flat image is unchanged"
    );
    let dot = {
        let mut v = vec![0u8; 3 * 3 * 4];
        v[4 * 4..4 * 4 + 4].copy_from_slice(&[90, 90, 90, 90]);
        v
    };
    let blurred = image::box_blur(&dot, 3, 3);
    assert_eq!(
        &blurred[4 * 4..4 * 4 + 4],
        [10, 10, 10, 10],
        "the centre averages 9 pixels"
    );
    assert_eq!(&blurred[..4], [23, 23, 23, 23], "a corner averages 4");
}
