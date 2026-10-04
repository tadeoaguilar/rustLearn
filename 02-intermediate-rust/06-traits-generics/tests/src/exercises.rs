use crate::sut::*;

// ---- Exercise 1 --------------------------------------------------------------

#[test]
fn ex1_task1_each_type_summarizes_itself() {
    use ex01_basic_traits::{sample_article, sample_tweet, task1::*};
    assert_eq!(sample_article().summarize(), "Rust is Great by Alice");
    assert_eq!(sample_tweet().summarize(), "@bob: Learning Rust!");
    assert_eq!(
        notify(&sample_tweet()),
        "Breaking news! @bob: Learning Rust!"
    );
}

#[test]
fn ex1_task2_default_and_overridden_methods() {
    use ex01_basic_traits::{sample_article, sample_tweet, task2::Summary};
    assert_eq!(
        sample_tweet().summarize(),
        "(Read more from @bob...)",
        "default method"
    );
    assert_eq!(
        sample_article().summarize(),
        "Rust is Great, by Alice",
        "overridden"
    );
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_largest_works_for_any_partial_ord() {
    use ex02_generic_functions::*;
    assert_eq!(*largest(&[34, 50, 25, 100, 65]), 100);
    assert_eq!(*largest(&['y', 'm', 'a', 'q']), 'y');
    assert_eq!(largest(&[1.5, -2.0]), &1.5);
    let words = [String::from("apple"), String::from("pear")];
    assert_eq!(largest(&words), "pear");
}

#[test]
#[should_panic]
fn ex2_largest_panics_on_empty_input() {
    ex02_generic_functions::largest::<i32>(&[]);
}

#[test]
fn ex2_largest_checked() {
    use ex02_generic_functions::largest_checked;
    assert_eq!(largest_checked::<i32>(&[]), None);
    assert_eq!(largest_checked(&[3, 9, 2]), Some(&9));
}

#[test]
fn ex2_swap_and_pair() {
    use ex02_generic_functions::*;
    let (mut x, mut y) = (5, 10);
    swap(&mut x, &mut y);
    assert_eq!((x, y), (10, 5));
    let (mut a, mut b) = (String::from("a"), String::from("b"));
    swap(&mut a, &mut b);
    assert_eq!((a.as_str(), b.as_str()), ("b", "a"));

    let pair = Pair::new(5, "hello");
    assert_eq!(*pair.get_first(), 5);
    assert_eq!(*pair.get_second(), "hello");
    let swapped: Pair<&str, i32> = pair.swap();
    assert_eq!(*swapped.get_first(), "hello");
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_bounds() {
    use ex03_trait_bounds::*;
    assert_eq!(print_pair(1, "two"), "1 and two");
    assert_eq!(print_pair_where('a', 2.5), "a and 2.5");
    assert_eq!(Pair::new(3, 7).cmp_display(), "Largest: 7");
    assert_eq!(Pair::new(9, 7).cmp_display(), "Largest: 9");
    assert_eq!(
        compare_and_print(String::from("pear"), String::from("apple")),
        "pear"
    );
    assert_eq!(compare_and_print(1, 2), 2);
    assert_eq!(describe(4).to_string(), "4 is even");
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_operators() {
    use ex04_operator_overloading::Vector2D as V;
    let v1 = V::new(1.0, 2.0);
    let v2 = V::new(3.0, 4.0);
    assert_eq!(v1 + v2, V::new(4.0, 6.0));
    assert_eq!(v2 - v1, V::new(2.0, 2.0));
    assert_eq!(v1 * 3.0, V::new(3.0, 6.0));
    assert_eq!(3.0 * v1, V::new(3.0, 6.0));
    assert_eq!(-v1, V::new(-1.0, -2.0));
    assert_eq!(v2.magnitude(), 5.0);
    assert_eq!(v1.dot(v2), 11.0);
}

#[test]
fn ex4_add_assign_sum_display() {
    use ex04_operator_overloading::Vector2D as V;
    let mut v = V::ZERO;
    v += V::new(1.0, 1.0);
    v += V::new(2.0, 3.0);
    assert_eq!(v, V::new(3.0, 4.0));
    let total: V = vec![V::new(1.0, 0.0), V::new(0.0, 1.0), V::new(1.0, 1.0)]
        .into_iter()
        .sum();
    assert_eq!(total, V::new(2.0, 2.0));
    assert_eq!(V::new(1.0, 2.5).to_string(), "(1, 2.5)");
}

// ---- Exercise 5 --------------------------------------------------------------

#[test]
fn ex5_counter() {
    use ex05_associated_types::Counter;
    assert_eq!(Counter::new(5).collect::<Vec<_>>(), vec![1, 2, 3, 4, 5]);
    assert_eq!(Counter::new(0).next(), None);
    let sum: u32 = Counter::new(5)
        .zip(Counter::new(5).skip(1))
        .map(|(a, b)| a * b)
        .filter(|x| x % 3 == 0)
        .sum();
    assert_eq!(sum, 18, "the Rust Book's adaptor exercise");
}

#[test]
fn ex5_graph() {
    use ex05_associated_types::{Graph, RoadMap, out_degree};
    let mut map = RoadMap::new();
    map.add_road("Paris", "Lyon", 465);
    map.add_road("Paris", "Lille", 225);
    map.add_road("Lyon", "Marseille", 315);
    let s = |x: &str| x.to_string();
    assert!(map.has_edge(&s("Paris"), &s("Lyon")));
    assert!(!map.has_edge(&s("Lyon"), &s("Paris")), "edges are directed");
    assert!(!map.has_edge(&s("Nowhere"), &s("Paris")));
    assert_eq!(out_degree(&map, &s("Paris")), 2);
    assert_eq!(out_degree(&map, &s("Marseille")), 0);
    let km: u32 = map.edges(&s("Paris")).iter().map(|r| r.km).sum();
    assert_eq!(km, 690);
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_plugin_manager_runs_in_registration_order() {
    use ex06_trait_objects::*;
    let mut manager = PluginManager::new();
    manager.register(Box::new(LoggerPlugin {
        log_level: "INFO".into(),
    }));
    manager.register(Box::new(CachePlugin { cache_size: 128 }));
    manager.register(Box::new(FnPlugin {
        name: "Test".into(),
        run: || "ok".to_string(),
    }));
    assert_eq!(manager.names(), vec!["Logger", "Cache", "Test"]);
    assert_eq!(
        manager.run_all(),
        vec![
            "Logger: Logging at level: INFO",
            "Cache: Cache size: 128 MB",
            "Test: ok"
        ]
    );
}

#[test]
fn ex6_trait_objects_are_fat_pointers() {
    use ex06_trait_objects::{LoggerPlugin, Plugin};
    let thin = std::mem::size_of::<&LoggerPlugin>();
    assert_eq!(std::mem::size_of::<&dyn Plugin>(), 2 * thin);
}

// ---- Exercise 7 --------------------------------------------------------------

#[test]
fn ex7_stack_from_the_exercise() {
    use ex07_generic_stack::Stack;
    let mut stack = Stack::new();
    assert!(stack.is_empty());
    stack.push(1);
    stack.push(2);
    stack.push(3);
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.peek(), Some(&2));
    assert_eq!(stack.len(), 2);
    if let Some(top) = stack.peek_mut() {
        *top = 20;
    }
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), None);
}

#[test]
fn ex7_bonus_into_iterator_yields_top_first() {
    use ex07_generic_stack::Stack;
    let stack: Stack<i32> = (1..=3).collect();
    let borrowed: Vec<&i32> = (&stack).into_iter().collect();
    assert_eq!(borrowed, vec![&3, &2, &1]);
    let mut seen = vec![];
    for x in &stack {
        seen.push(*x);
    }
    assert_eq!(seen, vec![3, 2, 1]);
    let owned: Vec<i32> = stack.into_iter().collect();
    assert_eq!(owned, vec![3, 2, 1]);
}

#[test]
fn ex7_works_with_non_copy_types() {
    use ex07_generic_stack::Stack;
    let mut s: Stack<String> = Stack::default();
    s.push("a".into());
    assert_eq!(s.peek().map(String::as_str), Some("a"));
}

// ---- Bonus -------------------------------------------------------------------

#[test]
fn bonus_static_and_dynamic_dispatch() {
    use bonus_shapes::*;
    use std::f64::consts::PI;
    let circles = vec![Circle { radius: 1.0 }, Circle { radius: 2.0 }];
    assert!((total_area(&circles) - 5.0 * PI).abs() < 1e-9);

    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle {
            width: 4.0,
            height: 5.0,
        }),
    ];
    assert!((total_area_dyn(&shapes) - (PI + 20.0)).abs() < 1e-9);
    assert_eq!(
        draw_all(&shapes),
        vec!["Drawing circle with radius 1", "Drawing rectangle 4x5"]
    );
}

#[test]
fn bonus_upcasting_and_blanket_impl() {
    use bonus_shapes::*;
    let rect = Rectangle {
        width: 2.0,
        height: 3.0,
    };
    let as_drawable: &dyn Drawable = &rect;
    let as_shape: &dyn Shape = as_shape(as_drawable);
    assert_eq!(as_shape.perimeter(), 10.0);
    assert_eq!(
        rect.describe(),
        "rectangle with area 6.00 and perimeter 10.00"
    );
    assert_eq!(Pretty(&rect).to_string(), "[rectangle: area 6.0]");
}
