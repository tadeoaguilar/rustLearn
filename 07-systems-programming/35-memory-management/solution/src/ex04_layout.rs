//! Exercise 4: memory layout -- sizes, alignment, padding, niches.
//!
//! Every type has a size and an alignment (its address must be a multiple
//! of it). A struct's fields are placed in order (with `#[repr(C)]`) or in
//! any order the compiler likes (the default `repr(Rust)`, which reorders
//! to reduce padding). Padding bytes are wasted; in an array of a million
//! structs, they add up. And some types have *niches* -- bit patterns that
//! are never valid (a null reference, a zero `NonZeroU32`) -- which `Option`
//! uses to store `None` for free.

/// Where each field goes under `#[repr(C)]`, and the struct's size and
/// alignment. Fields are `(size, align)`. Each field starts at the next
/// multiple of its alignment; the struct's alignment is the largest field
/// alignment, and its size is rounded up to it (so arrays stay aligned).
pub fn layout_c(fields: &[(usize, usize)]) -> (Vec<usize>, usize, usize) {
    let mut offsets = Vec::with_capacity(fields.len());
    let mut offset: usize = 0;
    let mut align: usize = 1;
    for &(size, field_align) in fields {
        offset = offset.next_multiple_of(field_align);
        offsets.push(offset);
        offset += size;
        align = align.max(field_align);
    }
    (offsets, offset.next_multiple_of(align), align)
}

/// Total padding bytes in that layout.
pub fn padding(fields: &[(usize, usize)]) -> usize {
    let (_, size, _) = layout_c(fields);
    size - fields.iter().map(|(s, _)| s).sum::<usize>()
}

/// A field order with minimal padding: decreasing alignment (what rustc
/// does for `repr(Rust)` structs, roughly). Returns indices into `fields`.
pub fn best_order(fields: &[(usize, usize)]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..fields.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(fields[i].1));
    order
}

/// An example of a badly ordered struct: 1 + 7 padding + 8 + 1 + 7 padding.
#[repr(C)]
pub struct Padded {
    pub flag: bool,
    pub value: u64,
    pub tag: u8,
}

/// The same fields, ordered by alignment: 8 + 1 + 1 + 6 padding.
#[repr(C)]
pub struct Packed {
    pub value: u64,
    pub flag: bool,
    pub tag: u8,
}

/// Facts about niches the tests check against the compiler: `Option` of a
/// type with a niche is no bigger than the type.
pub fn option_is_free<T>() -> bool {
    std::mem::size_of::<Option<T>>() == std::mem::size_of::<T>()
}
