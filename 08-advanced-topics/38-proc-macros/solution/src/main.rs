// Reference solution for 38-proc-macros.
//
//     cargo run -p m38-proc-macros-solution -- <1-4|bonus|all>
//
// Each demo uses a macro on types defined right here. To see the generated
// code: `cargo install cargo-expand`, then
// `cargo expand -p m38-proc-macros-solution --bin m38-proc-macros-solution`.

use m38_proc_macros_solution::json::ToJson;
use m38_proc_macros_solution::orm::{Table, Value};
use m38_proc_macros_solution::{EnumIter, Table, ToJson, memoize, state_machine};

#[derive(ToJson)]
struct User {
    id: u64,
    #[json(rename = "displayName")]
    name: String,
    tags: Vec<String>,
    manager: Option<Box<User>>,
    #[json(skip)]
    #[allow(dead_code)]
    password_hash: String,
}

#[derive(ToJson)]
enum Event {
    Started,
    #[json(rename = "stopped")]
    Stopped,
    Moved {
        x: i32,
        y: i32,
    },
    Said(String),
}

fn ex1() {
    println!("--- 1: #[derive(ToJson)]");
    let boss = User {
        id: 1,
        name: "Ana".into(),
        tags: vec![],
        manager: None,
        password_hash: "secret".into(),
    };
    let user = User {
        id: 2,
        name: "Ben \"B\"".into(),
        tags: vec!["rust".into()],
        manager: Some(Box::new(boss)),
        password_hash: "x".into(),
    };
    println!("{}", user.to_json());
    for e in [
        Event::Started,
        Event::Stopped,
        Event::Moved { x: 1, y: -2 },
        Event::Said("hi".into()),
    ] {
        println!("{}", e.to_json());
    }
}

#[derive(Table, Debug, PartialEq)]
#[table(name = "people")]
struct Person {
    #[column(primary_key)]
    id: i64,
    #[column(name = "full_name")]
    name: String,
    email: Option<String>,
    active: bool,
    #[column(skip)]
    cached_score: u32,
}

fn ex2() {
    println!("--- 2: #[derive(Table)]");
    println!("{}", Person::create_table_sql());
    println!("{}", Person::insert_sql());
    println!("{}", Person::select_by_key_sql());
    let p = Person {
        id: 7,
        name: "Cy".into(),
        email: None,
        active: true,
        cached_score: 99,
    };
    println!("values: {:?}", p.values());
    let back = Person::from_row(&p.values()).unwrap();
    println!("from_row: {back:?} (cached_score is skipped: back to its default)");
    println!(
        "a bad row: {:?}",
        Person::from_row(&[
            Value::Integer(1),
            Value::Integer(2),
            Value::Null,
            Value::Integer(0)
        ])
        .unwrap_err()
    );
}

state_machine! {
    machine Door {
        initial Closed;
        Closed -> Open on Push;
        Open -> Closed on Pull;
        Closed -> Locked on Lock;
        Locked -> Closed on Unlock;
    }
}

fn ex3() {
    println!("--- 3: state_machine!");
    let mut door = Door::new();
    for event in [
        DoorEvent::Push,
        DoorEvent::Lock,
        DoorEvent::Pull,
        DoorEvent::Lock,
        DoorEvent::Push,
        DoorEvent::Unlock,
    ] {
        match door.fire(event) {
            Ok(state) => println!("{event:?} -> {state:?}"),
            Err(e) => println!("{event:?} refused: {e}"),
        }
    }
    println!("{} transitions defined", Door::TRANSITIONS.len());
}

#[memoize]
fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

#[memoize]
fn edit_distance(a: String, b: String) -> usize {
    if a.is_empty() {
        return b.chars().count();
    }
    if b.is_empty() {
        return a.chars().count();
    }
    let (a1, b1) = (a[1..].to_string(), b[1..].to_string());
    if a.as_bytes()[0] == b.as_bytes()[0] {
        return edit_distance(a1, b1);
    }
    1 + edit_distance(a1.clone(), b.clone())
        .min(edit_distance(a.clone(), b1.clone()))
        .min(edit_distance(a1, b1))
}

fn ex4() {
    println!("--- 4: #[memoize]");
    let start = std::time::Instant::now();
    println!(
        "fib(90) = {} in {:?}, {} results cached",
        fib(90),
        start.elapsed(),
        fib_cache_len()
    );
    println!(
        "edit_distance(kitten, sitting) = {}",
        edit_distance("kitten".into(), "sitting".into())
    );
}

#[derive(EnumIter, Debug, Clone, Copy, PartialEq)]
enum Planet {
    Mercury,
    Venus,
    Earth,
    Mars,
}

fn bonus() {
    println!("--- bonus: #[derive(EnumIter)]");
    println!(
        "{} planets: {:?}",
        Planet::COUNT,
        Planet::ALL.iter().map(Planet::name).collect::<Vec<_>>()
    );
    println!(
        "from_name(\"Mars\") = {:?}, from_name(\"Pluto\") = {:?}",
        Planet::from_name("Mars"),
        Planet::from_name("Pluto")
    );
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            bonus();
        }
        _ => println!(
            "38-proc-macros -- reference solution\n\n  cargo run -p m38-proc-macros-solution -- <1-4|bonus|all>"
        ),
    }
}
