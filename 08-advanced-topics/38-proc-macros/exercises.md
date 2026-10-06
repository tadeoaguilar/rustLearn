# Exercises: Procedural Macros

Procedural macros are Rust functions that run inside the compiler: they
receive the tokens of your code and return new tokens. `serde`'s
`#[derive(Serialize)]`, `tokio::main`, `sqlx::query!` and `clap`'s derive
are all procedural macros. There are three kinds:

| Kind | Looks like | Receives |
|---|---|---|
| derive | `#[derive(ToJson)]` | the item; *adds* code after it |
| attribute | `#[memoize]` | the attribute's arguments and the item; *replaces* the item |
| function-like | `state_machine! { .. }` | whatever is inside the brackets |

A `proc-macro` crate can export only macros, so this module puts each
expansion in a normal library (`core/`), as a function from
`proc_macro2::TokenStream` to `syn::Result<TokenStream>`, and unit-tests it
directly. `derive/` is a thin wrapper; the main crate holds the runtime
traits the generated code implements.

**Setup**: `syn` (parsing), `quote` (generating), `proc-macro2`. Optional:
`cargo install cargo-expand` to see what a macro generates.

---

## Exercise 1: A Serialization Derive

**Difficulty**: Medium
**Time**: 2 hours

**Learning Objectives**:
- Parse a `DeriveInput` and walk structs and enums
- Generate an impl with `quote!`, including generics
- Support helper attributes and reject typos

1. `#[derive(ToJson)]` for structs with named fields: a JSON object with
   the fields in declaration order, each serialized through `ToJson`.
2. `#[json(rename = "name")]` and `#[json(skip)]` on fields (use the
   provided `attrs::parse`, which rejects unknown options).
3. Tuple structs -> arrays; unit structs -> `null`.
4. Enums: unit variants -> `"Name"` (or the rename); variants with fields
   -> `{"Name": <object or array>}`.
5. Generic type parameters get a `ToJson` bound.
6. Compile errors for unions and empty enums.

**Question**: the generated code refers to `::m38_proc_macros_solution::json::ToJson`
by its absolute path. Why not just `ToJson`? And why does the runtime crate
need `extern crate self as m38_proc_macros_solution;`?

---

## Exercise 2: An ORM Derive

**Difficulty**: Medium
**Time**: 2 hours

**Learning Objectives**:
- Generate code that the *type checker* validates (field types -> SQL types)
- Report errors with the right span and a helpful message

1. `#[derive(Table)]` implements `orm::Table`: `TABLE`, `columns()`,
   `values()`, `from_row()`. The provided trait builds `CREATE TABLE`,
   `INSERT` and `SELECT` from those.
2. `#[table(name = "..")]`; by default snake_case plus `s`
   (`BlogPost` -> `blog_posts`).
3. `#[column(primary_key)]` (exactly one), `#[column(name = "..")]`,
   `#[column(skip)]` (rebuilt with `Default`).
4. A column's SQL type and nullability come from `<Type as SqlType>`, so an
   unsupported field type fails to compile *at that field*.
5. Errors: no or two primary keys, a skipped primary key, tuple structs,
   enums, generics.

The tests run the generated SQL in an in-memory SQLite.

---

## Exercise 3: A DSL

**Difficulty**: Hard
**Time**: 2 hours

**Learning Objectives**:
- Implement `syn::parse::Parse` for your own grammar
- Use custom keywords and delimited groups
- Generate several items from one invocation

```text
state_machine! {
    machine Door {
        initial Closed;
        Closed -> Open on Push;
        Open -> Closed on Pull;
    }
}
```

1. Parse it into `Machine { name, initial, transitions }`.
2. Errors: missing `initial` first, no transitions, a second transition for
   the same state and event (pointing at the event).
3. Generate `DoorState`, `DoorEvent` (variants in order of first
   appearance, initial first), `DoorError { state, event }` with
   `Display` and `Error`, and `Door` with `new`, `Default`, `state`,
   `can_fire`, `fire` and `TRANSITIONS`.

---

## Exercise 4: An Attribute Macro

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Rewrite a function while keeping its signature and attributes
- Know what an attribute macro may and may not assume

`#[memoize]` caches a function's results per thread, keyed by its
arguments; recursive calls go through the cache too. It also generates
`<name>_cache_len()`.

Errors: arguments to the attribute, methods, `async`, generics, no return
type, non-identifier argument patterns.

**Question**: the original body runs inside a closure. What breaks if it is
pasted in directly? And why must no `RefCell` borrow be held while it runs?

---

## Bonus: `#[derive(EnumIter)]`

**Difficulty**: Easy
**Time**: 30 minutes

For enums of unit variants: `COUNT`, `ALL`, `name()`, `from_name()`. A
variant with fields is an error at that variant.
