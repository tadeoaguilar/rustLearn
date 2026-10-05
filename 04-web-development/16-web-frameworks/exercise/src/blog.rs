//! Exercises 5 and 6: server-rendered HTML with askama, forms, static files.
//!
//! askama compiles templates (in `templates/`) into Rust code at *build*
//! time: a typo in a variable name is a compile error, and rendering is just
//! string formatting -- no template parsing at runtime. HTML is escaped
//! automatically, so a post titled `<script>` can't inject anything.

use askama::Template;
use axum::Router;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use tower_http::services::ServeDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub body: String,
}

impl Post {
    /// Templates can call methods on the values they're given.
    pub fn paragraphs(&self) -> Vec<&str> {
        todo!("Exercise 5")
    }

    pub fn word_count(&self) -> usize {
        todo!("Exercise 5")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Blog {
    posts: Arc<RwLock<Vec<Post>>>,
}

/// "Hello, World! 2024" -> "hello-world-2024"
pub fn slugify(title: &str) -> String {
    todo!("Exercise 5")
}

impl Blog {
    pub fn new() -> Self {
        todo!("Exercise 5")
    }

    /// Validates and stores; a repeated title gets "-2", "-3"... appended.
    pub fn publish(&self, title: &str, body: &str) -> Result<Post, String> {
        todo!("Exercise 5")
    }

    pub fn find(&self, slug: &str) -> Option<Post> {
        todo!("Exercise 5")
    }

    pub fn all(&self) -> Vec<Post> {
        todo!("Exercise 5")
    }
}

// One struct per template. The fields are what the template can see.
#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
    posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostPage {
    post: Post,
}

#[derive(Template)]
#[template(path = "new.html")]
struct NewPostPage {
    error: Option<String>,
    title: String,
    body: String,
}

/// Rendering can fail (a method in a template returned an error); turn that
/// into a 500 instead of panicking.
fn render(t: impl Template) -> Response {
    todo!("Exercise 5")
}

async fn index(State(blog): State<Blog>) -> Response {
    todo!("Exercise 5")
}

async fn show(State(blog): State<Blog>, Path(slug): Path<String>) -> Response {
    todo!("Exercise 5")
}

async fn new_form() -> Response {
    todo!("Exercise 5")
}

#[derive(Debug, Deserialize)]
pub struct PostForm {
    pub title: String,
    pub body: String,
}

/// `Form<T>` parses `application/x-www-form-urlencoded`. On success: 303
/// See Other, so a browser refresh doesn't resubmit the form
/// (Post/Redirect/Get). On failure: show the form again with the values the
/// user typed and a 422 status.
async fn create(State(blog): State<Blog>, Form(form): Form<PostForm>) -> Response {
    todo!("Exercise 5")
}

/// Exercise 6: `ServeDir` serves files from a directory, with content types,
/// `If-Modified-Since` and range requests handled for you. The path is
/// anchored at the crate root so it works whatever the current directory is.
pub fn static_files() -> ServeDir {
    todo!("Exercise 5")
}

pub fn router(blog: Blog) -> Router {
    todo!("Exercise 5")
}
