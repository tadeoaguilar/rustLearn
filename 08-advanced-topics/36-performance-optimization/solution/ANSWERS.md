# Answers · 36 Performance Optimization

Numbers below were measured on the machine this module was built on (an
x86-64 Mac with AVX2), release builds, best of several runs. Yours will
differ; the ratios matter.

## Exercise 1: SipHash versus FNV

`top_words` on 1,000,000 words: naive 90 ms, fast 23 ms (3.9x).

The default hasher, SipHash-1-3, is designed so an attacker who controls
the keys can't make many of them collide -- otherwise anyone sending a web
server crafted form field names could turn its `HashMap` into a linked list
(hash-flooding DoS). That protection costs a few rounds of mixing per key.
FNV-1a is one xor and one multiply per byte: much faster for short keys,
and trivially attackable. Keep the default whenever keys come from outside
(HTTP headers, JSON object keys, user names); switch only for keys you
generate or trust. `ahash` and `rustc-hash` (FxHash) are popular faster choices.

The other two fixes mattered as much: the naive version allocated a
lowercase `String` for every one of the million words, and sorted every
distinct word to keep five.

## Exercise 2: Why `dot_scalar` isn't vectorized

Floating-point addition isn't associative: `(a + b) + c` can differ from
`a + (b + c)` in the last bit. Vectorizing a sum means adding in a
different order (eight partial sums, combined at the end), so the compiler
won't do it unless told the order doesn't matter (fast-math, which Rust
doesn't enable). Eight accumulators *write* that order in the source, and
LLVM vectorizes the result.

Measured: dot of 1M floats -- scalar 934 us, unrolled 169 us (5.5x),
AVX2+FMA intrinsics 128 us (7.3x). Counting a byte in 16 MiB -- scalar
4.0 ms, SSE2 1.8 ms, SWAR 1.2 ms. (The scalar `filter().count()` isn't
vectorized well; SWAR's plain arithmetic is.) Matrix multiply 512x512 --
i,j,k 195 ms, i,k,j 63 ms, transposed 68 ms, tiled 66 ms: loop order alone
is 3x. At this size tiling doesn't beat i,k,j yet; it pays off when
matrices outgrow the L2/L3 caches.

## Exercise 4: What we found

Random data: branchy 2.9 ms, branchless 3.5 ms. Sorted data: about the
same. The textbook result -- branchy code fast on sorted data and slow on
random data -- didn't appear, because LLVM compiled the "branchy" loop into
branch-free (vectorized compare-and-add) code itself. Likewise
`sum_indexed` and `sum_iter` ran at the same speed: the compiler proved the
index in range and removed the bounds checks. The lesson isn't that these
techniques are useless -- with a branch the compiler can't convert (a call
inside it, a data-dependent loop exit), they matter a lot -- but that you
must **measure and read the assembly** before rewriting code for speed.

## Exercise 5: Binary size

| Profile | Bytes |
|---|---|
| release | 550,232 |
| + `strip = true` | 438,112 (-20%) |
| + `panic = "abort"` | 429,696 |
| + `lto = true`, `codegen-units = 1` | 382,696 |
| + `opt-level = "z"` (= `min-size`) | 337,728 (-39% overall) |

Most of what remains is the standard library's formatting and I/O machinery
(`println!` and friends). Getting below 50% means giving some of that up:
`opt-level = "s"` vs `"z"` experiments, avoiding `format!`-heavy code, and
on nightly `-Z build-std=std,panic_abort -Z
build-std-features=panic_immediate_abort`, which rebuilds std for size and
drops panic messages. `cargo bloat` (`cargo install cargo-bloat`) shows
which functions take the space. Each step trades something: debuggability
(stripped symbols), unwinding (`panic = "abort"` means no `catch_unwind`, no
cleanup on panic), build time (LTO), speed (`opt-level = "z"`).
