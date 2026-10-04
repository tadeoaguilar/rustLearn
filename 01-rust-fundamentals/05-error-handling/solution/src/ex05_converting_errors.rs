//! Exercise 5: Converting Between Error Types.
//!
//! One function, two failure types (io::Error, ParseIntError). `?` needs a
//! single return type that both convert into. Two ways:

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::num::ParseIntError;
use std::path::Path;

/// Task 1: `Box<dyn Error>` -- any error converts into it. Easy, but callers
/// can only print the error or try to downcast it.
pub fn read_number_boxed(path: &Path) -> Result<i32, Box<dyn Error>> {
    let mut file = File::open(path)?; // io::Error -> Box<dyn Error>
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let number: i32 = contents.trim().parse()?; // ParseIntError -> Box<dyn Error>
    Ok(number)
}

/// Task 2: an enum that wraps each source error. Callers can match on it.
#[derive(Debug)]
pub enum MyError {
    Io(io::Error),
    Parse(ParseIntError),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Io(e) => write!(f, "IO error: {e}"),
            MyError::Parse(e) => write!(f, "Parse error: {e}"),
        }
    }
}

/// `source()` exposes the wrapped error, so tools that walk error chains
/// (anyhow's `{:#}`, logging libraries) can show the underlying cause.
impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyError::Io(e) => Some(e),
            MyError::Parse(e) => Some(e),
        }
    }
}

/// These two impls are what make `?` work: it calls `From::from(err)`.
impl From<io::Error> for MyError {
    fn from(error: io::Error) -> Self {
        MyError::Io(error)
    }
}

impl From<ParseIntError> for MyError {
    fn from(error: ParseIntError) -> Self {
        MyError::Parse(error)
    }
}

pub fn read_number(path: &Path) -> Result<i32, MyError> {
    let contents = std::fs::read_to_string(path)?;
    let number: i32 = contents.trim().parse()?;
    Ok(number)
}

pub fn run() {
    let dir = std::env::temp_dir();
    let good = dir.join("rustlearn-m05-number.txt");
    let bad = dir.join("rustlearn-m05-not-a-number.txt");
    let missing = dir.join("rustlearn-m05-missing.txt");
    std::fs::write(&good, "42\n").unwrap();
    std::fs::write(&bad, "forty-two").unwrap();

    for path in [&good, &bad, &missing] {
        let name = path.file_name().unwrap().to_string_lossy();
        match read_number(path) {
            Ok(n) => println!("{name}: {n}"),
            Err(MyError::Io(e)) => println!("{name}: could not read ({e})"),
            Err(MyError::Parse(e)) => println!("{name}: bad contents ({e})"),
        }
        println!(
            "{name} (boxed): {:?}",
            read_number_boxed(path).map_err(|e| e.to_string())
        );
    }
    let _ = std::fs::remove_file(good);
    let _ = std::fs::remove_file(bad);
}
