//! Exercise 3: The ? Operator.
//!
//! `expr?` means: if `expr` is `Err(e)`, return `Err(From::from(e))` from the
//! current function right now; otherwise unwrap the `Ok` value. The four
//! versions below behave identically.

use std::fs::File;
use std::io::{self, Read};
use std::num::ParseIntError;
use std::path::Path;

/// v1: manual propagation. (Clippy's `question_mark` lint suggests exactly
/// the rewrite that v2 makes.)
#[allow(clippy::question_mark)]
pub fn read_username_v1(path: &Path) -> Result<String, io::Error> {
    let f = File::open(path);
    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}

/// v2: `?` replaces each match.
pub fn read_username_v2(path: &Path) -> Result<String, io::Error> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

/// v3: chained.
pub fn read_username_v3(path: &Path) -> Result<String, io::Error> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    Ok(s)
}

/// v4: the standard library already has this function.
pub fn read_username_v4(path: &Path) -> Result<String, io::Error> {
    std::fs::read_to_string(path)
}

/// Task 2.
pub fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {
    let n: i32 = s.parse()?;
    Ok(n * 2)
}

pub fn sum_from_strings(s1: &str, s2: &str) -> Result<i32, ParseIntError> {
    let n1: i32 = s1.parse()?;
    let n2: i32 = s2.parse()?;
    Ok(n1 + n2)
}

pub fn run() {
    let path = std::env::temp_dir().join("rustlearn-m05-username.txt");
    std::fs::write(&path, "ferris").expect("can write to the temp dir");
    println!("v1: {:?}", read_username_v1(&path));
    println!("v2: {:?}", read_username_v2(&path));
    println!("v3: {:?}", read_username_v3(&path));
    println!("v4: {:?}", read_username_v4(&path));
    let _ = std::fs::remove_file(&path);
    // The error case: io::Error's Display includes the OS reason.
    if let Err(e) = read_username_v2(&path) {
        println!("missing file: {e} (kind: {:?})", e.kind());
    }

    println!("parse_and_double(\"21\") = {:?}", parse_and_double("21"));
    match sum_from_strings("10", "20") {
        Ok(sum) => println!("Sum: {sum}"),
        Err(e) => println!("Parse error: {e}"),
    }
    match sum_from_strings("10", "twenty") {
        Ok(sum) => println!("Sum: {sum}"),
        Err(e) => println!("Parse error: {e}"),
    }
}
