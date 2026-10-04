// Reference solution for 14-unsafe-ffi.
//
//     cargo run -p m14-unsafe-ffi-solution -- 3       # one exercise
//     cargo run -p m14-unsafe-ffi-solution -- all     # everything

use m14_unsafe_ffi_solution::ex06_allocator::CountingAllocator;
use m14_unsafe_ffi_solution::*;

/// Exercise 6: every allocation in this program goes through here.
#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator::new();

fn allocator_demo() {
    let measure = |label: &str, f: &dyn Fn()| {
        let (allocs, reallocs) = (ALLOC.allocations(), ALLOC.reallocations());
        f();
        println!(
            "{label:<42} {} alloc, {} realloc",
            ALLOC.allocations() - allocs,
            ALLOC.reallocations() - reallocs
        );
    };
    measure("vec![0u8; 1000]", &|| {
        drop(std::hint::black_box(vec![0u8; 1000]))
    });
    measure("String::from(\"hi\")", &|| {
        drop(std::hint::black_box(String::from("hi")))
    });
    measure("Vec::new() + 1000 pushes", &|| {
        let mut v = Vec::new();
        for i in 0..1000 {
            v.push(std::hint::black_box(i));
        }
    });
    measure("Vec::with_capacity(1000) + 1000 pushes", &|| {
        let mut v = Vec::with_capacity(1000);
        for i in 0..1000 {
            v.push(std::hint::black_box(i));
        }
    });
    println!("bytes in use right now: {}", ALLOC.bytes_in_use());
    ex06_allocator::run();
}

fn main() {
    let parts: [(&str, &str, fn()); 7] = [
        ("1", "Raw pointers", ex01_raw_pointers::run),
        ("2", "Calling libc", ex02_libc::run),
        ("3", "Bindings to a C library", ex03_c_library::run),
        ("4", "Calling Rust from C", ex04_rust_from_c::run),
        ("5", "StackVec with MaybeUninit", ex05_stack_vec::run),
        ("6", "Counting allocator and arena", allocator_demo),
        ("7", "POSIX system calls", ex07_system::run),
    ];

    let choice = std::env::args().nth(1).unwrap_or_default();
    match parts.iter().find(|(key, _, _)| *key == choice) {
        Some((key, title, run)) => {
            println!("=== EXERCISE {key}: {title} ===\n");
            run();
        }
        None if choice == "all" => {
            for (key, title, run) in parts {
                println!("\n=== EXERCISE {key}: {title} ===\n");
                run();
            }
        }
        None => {
            println!("14-unsafe-ffi -- reference solution\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m14-unsafe-ffi-solution -- {key:<6} {title}");
            }
            println!("  cargo run -p m14-unsafe-ffi-solution -- all    Everything");
        }
    }
}
