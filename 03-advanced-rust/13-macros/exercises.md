# Exercises: Macros

Macros are code that writes code. **Declarative macros** (`macro_rules!`)
match token patterns and expand to new tokens. **Procedural macros** are
Rust functions, compiled into a separate crate, that receive a token stream
and return another — `#[derive(...)]`, `#[attribute]` and `function_like!()`.

Reach for a macro only when a function or a generic can't do the job: variable
numbers of arguments, generating items (structs, impls), or checking things at
compile time.

**Setup**: procedural macros must live in their own crate with
`proc-macro = true`. In this module that's `exercise/derive/` (package
`m13-macros-derive`), already created with `syn`, `quote` and `proc-macro2`.

---

## Exercise 1: Your First `macro_rules!`

**Difficulty**: Easy
**Time**: 30 minutes

**Learning Objectives**:
- Write matchers with fragment specifiers (`$x:expr`, `$name:ident`, `$t:ty`)
- Use repetitions `$( ... ),*` and recursion
- Export macros with `#[macro_export]`

**Tasks**:
```rust
square!(4)                        // 16 -- evaluate the argument only ONCE
max!(3)                           // 3
max!(3, 9, 2, 7)                  // 9  -- any number of arguments
hashmap!{ "a" => 1, "b" => 2, }   // HashMap, trailing comma allowed
vec_of_strings!["a", "b"]         // vec!["a".to_string(), "b".to_string()]
```

**Why "only once"?** Write the naive version, `$x * $x`, and call
`square!(next())` where `next()` increments a counter. How many times does it
run?

**Hints**:
- Multiple rules are tried top to bottom; the first match wins
- Recursion: `($x:expr, $($rest:expr),+) => { ... max!($($rest),+) ... }`
- Inside an exported macro, call other macros as `$crate::max!` so it works from other crates

---

## Exercise 2: Generating Items

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Generate structs, enums and `impl` blocks
- Use `stringify!`, `$vis:vis` and a counting macro

**Task 1**: Newtypes
```rust
newtype!(pub Meters(f64));
// generates: a tuple struct deriving Debug, Clone, Copy, PartialEq, PartialOrd,
//            fn value(self) -> f64, impl From<f64>, impl Display (just the value)
let m: Meters = 3.5.into();
assert_eq!(m.value(), 3.5);
```

**Task 2**: String-backed enums
```rust
string_enum! {
    pub enum Color { Red, Green, Blue }
}
assert_eq!(Color::Green.as_str(), "Green");
assert_eq!("Blue".parse::<Color>(), Ok(Color::Blue));
assert_eq!(Color::ALL.len(), 3);
assert_eq!(Color::COUNT, 3);
```

**Task 3**: `count!(a b c) == 3` — count tokens at compile time (used by `COUNT` above).

---

## Exercise 3: Compile-Time Validation

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Turn bad input into a *compile* error instead of a runtime panic
- Combine macros with `const fn` and `const { }` blocks

**Tasks**:
```rust
const_assert!(std::mem::size_of::<u64>() == 8);   // fails the build if false

let n: NonZeroU32 = nonzero!(5);                   // nonzero!(0) doesn't compile

const ORANGE: Rgb = hex_color!("#ff8800");         // Rgb { r: 255, g: 136, b: 0 }
// hex_color!("#ff88")  -> compile error
// hex_color!("ff8800") -> compile error (no #)
```

Write `const fn parse_hex_color(s: &str) -> Result<Rgb, HexError>` first, then
make `hex_color!` call it inside a `const { }` block and `panic!` on `Err` —
a panic during constant evaluation is a compile error.

**Hints**:
- `const _: () = assert!(...);` evaluates at compile time
- In a `const fn` you can't use iterators or `?`; use `while` loops over `s.as_bytes()`
- Document the failing cases with ` ```compile_fail ` doc tests

---

## Exercise 4: A Domain-Specific Language

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Design a readable syntax and parse it with `macro_rules!`
- Generate matching code from a table

**Task**: a state-machine DSL.
```rust
state_machine! {
    pub machine DoorState on DoorEvent {
        states: [Open, Closed, Locked],
        events: [OpenDoor, CloseDoor, Lock, Unlock],
        transitions: {
            Open   + CloseDoor => Closed,
            Closed + OpenDoor  => Open,
            Closed + Lock      => Locked,
            Locked + Unlock    => Closed,
        }
    }
}

assert_eq!(DoorState::Open.next(DoorEvent::CloseDoor), Some(DoorState::Closed));
assert_eq!(DoorState::Open.next(DoorEvent::Lock), None);
assert_eq!(DoorState::Closed.accepted_events(), vec![DoorEvent::OpenDoor, DoorEvent::Lock]);
assert_eq!(DoorState::ALL.len(), 3);
```

Generate: both enums (with `Debug, Clone, Copy, PartialEq, Eq, Hash`),
`ALL`, `next` and `accepted_events`. Then define a second machine (a traffic
light) with the same macro.

**Hints**:
- `$from:ident + $ev:ident => $to:ident` is a valid matcher: `+` and `=>` are literal tokens
- `next` is a `match (self, event)` with one arm per transition and `_ => None`

---

## Exercise 5: Hygiene, `$crate` and Debugging Macros

**Difficulty**: Medium
**Time**: 30 minutes

**Learning Objectives**:
- Understand macro hygiene
- Use `stringify!`, `file!`, `line!`, `concat!`
- Expand macros with `cargo expand`

**Tasks**:
1. Hygiene: why does this fail?
   ```rust
   macro_rules! make_x { () => { let x = 42; }; }
   make_x!();
   println!("{x}");   // error[E0425]: cannot find value `x`
   ```
   Write `make_var!(name, value)` that *does* create a variable the caller can use.
2. `my_assert_eq!(a, b)` that panics with
   `` assertion failed: `a + 1 == b` (left: 3, right: 4) at src/file.rs:12 ``
3. `traced!(expr)` that evaluates `expr`, records `"file:line: expr = value"`
   in a log, and returns the value.
4. Install `cargo-expand` and look at what `hashmap!` and `#[derive(Debug)]` expand to:
   ```bash
   cargo install cargo-expand
   cargo expand -p m13-macros-solution ex01_basics
   ```

---

## Exercise 6: A Derive Macro

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Write a `#[proc_macro_derive]` with `syn` and `quote`
- Walk a struct's or enum's shape: named fields, tuple fields, unit, variants
- Support generic types with `split_for_impl`

**Task**: `#[derive(Describe)]` adds `fn describe() -> String` (and
`field_names()` for structs with named fields):

```rust
#[derive(Describe)]
struct Point { x: i32, y: i32 }
assert_eq!(Point::describe(), "struct Point { x: i32, y: i32 }");
assert_eq!(Point::field_names(), &["x", "y"]);

#[derive(Describe)]
enum Shape { Circle(f64), Square { side: f64 }, Empty }
assert_eq!(Shape::describe(), "enum Shape { Circle(f64), Square { side: f64 }, Empty }");

#[derive(Describe)]
struct Wrapper<T> { inner: T }     // generics must work
```

**Hints**:
```rust
#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let text = /* build the description from input.data */;
    quote::quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn describe() -> String { #text.to_string() }
        }
    }.into()
}
```
- `quote!(#ty).to_string()` renders a type with extra spaces (`Vec < String >`); tidy it
- Report unsupported input (unions) with `syn::Error::new_spanned(..).to_compile_error()`

---

## Exercise 7: A Builder Derive

**Difficulty**: Very Hard
**Time**: 90 minutes

**Learning Objectives**:
- Generate a whole new type from a struct definition
- Inspect field types (`Option<T>`) and helper attributes (`#[builder(default)]`)

**Task**: `#[derive(Builder)]` — the classic from
[dtolnay's proc-macro workshop](https://github.com/dtolnay/proc-macro-workshop).

```rust
#[derive(Builder, Debug, PartialEq)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    current_dir: Option<String>,   // optional: stays None if never set
    #[builder(default)]
    verbose: bool,                 // optional: Default::default() if never set
}

let cmd = Command::builder()
    .executable("cargo".to_string())
    .args(vec!["build".to_string()])
    .build()?;
assert_eq!(cmd.current_dir, None);

let err = Command::builder().build().unwrap_err();
assert_eq!(err, "missing field `executable`");
```

Generate:
- `struct CommandBuilder` with every field wrapped in `Option`
- `Command::builder()`
- one setter per field, taking the field type (`T`, not `Option<T>`, for optional fields) and returning `Self`
- `build(self) -> Result<Command, String>`

**Hints**:
- An `Option<T>` field: the type's last path segment is `Option` with one generic argument
- Read helper attributes with `field.attrs` and `attr.path().is_ident("builder")`, then `attr.parse_nested_meta`
- Register the helper attribute: `#[proc_macro_derive(Builder, attributes(builder))]`

---

## Bonus Challenge: An Attribute Macro

**Difficulty**: Hard
**Time**: 45 minutes

`#[timed]` wraps a function so it prints how long each call took, without
changing what it returns:

```rust
#[timed]
fn slow_sum(n: u64) -> u64 { (0..n).sum() }

#[timed("parsing")]
fn parse(s: &str) -> Result<i32, std::num::ParseIntError> {
    let n = s.parse()?;      // `?` and early `return` must keep working
    Ok(n)
}
// stderr: [timed] slow_sum took 1.2ms
//         [timed] parsing took 3.1µs
```

**Hints**:
- `#[proc_macro_attribute] pub fn timed(attr: TokenStream, item: TokenStream) -> TokenStream`
- Parse `item` as `syn::ItemFn`; keep `vis` and `sig`, wrap `block`
- Run the original body in a closure with the function's return type, so `return` and `?` still mean the same thing

---

## Check Your Understanding

- [ ] Write a `macro_rules!` with several rules, repetitions and recursion
- [ ] Explain why `$x * $x` is a bug in a macro
- [ ] Generate items with `macro_rules!`
- [ ] Turn invalid input into a compile error
- [ ] Explain macro hygiene and `$crate`
- [ ] Write derive and attribute macros with `syn` and `quote`
- [ ] Inspect expansions with `cargo expand`

---

## Additional Resources

- [Rust Book 19.5: Macros](https://doc.rust-lang.org/book/ch20-05-macros.html)
- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
- [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop)
- [syn](https://docs.rs/syn/) and [quote](https://docs.rs/quote/)
- [cargo-expand](https://github.com/dtolnay/cargo-expand)
