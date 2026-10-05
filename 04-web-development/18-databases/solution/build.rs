// `sqlx::migrate!` embeds migrations/ at compile time, but on stable Rust
// Cargo isn't told to rebuild when a *new* file appears there. This line
// tells it to.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
