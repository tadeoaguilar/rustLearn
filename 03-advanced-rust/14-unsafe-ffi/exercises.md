# Exercises: Unsafe Rust & FFI

`unsafe` doesn't turn off the borrow checker. It unlocks five extra abilities —
dereferencing raw pointers, calling `unsafe` functions (including foreign
ones), implementing `unsafe` traits, accessing mutable statics, and accessing
union fields — and makes **you** responsible for the rules the compiler can no
longer check. The goal is always the same: a small `unsafe` core, wrapped in a
**safe API** that can't be misused, with every `unsafe` block justified by a
`// SAFETY:` comment.

**Setup**: this module includes a small C library, `csrc/shapes.c`, compiled
by a build script with the [`cc`](https://docs.rs/cc) crate. You need a C
compiler: Xcode Command Line Tools on macOS (`xcode-select --install`),
`build-essential` on Debian/Ubuntu. The C code is provided; you write the Rust.

**Edition 2024 notes** — this workspace uses Rust 2024, which tightened the rules:
- `extern` blocks must be written `unsafe extern "C" { ... }`; items in them
  can be declared `safe fn` when calling them can never cause UB
- `#[no_mangle]` is written `#[unsafe(no_mangle)]`
- inside an `unsafe fn`, unsafe operations still need an `unsafe { }` block

---

## Exercise 1: Raw Pointers

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Create raw pointers safely; dereference them only in `unsafe`
- Use pointer arithmetic and `slice::from_raw_parts_mut`

**Tasks**:
1. `swap_via_pointers(a: &mut i32, b: &mut i32)` using raw pointers and `ptr::swap`
2. `sum_with_pointer_arithmetic(slice: &[i32]) -> i32` walking with `ptr.add(i)`
3. The Rust Book classic — implement `split_at_mut` yourself:
   ```rust
   pub fn my_split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]);
   ```
   Explain why the obvious safe version doesn't compile:
   ```rust
   (&mut slice[..mid], &mut slice[mid..])   // error[E0499]
   ```

**Hints**:
- Creating a raw pointer is safe: `let p = &mut x as *mut i32;` or `&raw mut x`
- `my_split_at_mut` must `assert!(mid <= len)` *before* any unsafe code — the
  safe API's job is to make the unsafe part impossible to misuse

---

## Exercise 2: Calling C — the C Standard Library

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Declare foreign functions with `unsafe extern "C"`
- Convert strings with `CString` / `CStr`
- Pass a Rust function as a C callback

**Tasks**:
```rust
unsafe extern "C" {
    safe fn abs(x: c_int) -> c_int;                 // can't cause UB: mark it safe
    fn strlen(s: *const c_char) -> usize;           // reads until NUL: unsafe
    fn qsort(base: *mut c_void, n: usize, size: usize,
             compar: extern "C" fn(*const c_void, *const c_void) -> c_int);
}
```
1. `c_abs(x: i32) -> i32`
2. `c_strlen(s: &str) -> Result<usize, NulError>` — strings with an interior NUL can't be C strings
3. `c_sort(values: &mut [i32])` using `qsort` and an `extern "C" fn` comparator

**Questions**:
- Why does `CString::new("a\0b")` fail?
- What goes wrong if the comparator panics? (Look up `extern "C-unwind"`.)

---

## Exercise 3: Bindings to Your Own C Library

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Match C types and struct layouts with `#[repr(C)]`
- Wrap an opaque C handle in a Rust type with `Drop`
- Pass closures to C through `void *user_data`

The C side (`csrc/shapes.h`):
```c
typedef struct { double x; double y; } Point;
double  shapes_distance(Point a, Point b);
double  shapes_polygon_area(const Point *points, size_t len);

typedef struct Counter Counter;            /* opaque */
Counter *counter_new(const char *name);
void     counter_add(Counter *c, int64_t n);
int64_t  counter_get(const Counter *c);
size_t   counter_name(const Counter *c, char *buf, size_t buf_len);
void     counter_free(Counter *c);

void shapes_for_each_point(const Point *points, size_t len,
                           void (*cb)(Point p, void *user_data), void *user_data);
```

**Tasks**:
1. Declare the bindings in `ffi` (a private module). `#[repr(C)] struct Point`.
2. Safe wrappers: `distance(Point, Point) -> f64`, `polygon_area(&[Point]) -> f64`
3. `struct Counter { ptr: NonNull<ffi::Counter> }` with `new(&str)`, `add`, `get`,
   `name() -> String`, and `impl Drop` calling `counter_free`. Should it be
   `Send`? `Sync`?
4. `for_each_point(points: &[Point], f: impl FnMut(Point))` — the closure goes
   through `user_data` as `*mut c_void`, and a generic `extern "C" fn
   trampoline<F: FnMut(Point)>` turns it back into `&mut F`.

---

## Exercise 4: Calling Rust from C

**Difficulty**: Medium
**Time**: 30 minutes

**Learning Objectives**:
- Export Rust functions with a C ABI
- Understand who frees memory allocated on the other side

**Tasks**:
1. Export
   ```rust
   #[unsafe(no_mangle)]
   pub unsafe extern "C" fn rust_checksum(data: *const u8, len: usize) -> u32;
   ```
   Why `unsafe fn`? It dereferences a pointer it can't validate. As a *safe*
   `pub fn`, any safe Rust code could pass a dangling pointer and cause UB —
   clippy rejects that (`not_unsafe_ptr_arg_deref`).
   The C library's `shapes_checksum_twice` calls it — call that from Rust and
   check the result.
2. Export `unsafe extern "C" fn rust_greeting(name: *const c_char) -> *mut c_char` returning a newly
   allocated C string, and `rust_free_string(s: *mut c_char)`. Why must the C
   side call *your* free function instead of `free()`?

---

## Exercise 5: A Safe Abstraction — `StackVec`

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Work with uninitialised memory through `MaybeUninit<T>`
- Keep an invariant ("the first `len` slots are initialised") that makes the unsafe code sound
- Drop exactly the initialised elements

**Task**: a vector with fixed capacity `N`, stored inline (no heap):

```rust
pub struct StackVec<T, const N: usize> {
    items: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    pub fn new() -> Self;
    pub fn push(&mut self, value: T) -> Result<(), T>;   // Err(value) when full
    pub fn pop(&mut self) -> Option<T>;
    pub fn as_slice(&self) -> &[T];
    pub fn as_mut_slice(&mut self) -> &mut [T];
    pub fn len(&self) -> usize;
}
impl<T, const N: usize> Drop for StackVec<T, N> { ... }  // drop the first len items
impl<T, const N: usize> Deref for StackVec<T, N> { type Target = [T]; ... }
```

**Test with a drop counter**: push 3 values that count their drops, pop one,
drop the vec — exactly 3 drops, never 2, never 4.

---

## Exercise 6: A Custom Allocator

**Difficulty**: Hard
**Time**: 45 minutes

**Learning Objectives**:
- Implement the `unsafe trait GlobalAlloc`
- Install an allocator with `#[global_allocator]`

**Task 1**: `CountingAllocator` wraps `std::alloc::System` and counts
allocations, deallocations and bytes currently in use with atomics:
```rust
pub struct CountingAllocator { allocations: AtomicUsize, deallocations: AtomicUsize, bytes: AtomicUsize }
unsafe impl GlobalAlloc for CountingAllocator { ... }
```
Install it in your binary (`src/main.rs`):
```rust
#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator::new();
```
and print how many allocations `vec![0; 1000]`, `String::from("hi")` and
`Vec::with_capacity` vs repeated `push` make.

**Task 2**: a bump arena — hand out string slices from big chunks, free
everything at once:
```rust
pub struct Arena { chunks: RefCell<Vec<Box<[u8]>>>, used: Cell<usize> }
impl Arena {
    pub fn alloc_str(&self, s: &str) -> &str;   // lives as long as the arena
}
```
Why is it sound to return `&str` tied to `&self` even though `chunks` is
mutated later? (What would break if a chunk were a `Vec<u8>` that grows?)

---

## Exercise 7: System Libraries

**Difficulty**: Medium
**Time**: 30 minutes

**Task**: call POSIX functions directly (Unix only — gate with `#[cfg(unix)]`):
```rust
unsafe extern "C" {
    safe fn getpid() -> c_int;
    fn gethostname(name: *mut c_char, len: usize) -> c_int;
}
pub fn process_id() -> u32;                 // compare with std::process::id()
pub fn hostname() -> std::io::Result<String>;   // errno via io::Error::last_os_error()
```

---

## Bonus Challenge: Hunting UB with Miri

**Difficulty**: Hard
**Time**: 45 minutes

[Miri](https://github.com/rust-lang/miri) runs your tests in an interpreter
that detects undefined behaviour: use-after-free, out-of-bounds reads,
uninitialised memory, aliasing violations.

```bash
rustup +nightly component add miri
cargo +nightly miri test -p m14-unsafe-ffi-tests -- ex1_ ex5_
```

1. Run Miri on your Exercise 1 and 5 tests (Miri can't call C, so skip FFI tests).
2. Break `StackVec::pop` deliberately (forget to decrement `len`, or read slot `len` instead of `len - 1`) and read Miri's report.
3. In `my_split_at_mut`, remove the `assert!` and call it with `mid > len`. What does Miri say?

---

## Check Your Understanding

- [ ] List the five things `unsafe` allows
- [ ] Write a safe function around an `unsafe` core, with `SAFETY:` comments
- [ ] Declare and call C functions; convert strings both ways
- [ ] Use `#[repr(C)]` and explain why it's needed
- [ ] Wrap a C resource in a type with `Drop`
- [ ] Work with `MaybeUninit<T>` without reading uninitialised memory
- [ ] Implement `GlobalAlloc`
- [ ] Run Miri

---

## Additional Resources

- [Rust Book 20.1: Unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Rustonomicon: FFI](https://doc.rust-lang.org/nomicon/ffi.html)
- [`std::ffi`](https://doc.rust-lang.org/std/ffi/index.html)
- [Miri](https://github.com/rust-lang/miri)
- [bindgen](https://rust-lang.github.io/rust-bindgen/) — generate bindings from C headers automatically
