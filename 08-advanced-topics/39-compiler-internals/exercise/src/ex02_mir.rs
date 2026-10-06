//! Exercise 2: reading MIR.
//!
//! MIR (Mid-level IR) is the compiler's representation after type checking
//! and before LLVM: each function is a control-flow graph of *basic blocks*
//! (`bb0`, `bb1`, ...), each a list of simple statements ending in one
//! terminator (`goto`, `switchInt`, a call, `return`, `assert`). The borrow
//! checker runs on MIR, and Rust's safety checks are visible in it:
//!
//! ```text
//! assert(move _13, "index out of bounds: ...") -> [success: bb7, ...];
//! assert(!move (_14.1: bool), "attempt to compute `{} + {}`, which would overflow", ...)
//! ```
//!
//! `parse_mir` summarizes the text `rustc --emit=mir` prints. The format is
//! unstable (it changes between compiler versions), so parse loosely:
//!
//! - a function starts at a line beginning with `fn ` (the name runs to the
//!   first `(`; closures look like `outer::{closure#0}`);
//! - a basic block is a line whose trimmed text starts with `bb` and ends with `{`
//!   (`bb3: {`, `bb7 (cleanup): {`);
//! - a bounds check is an `assert(` line mentioning `index out of bounds`;
//! - an overflow check is an `assert(` line mentioning `overflow`;
//! - a call is a line containing `-> [return:`.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MirFunction {
    pub name: String,
    pub basic_blocks: usize,
    pub bounds_checks: usize,
    pub overflow_checks: usize,
    pub calls: usize,
}

pub fn parse_mir(text: &str) -> Vec<MirFunction> {
    todo!("Exercise 2")
}

/// The summary for one function, by name.
pub fn find<'a>(functions: &'a [MirFunction], name: &str) -> Option<&'a MirFunction> {
    functions.iter().find(|f| f.name == name)
}
