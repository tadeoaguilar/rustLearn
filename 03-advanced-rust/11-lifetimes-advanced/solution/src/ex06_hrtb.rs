//! Exercise 6: Higher-Ranked Trait Bounds.
//!
//! `for<'a> Fn(&'a str) -> &'a str` reads: "for *every* lifetime 'a, given a
//! &'a str this returns a &'a str" -- one closure that works for any borrow
//! you hand it, however short.

/// Why not `fn apply_to_all<'a, F: Fn(&'a str) -> &'a str>`? Then the
/// *caller* picks one 'a, and `f` only accepts strings living exactly that
/// long. Inside the function we create borrows the caller can't name --
/// `item.as_str()` on each element is a fresh short borrow; a `String` built
/// inside the function lives even less -- and `f` must accept all of them.
///
/// (For `Fn(&str) -> &str` written without names, the compiler inserts the
/// `for<'a>` itself. Writing it out is needed when you name the lifetime, and
/// in `dyn` types as in Task 2.)
pub fn apply_to_all<F>(items: &[String], f: F) -> Vec<&str>
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    // Demonstration that f works on a borrow that exists only in here:
    let local = String::from("  local  ");
    let _ = f(&local);

    items.iter().map(|s| f(s)).collect()
}

/// Task 2: boxed closures that slice their input. The HRTB is required in the
/// `dyn` type: each step must work for whatever borrow `run` passes in.
#[derive(Default)]
pub struct SlicePipeline {
    steps: Vec<SliceStep>,
}

/// One step: works for *any* input lifetime, returns a slice of its input.
pub type SliceStep = Box<dyn for<'a> Fn(&'a str) -> &'a str>;

impl SlicePipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn then(mut self, step: impl for<'a> Fn(&'a str) -> &'a str + 'static) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    /// The output borrows from `input` -- every step only narrows it.
    pub fn run<'s>(&self, input: &'s str) -> &'s str {
        self.steps.iter().fold(input, |s, step| step(s))
    }
}

pub fn run() {
    let names = vec!["  Ann ".to_string(), "bob@example.com".to_string()];
    println!("trimmed:  {:?}", apply_to_all(&names, str::trim));
    println!(
        "local part: {:?}",
        apply_to_all(&names, |s| s.split('@').next().unwrap_or(s))
    );

    let p = SlicePipeline::new()
        .then(str::trim)
        .then(|s| s.strip_prefix("re: ").unwrap_or(s))
        .then(|s| s.strip_suffix('!').unwrap_or(s));
    println!("pipeline: {:?}", p.run("  re: hello!  "));
}
