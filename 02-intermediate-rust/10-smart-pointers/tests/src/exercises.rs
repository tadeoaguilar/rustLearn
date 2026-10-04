use crate::sut::*;
use std::rc::Rc;

// ---- Exercise 1 --------------------------------------------------------------

#[test]
fn ex1_cons_list() {
    use ex01_box::{List::*, from_slice, len, sum};
    let list = from_slice(&[1, 2, 3]);
    assert_eq!(
        list,
        Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))))
    );
    assert_eq!(sum(&list), 6);
    assert_eq!(len(&list), 3);
    assert_eq!(from_slice(&[]), Nil);
}

#[test]
fn ex1_expression_tree() {
    use ex01_box::{Expr, eval, to_string};
    let e = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Num(1.0)),
            Box::new(Expr::Num(2.0)),
        )),
        Box::new(Expr::Neg(Box::new(Expr::Num(3.0)))),
    );
    assert_eq!(eval(&e), -9.0);
    assert_eq!(to_string(&e), "((1 + 2) * -3)");
    assert_eq!(
        e,
        Expr::times(
            Expr::plus(Expr::num(1.0), Expr::num(2.0)),
            Expr::negate(Expr::num(3.0))
        )
    );
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_push_pop_peek() {
    let mut s = ex02_linked_list::Stack::new();
    assert!(s.is_empty());
    assert_eq!(s.pop(), None);
    s.push(1);
    s.push(2);
    s.push(3);
    assert_eq!(s.len(), 3);
    assert_eq!(s.peek(), Some(&3));
    if let Some(top) = s.peek_mut() {
        *top = 30;
    }
    assert_eq!(s.pop(), Some(30));
    assert_eq!(s.pop(), Some(2));
    assert_eq!(s.len(), 1);
}

#[test]
fn ex2_iter_reverse_into_iter() {
    let mut s: ex02_linked_list::Stack<i32> = (1..=4).collect();
    assert_eq!(s.iter().copied().collect::<Vec<_>>(), vec![4, 3, 2, 1]);
    s.reverse();
    assert_eq!(s.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
    assert_eq!(s.peek(), Some(&1));
    assert_eq!(s.len(), 4, "reverse keeps the length");
    assert_eq!(s.into_iter().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
}

#[test]
fn ex2_dropping_a_long_list_does_not_overflow_the_stack() {
    // Test threads have a small stack (2 MiB); a recursive drop of a million
    // boxes needs far more. Without the custom Drop this test aborts.
    let mut s = ex02_linked_list::Stack::new();
    for i in 0..1_000_000 {
        s.push(i);
    }
    drop(s);
}

#[test]
fn ex2_works_with_non_copy_values() {
    let mut s = ex02_linked_list::Stack::new();
    s.push(String::from("a"));
    s.push(String::from("b"));
    assert_eq!(
        s.iter().map(String::as_str).collect::<Vec<_>>(),
        vec!["b", "a"]
    );
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_deref_coercion() {
    use ex03_deref_drop::*;
    let mut m = MyBox::new(String::from("Rust"));
    assert_eq!(greet_boxed(&m), "Hello, Rust!");
    assert_eq!(*m, "Rust");
    m.push('!'); // DerefMut
    assert_eq!(m.len(), 5);
}

#[test]
fn ex3_drop_order() {
    use ex03_deref_drop::*;
    let check = |f: fn(&Log), expected: &[&str]| {
        let log = new_log();
        f(&log);
        assert_eq!(*log.borrow(), expected);
    };
    check(locals_order, &["drop c", "drop b", "drop a"]);
    check(fields_order, &["drop first", "drop second"]);
    check(early_drop, &["drop a", "end of function", "drop b"]);
    check(
        moved_into_function,
        &["inside consume(a)", "drop a", "back in caller"],
    );
    check(
        underscore_vs_named,
        &["drop unbound", "end of function", "drop kept"],
    );
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_tree_navigation() {
    use ex04_rc_weak_tree::*;
    let root = sample_tree();
    let bin = TreeNode::find(&root, "bin").expect("bin exists");
    assert_eq!(bin.depth(), 2);
    assert_eq!(bin.path(), "root/usr/bin");
    assert_eq!(root.depth(), 0);
    assert_eq!(root.path(), "root");
    assert_eq!(
        bin.parent().map(|p| p.value.clone()),
        Some("usr".to_string())
    );
    assert!(root.parent().is_none());
    assert!(TreeNode::find(&root, "etc").is_none());
    let names: Vec<String> = root.children().iter().map(|c| c.value.clone()).collect();
    assert_eq!(names, vec!["usr", "home"]);
}

#[test]
fn ex4_counts_and_weak_parents() {
    use ex04_rc_weak_tree::*;
    let root = sample_tree();
    let usr = TreeNode::find(&root, "usr").unwrap();
    assert_eq!(
        Rc::strong_count(&usr),
        2,
        "root's children list + our handle"
    );
    assert_eq!(Rc::weak_count(&usr), 2, "parent links from bin and lib");
    assert_eq!(
        Rc::strong_count(&root),
        1,
        "children don't own their parent"
    );

    let bin = TreeNode::find(&root, "bin").unwrap();
    drop(usr);
    drop(root);
    assert!(bin.parent().is_none(), "the tree above bin was freed");
}

// ---- Exercise 5 --------------------------------------------------------------

#[test]
fn ex5_limit_tracker_messages() {
    use ex05_refcell::*;
    let mock = MockMessenger::new();
    let mut tracker = LimitTracker::new(&mock, 100);
    tracker.set_value(50);
    assert!(mock.sent.borrow().is_empty());
    tracker.set_value(75);
    tracker.set_value(90);
    tracker.set_value(100);
    assert_eq!(tracker.value(), 100);
    let sent = mock.sent.borrow();
    assert_eq!(sent.len(), 3);
    assert!(sent[0].starts_with("Warning"));
    assert!(sent[1].starts_with("Urgent warning"));
    assert!(sent[2].starts_with("Error"));
    assert_eq!(mock.calls.get(), 3, "the Cell counted every call");
}

#[test]
#[should_panic(expected = "already")]
fn ex5_double_borrow_mut_panics() {
    let cell = std::cell::RefCell::new(vec![]);
    ex05_refcell::double_borrow_panics(&cell);
}

#[test]
fn ex5_try_borrow_mut_does_not_panic() {
    let cell = std::cell::RefCell::new(vec![]);
    assert!(ex05_refcell::double_borrow_checked(&cell).is_err());
    assert!(cell.borrow().is_empty(), "nothing was pushed");
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_friendships() {
    use ex06_graph::*;
    let [ann, bob, cat, dan, eve] = ["Ann", "Bob", "Cat", "Dan", "Eve"].map(person);
    befriend(&ann, &bob);
    befriend(&ann, &cat);
    befriend(&bob, &cat);
    befriend(&cat, &dan);
    befriend(&ann, &bob); // duplicate: ignored
    befriend(&eve, &eve); // self: ignored
    assert_eq!(friend_names(&ann), vec!["Bob", "Cat"]);
    assert_eq!(friend_names(&cat), vec!["Ann", "Bob", "Dan"]);
    assert_eq!(mutual_friends(&ann, &bob), vec!["Cat"]);
    assert_eq!(reachable(&ann), vec!["Bob", "Cat", "Dan"]);
    assert_eq!(reachable(&eve), Vec::<String>::new());
}

#[test]
fn ex6_weak_friends_disappear_when_dropped() {
    use ex06_graph::*;
    let ann = person("Ann");
    {
        let bob = person("Bob");
        befriend(&ann, &bob);
        assert_eq!(friend_names(&ann), vec!["Bob"]);
    }
    assert!(
        friend_names(&ann).is_empty(),
        "Bob was freed; Ann's Weak link no longer upgrades"
    );
}

#[test]
fn ex6_strong_cycle_leaks_weak_does_not() {
    use ex06_graph::*;
    let r = leak_with_rc_cycle();
    assert_eq!(
        r.drops_after_dropping_variables, 0,
        "nothing freed: the cycle keeps both alive"
    );
    assert!(r.still_alive);
    assert_eq!(r.drops_after_breaking_cycle, 2);
    assert!(!no_leak_with_weak());
}

// ---- Exercise 7 --------------------------------------------------------------

#[test]
fn ex7_basic_operations() {
    let cache: ex07_shared_cache::SharedCache<&str, i32> = ex07_shared_cache::SharedCache::new();
    assert!(cache.is_empty());
    cache.insert("a", 1);
    let handle = cache.clone();
    handle.insert("b", 2);
    assert_eq!(cache.get(&"b"), Some(2), "clones share one map");
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get_or_compute("a", || 100), 1, "existing value wins");
    assert_eq!(cache.get_or_compute("c", || 3), 3);
}

#[test]
fn ex7_compute_runs_once_across_threads() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let cache = ex07_shared_cache::SharedCache::<u32, u32>::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let (cache, calls) = (cache.clone(), Arc::clone(&calls));
            std::thread::spawn(move || {
                cache.get_or_compute(7, || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    49
                })
            })
        })
        .collect();
    for h in handles {
        assert_eq!(h.join().unwrap(), 49);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn ex7_rwlock_version() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let cache = ex07_shared_cache::ReadMostlyCache::<u32, u32>::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let (cache, calls) = (cache.clone(), Arc::clone(&calls));
            std::thread::spawn(move || {
                cache.get_or_compute(1, || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    10
                })
            })
        })
        .collect();
    for h in handles {
        assert_eq!(h.join().unwrap(), 10);
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "the re-check under the write lock matters"
    );
    assert_eq!(cache.get(&1), Some(10));
    assert_eq!(cache.len(), 1);
}

#[test]
fn ex7_cache_survives_a_panicking_thread() {
    let cache = ex07_shared_cache::SharedCache::<u32, u32>::new();
    let c = cache.clone();
    let result = std::thread::spawn(move || {
        c.get_or_compute(1, || panic!("compute failed"));
    })
    .join();
    assert!(result.is_err());
    // The Mutex is poisoned now; the cache must still work.
    cache.insert(2, 2);
    assert_eq!(cache.get(&2), Some(2));
}

// ---- Bonus -------------------------------------------------------------------

#[test]
fn bonus_cow_allocates_only_when_needed() {
    use bonus_cow::normalize_whitespace as n;
    use std::borrow::Cow;
    assert!(matches!(n("already fine"), Cow::Borrowed("already fine")));
    assert!(matches!(n(""), Cow::Borrowed("")));
    assert_eq!(n("too   many"), "too many");
    assert!(matches!(n("too   many"), Cow::Owned(_)));
    assert_eq!(n("\ttabs\tand\nnewlines "), "tabs and newlines");
    assert_eq!(n(" lead"), "lead");
}
