//! Exercise 6: Trait Objects -- a plugin system.
//!
//! `Vec<Box<dyn Plugin>>` holds *different* types behind one interface. Each
//! `Box<dyn Plugin>` is a fat pointer: (data pointer, vtable pointer). Calls
//! look up the method in the vtable at runtime.
//!
//! Change from the exercise: `execute` returns its report as a String instead
//! of printing, so the manager -- and the tests -- can see what happened.

pub trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self) -> String;
}

pub struct LoggerPlugin {
    pub log_level: String,
}

pub struct CachePlugin {
    pub cache_size: usize,
}

impl Plugin for LoggerPlugin {
    fn name(&self) -> &str {
        todo!("Exercise 6")
    }
    fn execute(&self) -> String {
        todo!("Exercise 6")
    }
}

impl Plugin for CachePlugin {
    fn name(&self) -> &str {
        todo!("Exercise 6")
    }
    fn execute(&self) -> String {
        todo!("Exercise 6")
    }
}

/// Any closure can be a plugin too -- handy for tests and one-offs.
pub struct FnPlugin<F: Fn() -> String> {
    pub name: String,
    pub run: F,
}

impl<F: Fn() -> String> Plugin for FnPlugin<F> {
    fn name(&self) -> &str {
        todo!("Exercise 6")
    }
    fn execute(&self) -> String {
        todo!("Exercise 6")
    }
}

#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        todo!("Exercise 6")
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        todo!("Exercise 6")
    }

    pub fn names(&self) -> Vec<&str> {
        todo!("Exercise 6")
    }

    /// Runs every plugin in registration order; returns "Name: report" lines.
    pub fn run_all(&self) -> Vec<String> {
        todo!("Exercise 6")
    }
}

pub fn run() {
    todo!("Exercise 6")
}
