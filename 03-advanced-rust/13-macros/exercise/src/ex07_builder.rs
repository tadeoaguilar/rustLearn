//! Exercise 7: A builder derive. Write `Builder` in ../derive/src/lib.rs.

use crate::Builder;

#[derive(Builder, Debug, PartialEq)]
pub struct Command {
    pub executable: String,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
    #[builder(default)]
    pub verbose: bool,
}

pub fn run() {
    todo!("Exercise 7: build a Command with Command::builder()")
}
