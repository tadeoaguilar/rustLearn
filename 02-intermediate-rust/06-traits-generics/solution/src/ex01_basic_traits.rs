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
            format!("{} by {}", self.title, self.author)
        }
    }

    impl Summary for Tweet {
        fn summarize(&self) -> String {
            format!("@{}: {}", self.username, self.content)
        }
    }

    /// `impl Summary` in argument position: "any type that implements Summary".
    /// Sugar for `fn notify<T: Summary>(item: &T)`.
    pub fn notify(item: &impl Summary) -> String {
        format!("Breaking news! {}", item.summarize())
    }
}

/// Task 2: a default method built on a required one. Implementors only have
/// to provide `summarize_author`; they *may* override `summarize`.
pub mod task2 {
    use super::{Article, Tweet};

    pub trait Summary {
        fn summarize_author(&self) -> String;

        fn summarize(&self) -> String {
            format!("(Read more from {}...)", self.summarize_author())
        }
    }

    impl Summary for Tweet {
        fn summarize_author(&self) -> String {
            format!("@{}", self.username)
        }
        // uses the default summarize
    }

    impl Summary for Article {
        fn summarize_author(&self) -> String {
            self.author.clone()
        }

        // overrides the default
        fn summarize(&self) -> String {
            format!("{}, by {}", self.title, self.summarize_author())
        }
    }
}

pub fn sample_article() -> Article {
    Article {
        title: "Rust is Great".to_string(),
        author: "Alice".to_string(),
        content: "Rust provides...".to_string(),
    }
}

pub fn sample_tweet() -> Tweet {
    Tweet {
        username: "bob".to_string(),
        content: "Learning Rust!".to_string(),
        reply: false,
    }
}

pub fn run() {
    let (article, tweet) = (sample_article(), sample_tweet());
    {
        use task1::Summary;
        println!("task 1: {}", article.summarize());
        println!("task 1: {}", tweet.summarize());
        println!("task 1: {}", task1::notify(&tweet));
    }
    {
        use task2::Summary;
        println!("task 2: {}", article.summarize());
        println!("task 2: {}", tweet.summarize());
    }
}
