# Exercises: Advanced Lifetimes

Lifetimes don't change how long anything lives. They are names for regions of
code that let you *tell the compiler* how the references in a signature relate
— so it can check every caller. Most of this module is about writing those
relationships down precisely, and reading the errors when they're wrong.

---

## Exercise 1: Annotations and Elision

**Difficulty**: Easy
**Time**: 30 minutes

**Learning Objectives**:
- Write lifetime annotations on functions
- Know the three elision rules and when you must annotate

**Task 1**: Which of these compile as written? Add annotations only where needed.
```rust
fn first_word(s: &str) -> &str { ... }
fn longest(x: &str, y: &str) -> &str { ... }
fn pick_first(x: &str, y: &str) -> &str { x }          // y is unrelated
fn longest_with_announcement<T: Display>(x: &str, y: &str, ann: T) -> &str { ... }
fn make_greeting(name: &str) -> String { ... }
```

**The elision rules**:
1. Each reference parameter gets its own lifetime.
2. If there is exactly one input lifetime, it's assigned to all outputs.
3. If one parameter is `&self` / `&mut self`, its lifetime goes to all outputs.

If the rules don't determine every output lifetime, you must annotate.

**Task 2**: `pick_first` returns `x`. Annotate it so this compiles:
```rust
let x = String::from("kept");
let result;
{
    let y = String::from("temporary");
    result = pick_first(&x, &y);
}
println!("{result}");   // y is gone, but result only borrows x
```

**Questions**:
- Why does `longest(&x, &y)` reject the same code?

---

## Exercise 2: Structs That Borrow

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Declare structs holding references
- Return references with the struct's lifetime, not `&self`'s

**Task**:
```rust
pub struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    /// The first sentence (up to and including the first '.', '!' or '?').
    pub fn first_sentence(novel: &'a str) -> Excerpt<'a>;
    pub fn text(&self) -> &'a str;            // note: &'a str, not &str
    pub fn word_count(&self) -> usize;
    pub fn longest_word(&self) -> &'a str;
}
```

**Why `-> &'a str` and not `-> &str`?** Elision rule 3 would tie the result to
`&self` — to the `Excerpt`, not to the novel. This must compile:

```rust
let novel = String::from("Call me Ishmael. Some years ago...");
let word;
{
    let excerpt = Excerpt::first_sentence(&novel);
    word = excerpt.longest_word();
}   // excerpt dropped -- but word points into `novel`, which is still alive
assert_eq!(word, "Ishmael");
```

**Task 2**: A struct with two references and two lifetimes:
```rust
pub struct Highlighter<'t, 'k> {
    text: &'t str,
    keyword: &'k str,
}
impl<'t, 'k> Highlighter<'t, 'k> {
    pub fn new(text: &'t str, keyword: &'k str) -> Self;
    /// Words of the text that contain the keyword, ignoring case.
    /// The results borrow from the text only -- the keyword can be dropped.
    pub fn matching_words(&self) -> Vec<&'t str>;
}
```

```rust
let text = String::from("Rustaceans love rust and trust it");
let found;
{
    let keyword = String::from("RUST");
    found = Highlighter::new(&text, &keyword).matching_words();
}   // keyword dropped
assert_eq!(found, vec!["Rustaceans", "rust", "trust"]);
```

With a single lifetime `Highlighter<'a>` for both fields, this fails. Why?

---

## Exercise 3: A Zero-Copy Parser

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Parse input into structures that borrow from it — no `String` allocation
- Tie error values to the input lifetime too

**Task**: Parse an HTTP request head into borrowed parts.

```rust
pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub version: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    pub body: &'a str,
}

pub enum ParseError<'a> {
    Empty,
    BadRequestLine(&'a str),
    BadHeader(&'a str),
}

pub fn parse_request<'a>(raw: &'a str) -> Result<Request<'a>, ParseError<'a>>;

impl<'a> Request<'a> {
    pub fn header(&self, name: &str) -> Option<&'a str>;   // case-insensitive
    pub fn query_params(&self) -> Vec<(&'a str, &'a str)>; // "?a=1&b=2"
}
```

**Test**:
```rust
let raw = "GET /search?q=rust&page=2 HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\n\r\n";
let req = parse_request(raw).unwrap();
assert_eq!(req.method, "GET");
assert_eq!(req.header("host"), Some("example.com"));
assert_eq!(req.query_params(), vec![("q", "rust"), ("page", "2")]);
// Zero-copy: req.method points into raw
assert_eq!(req.method.as_ptr(), raw.as_ptr());
```

**Hints**:
- Header and body are separated by an empty line (`\r\n\r\n`, but accept `\n\n` too)
- `split_once`, `strip_suffix`, `lines()` all return slices of the input

---

## Exercise 4: Iterators That Borrow

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Implement `Iterator` for a type with a lifetime
- Return `&'a T` items that outlive the call to `next`
- Write a mutable iterator without `unsafe`

**Task 1**: Adjacent pairs of a slice.
```rust
pub struct Pairs<'a, T> { slice: &'a [T], index: usize }
impl<'a, T> Iterator for Pairs<'a, T> {
    type Item = (&'a T, &'a T);
}
// [1, 2, 3] -> (1, 2), (2, 3)
```

**Task 2**: Words of a string, written by hand (no `split_whitespace`).
```rust
pub struct Words<'a> { rest: &'a str }
impl<'a> Iterator for Words<'a> { type Item = &'a str; }
```

**Task 3**: Every other element, *mutably*.
```rust
pub struct EveryOtherMut<'a, T> { slice: &'a mut [T] }
impl<'a, T> Iterator for EveryOtherMut<'a, T> { type Item = &'a mut T; }

let mut v = vec![1, 2, 3, 4, 5];
for x in EveryOtherMut::new(&mut v) { *x *= 10; }
assert_eq!(v, vec![10, 2, 30, 4, 50]);
```

**Hint for Task 3**: you can't hand out `&'a mut T` from `&mut self` while
keeping `self.slice` — take the slice out first:
`let slice = std::mem::take(&mut self.slice);` then `split_first_mut()`.

---

## Exercise 5: `'static` and Lifetime Bounds

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Tell `&'static T` from `T: 'static`
- Understand why `thread::spawn` and `Box<dyn Trait>` default to `'static`

**Tasks**:
1. Write `fn spawn_and_join<T: Display + Send + 'static>(value: T) -> String`
   that formats `value` on another thread. Then explain why
   `spawn_and_join(&local_string)` doesn't compile but `spawn_and_join(local_string)` does.
2. `String` is `'static` but `&'a str` is not. Prove it with
   `fn is_static<T: 'static>(_: &T) {}`.
3. Store closures in a struct, once requiring `'static` and once allowing borrows:
   ```rust
   pub struct Callbacks<'a> {
       handlers: Vec<Box<dyn Fn(&str) -> String + 'a>>,
   }
   ```
   Register a closure that borrows a local `prefix: &str`. What would happen
   with `Box<dyn Fn(&str) -> String>` (no `+ 'a`)?
4. A global configuration read from anywhere: `static CONFIG: OnceLock<Config>`
   with `fn config() -> &'static Config`.

---

## Exercise 6: Higher-Ranked Trait Bounds

**Difficulty**: Hard
**Time**: 40 minutes

**Learning Objectives**:
- Read and write `for<'a> Fn(&'a str) -> &'a str`
- Know when the compiler writes it for you

**Task 1**:
```rust
pub fn apply_to_all<F>(items: &[String], f: F) -> Vec<&str>
where
    F: for<'a> Fn(&'a str) -> &'a str;

apply_to_all(&names, |s| s.trim());
apply_to_all(&names, |s| s.split('@').next().unwrap_or(s));
```

Why can't this be `fn apply_to_all<'a, F: Fn(&'a str) -> &'a str>`?
(Hint: try calling `f` on a `String` created *inside* `apply_to_all`.)

**Task 2**: A pipeline of string-slicing steps:
```rust
pub struct SlicePipeline {
    steps: Vec<Box<dyn for<'a> Fn(&'a str) -> &'a str>>,
}
// .then(str::trim).then(|s| s.strip_prefix("re: ").unwrap_or(s)).run("  re: hello ")  == "hello"
```

---

## Exercise 7: Fix the Lifetime Errors

**Difficulty**: Hard
**Time**: 60 minutes

Each snippet fails to compile. Read the error, decide what the code *means*,
and fix it — sometimes with an annotation, sometimes by changing ownership.

**7a** — returning a reference to a local
```rust
fn longest_line(text: &str) -> &str {
    let upper = text.to_uppercase();
    upper.lines().max_by_key(|l| l.len()).unwrap_or("")
}
```

**7b** — a struct outliving its source
```rust
struct Config<'a> { name: &'a str }
fn load() -> Config<'static> {
    let raw = std::fs::read_to_string("app.conf").unwrap();
    Config { name: raw.trim() }
}
```

**7c** — conditional insert into a map (a known borrow-checker limitation)
```rust
fn get_or_default<'m>(map: &'m mut HashMap<String, String>, key: &str) -> &'m String {
    if let Some(v) = map.get(key) {
        return v;
    }
    map.insert(key.to_string(), String::new());
    map.get(key).unwrap()
}
```

**7d** — a getter tied to the wrong lifetime
```rust
struct Parser<'a> { input: &'a str, pos: usize }
impl<'a> Parser<'a> {
    fn next_token(&mut self) -> &str { ... }   // ties the token to &mut self
}
let mut p = Parser { input: "a b", pos: 0 };
let first = p.next_token();
let second = p.next_token();     // error: cannot borrow `p` as mutable more than once
println!("{first} {second}");
```

**7e** — a closure that outlives what it borrows
```rust
fn make_adder(n: &i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(|x| x + n)
}
```

---

## Bonus Challenge: `StrSplit` with Two Lifetimes

**Difficulty**: Very Hard
**Time**: 60 minutes

The classic from Jon Gjengset's "Crust of Rust: Lifetime Annotations":

```rust
pub struct StrSplit<'haystack, D> {
    remainder: Option<&'haystack str>,
    delimiter: D,
}

pub trait Delimiter {
    fn find_next(&self, s: &str) -> Option<(usize, usize)>;   // (start, end) of the match
}
impl Delimiter for &str { ... }
impl Delimiter for char { ... }

impl<'haystack, D: Delimiter> Iterator for StrSplit<'haystack, D> {
    type Item = &'haystack str;
}

pub fn until_char(s: &str, c: char) -> &str {
    StrSplit::new(s, c).next().expect("always at least one part")
}
```

First write it with a **single** lifetime for both haystack and delimiter
(`StrSplit<'a> { remainder: Option<&'a str>, delimiter: &'a str }`) and try
`until_char` with `&format!("{c}")` as the delimiter. Read the error. Then
explain why the generic `D` (or a second lifetime) fixes it.

---

## Check Your Understanding

- [ ] State the three elision rules from memory
- [ ] Annotate a struct that holds references, and its methods
- [ ] Return `&'a T` from a method instead of a reference tied to `&self`
- [ ] Implement an iterator that yields borrowed items
- [ ] Explain `T: 'static` vs `&'static T`
- [ ] Read a `for<'a>` bound out loud
- [ ] Fix a lifetime error by changing ownership, not only annotations

---

## Additional Resources

- [Rust Book 10.3: Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [Rustonomicon: Lifetimes](https://doc.rust-lang.org/nomicon/lifetimes.html)
- [Common Rust Lifetime Misconceptions](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md)
- [Crust of Rust: Lifetime Annotations](https://www.youtube.com/watch?v=rAl-9HwD858)
