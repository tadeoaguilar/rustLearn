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
        self.body
            .split("\n\n")
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect()
    }

    pub fn word_count(&self) -> usize {
        self.body.split_whitespace().count()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Blog {
    posts: Arc<RwLock<Vec<Post>>>,
}

/// "Hello, World! 2024" -> "hello-world-2024"
pub fn slugify(title: &str) -> String {
    title
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}

impl Blog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates and stores; a repeated title gets "-2", "-3"... appended.
    pub fn publish(&self, title: &str, body: &str) -> Result<Post, String> {
        let title = title.trim();
        if title.is_empty() {
            return Err("Title is required.".into());
        }
        let base = slugify(title);
        if base.is_empty() {
            return Err("Title needs at least one letter or digit.".into());
        }
        if body.trim().is_empty() {
            return Err("Body is required.".into());
        }
        let mut posts = self.posts.write().unwrap();
        let mut slug = base.clone();
        let mut n = 1;
        while posts.iter().any(|p| p.slug == slug) {
            n += 1;
            slug = format!("{base}-{n}");
        }
        let post = Post {
            slug,
            title: title.to_string(),
            body: body.trim().to_string(),
        };
        posts.push(post.clone());
        Ok(post)
    }

    pub fn find(&self, slug: &str) -> Option<Post> {
        self.posts
            .read()
            .unwrap()
            .iter()
            .find(|p| p.slug == slug)
            .cloned()
    }

    pub fn all(&self) -> Vec<Post> {
        self.posts.read().unwrap().clone()
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
    match t.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("template error: {e}"),
        )
            .into_response(),
    }
}

async fn index(State(blog): State<Blog>) -> Response {
    render(IndexPage { posts: blog.all() })
}

async fn show(State(blog): State<Blog>, Path(slug): Path<String>) -> Response {
    match blog.find(&slug) {
        Some(post) => render(PostPage { post }),
        None => (
            StatusCode::NOT_FOUND,
            Html("<h1>No such post</h1>".to_string()),
        )
            .into_response(),
    }
}

async fn new_form() -> Response {
    render(NewPostPage {
        error: None,
        title: String::new(),
        body: String::new(),
    })
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
    match blog.publish(&form.title, &form.body) {
        Ok(post) => Redirect::to(&format!("/blog/{}", post.slug)).into_response(),
        Err(message) => {
            let page = NewPostPage {
                error: Some(message),
                title: form.title,
                body: form.body,
            };
            (StatusCode::UNPROCESSABLE_ENTITY, render(page)).into_response()
        }
    }
}

/// Exercise 6: `ServeDir` serves files from a directory, with content types,
/// `If-Modified-Since` and range requests handled for you. The path is
/// anchored at the crate root so it works whatever the current directory is.
pub fn static_files() -> ServeDir {
    ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static"))
}

pub fn router(blog: Blog) -> Router {
    Router::new()
        .route("/blog", get(index).post(create))
        .route("/blog/new", get(new_form))
        .route("/blog/{slug}", get(show))
        .nest_service("/static", static_files())
        .with_state(blog)
}
