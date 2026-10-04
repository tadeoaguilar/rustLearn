//! Exercise 1: Basic Traits.
//!
//! Tasks 1 and 2 both define a trait called `Summary` with different methods,
//! so each lives in its own module here.

#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub author: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
}

/// Task 1: every implementor writes its own `summarize`.
pub mod task1 {
    use super::{Article, Tweet};

    pub trait Summary {
        fn summarize(&self) -> String;
    }

    impl Summary for Article {
        fn summarize(&self) -> String {
            todo!("Exercise 1")
        }
    }

    impl Summary for Tweet {
        fn summarize(&self) -> String {
            todo!("Exercise 1")
        }
    }

    /// `impl Summary` in argument position: "any type that implements Summary".
    /// Sugar for `fn notify<T: Summary>(item: &T)`.
    pub fn notify(item: &impl Summary) -> String {
        todo!("Exercise 1")
    }
}

/// Task 2: a default method built on a required one. Implementors only have
/// to provide `summarize_author`; they *may* override `summarize`.
pub mod task2 {
    use super::{Article, Tweet};

    pub trait Summary {
        fn summarize_author(&self) -> String;

        fn summarize(&self) -> String {
            todo!("Exercise 1")
        }
    }

    impl Summary for Tweet {
        fn summarize_author(&self) -> String {
            todo!("Exercise 1")
        }
        // uses the default summarize
    }

    impl Summary for Article {
        fn summarize_author(&self) -> String {
            todo!("Exercise 1")
        }

        // overrides the default
        fn summarize(&self) -> String {
            todo!("Exercise 1")
        }
    }
}

pub fn sample_article() -> Article {
    todo!("Exercise 1")
}

pub fn sample_tweet() -> Tweet {
    todo!("Exercise 1")
}

pub fn run() {
    todo!("Exercise 1")
}
