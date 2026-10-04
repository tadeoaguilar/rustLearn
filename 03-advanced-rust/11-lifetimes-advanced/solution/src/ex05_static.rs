//! Exercise 5: `'static` and Lifetime Bounds.
//!
//! | Written           | Means                                                      |
//! |-------------------|------------------------------------------------------------|
//! | `&'static T`      | a reference valid for the whole program (literals, leaks, statics) |
//! | `T: 'static`      | `T` contains no *borrowed* data shorter than 'static. Owned types like `String`, `Vec<u8>`, `i32` all qualify -- they can be kept as long as you like |
//! | `T: 'a`           | any references inside `T` live at least as long as 'a     |

use std::fmt::Display;
use std::sync::OnceLock;

/// Task 1. `thread::spawn` requires `'static` because the new thread may run
/// longer than the function that started it; if the closure borrowed a local,
/// that local could be gone first. So:
///
/// ```text
/// let local = String::from("hi");
/// spawn_and_join(&local);   // error[E0597]: `local` does not live long enough
///                           //   ...argument requires that `local` is borrowed for `'static`
/// spawn_and_join(local);    // fine: the String is *moved* into the thread
/// ```
///
/// (`std::thread::scope` lifts this restriction by guaranteeing the threads
/// finish before the scope ends -- see module 15.)
pub fn spawn_and_join<T: Display + Send + 'static>(value: T) -> String {
    std::thread::spawn(move || format!("from another thread: {value}"))
        .join()
        .expect("thread panicked")
}

/// Task 2: compiles only if T: 'static. `is_static(&String::new())` works;
/// `is_static(&s.as_str())` with a local `s` doesn't.
pub fn is_static<T: 'static>(_: &T) {}

/// Task 3. `Box<dyn Trait>` in a struct field defaults to `Box<dyn Trait + 'static>`
/// -- closures that borrow locals wouldn't be allowed. `+ 'a` relaxes it to
/// "may borrow anything that lives at least as long as the Callbacks".
pub struct Callbacks<'a> {
    handlers: Vec<Handler<'a>>,
}

/// A type alias can carry a lifetime parameter too.
pub type Handler<'a> = Box<dyn Fn(&str) -> String + 'a>;

impl<'a> Callbacks<'a> {
    pub fn new() -> Self {
        Callbacks {
            handlers: Vec::new(),
        }
    }

    pub fn register(&mut self, f: impl Fn(&str) -> String + 'a) {
        self.handlers.push(Box::new(f));
    }

    pub fn fire(&self, event: &str) -> Vec<String> {
        self.handlers.iter().map(|h| h(event)).collect()
    }
}

impl Default for Callbacks<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Task 4: a global, initialised once on first use, readable from anywhere.
#[derive(Debug)]
pub struct Config {
    pub app_name: String,
    pub max_connections: u32,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config {
        app_name: std::env::var("APP_NAME").unwrap_or_else(|_| "rustlearn".to_string()),
        max_connections: 16,
    })
}

/// Another way to get a `&'static str` at runtime: leak it. The memory is
/// never freed -- fine for a handful of values set once at startup, a leak
/// if done in a loop.
pub fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn run() {
    let owned = String::from("an owned String");
    println!("{}", spawn_and_join(owned));
    println!("{}", spawn_and_join(42));

    let prefix = String::from("[app]");
    let mut callbacks = Callbacks::new();
    callbacks.register(|e| format!("{prefix} got {e}")); // borrows `prefix`
    callbacks.register(|e| e.to_uppercase());
    println!("{:?}", callbacks.fire("click"));

    println!("config: {:?}", config());
    let name: &'static str = leak_str(format!("built-{}", 7));
    println!("leaked: {name}");
}
