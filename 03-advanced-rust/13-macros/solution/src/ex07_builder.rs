//! Exercise 7: A builder derive -- see `derive/src/lib.rs` for `Builder`.
//!
//! What `#[derive(Builder)]` generates for `Command`, roughly:
//!
//! ```text
//! pub struct CommandBuilder { executable: Option<String>, args: Option<Vec<String>>,
//!                             current_dir: Option<String>, verbose: Option<bool> }
//! impl Command { pub fn builder() -> CommandBuilder { ... all None ... } }
//! impl CommandBuilder {
//!     pub fn executable(mut self, value: String) -> Self { ... }
//!     pub fn current_dir(mut self, value: String) -> Self { ... }   // String, not Option<String>
//!     ...
//!     pub fn build(self) -> Result<Command, String> { ... }
//! }
//! ```

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
    let cmd = Command::builder()
        .executable("cargo".to_string())
        .args(vec!["build".to_string(), "--release".to_string()])
        .current_dir("/tmp".to_string())
        .build();
    println!("{cmd:#?}");
    println!(
        "missing field -> {:?}",
        Command::builder().args(vec![]).build()
    );
}
