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
        "Logger"
    }
    fn execute(&self) -> String {
        format!("Logging at level: {}", self.log_level)
    }
}

impl Plugin for CachePlugin {
    fn name(&self) -> &str {
        "Cache"
    }
    fn execute(&self) -> String {
        format!("Cache size: {} MB", self.cache_size)
    }
}

/// Any closure can be a plugin too -- handy for tests and one-offs.
pub struct FnPlugin<F: Fn() -> String> {
    pub name: String,
    pub run: F,
}

impl<F: Fn() -> String> Plugin for FnPlugin<F> {
    fn name(&self) -> &str {
        &self.name
    }
    fn execute(&self) -> String {
        (self.run)()
    }
}

#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// Runs every plugin in registration order; returns "Name: report" lines.
    pub fn run_all(&self) -> Vec<String> {
        self.plugins
            .iter()
            .map(|p| format!("{}: {}", p.name(), p.execute()))
            .collect()
    }
}

pub fn run() {
    let mut manager = PluginManager::new();
    manager.register(Box::new(LoggerPlugin {
        log_level: "INFO".to_string(),
    }));
    manager.register(Box::new(CachePlugin { cache_size: 128 }));
    manager.register(Box::new(FnPlugin {
        name: "Clock".into(),
        run: || "tick".to_string(),
    }));
    for line in manager.run_all() {
        println!("Running plugin {line}");
    }
    println!(
        "size of &LoggerPlugin = {} bytes, &dyn Plugin = {} bytes (data + vtable)",
        std::mem::size_of::<&LoggerPlugin>(),
        std::mem::size_of::<&dyn Plugin>()
    );
}
