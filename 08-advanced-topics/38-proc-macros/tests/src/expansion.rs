//! The expansion functions on token streams: what they generate, and the
//! compile errors they report (with messages a user can act on).

use crate::core_sut::{
    bonus_enum_iter, ex01_to_json, ex02_table, ex03_state_machine, ex04_memoize,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

fn krate() -> syn::Path {
    parse_quote!(::runtime)
}

#[track_caller]
fn error_of(result: syn::Result<TokenStream>) -> String {
    match result {
        Ok(tokens) => panic!("expected a compile error, got: {tokens}"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn ex1_expansion() {
    let out = ex01_to_json::derive(quote! { struct P { x: i32 } }, &krate())
        .unwrap()
        .to_string();
    assert!(
        out.contains(":: runtime :: json :: ToJson") && out.contains("fn to_json"),
        "{out}"
    );
    let generic = ex01_to_json::derive(quote! { struct W<T> { v: T } }, &krate())
        .unwrap()
        .to_string();
    assert!(
        generic.contains("T : :: runtime :: json :: ToJson"),
        "type parameters get the bound: {generic}"
    );
    assert_eq!(ex01_to_json::json_string("a\"b\\c"), "\"a\\\"b\\\\c\"");
}

#[test]
fn ex1_errors() {
    let unknown = error_of(ex01_to_json::derive(
        quote! { struct P { #[json(renam = "y")] x: i32 } },
        &krate(),
    ));
    assert!(
        unknown.contains("unknown `json` option `renam`"),
        "{unknown}"
    );
    assert!(
        error_of(ex01_to_json::derive(quote! { union U { a: u8 } }, &krate())).contains("union")
    );
    assert!(error_of(ex01_to_json::derive(quote! { enum E {} }, &krate())).contains("no variants"));
}

#[test]
fn ex2_expansion_and_errors() {
    let out = ex02_table::derive(
        quote! { struct User { #[column(primary_key)] id: i64, name: String } },
        &krate(),
    )
    .unwrap()
    .to_string();
    assert!(
        out.contains("\"users\""),
        "default table name: snake_case + s: {out}"
    );
    assert!(out.contains(":: runtime :: orm :: Table"));
    let renamed = ex02_table::derive(
        quote! { #[table(name = "people")] struct UserAccount { #[column(primary_key)] id: i64 } },
        &krate(),
    )
    .unwrap();
    assert!(renamed.to_string().contains("\"people\""));
    let camel = ex02_table::derive(
        quote! { struct UserAccount { #[column(primary_key)] id: i64 } },
        &krate(),
    )
    .unwrap();
    assert!(camel.to_string().contains("\"user_accounts\""));

    let no_pk = error_of(ex02_table::derive(
        quote! { struct T { id: i64 } },
        &krate(),
    ));
    assert!(no_pk.contains("primary_key"), "{no_pk}");
    let two = error_of(ex02_table::derive(
        quote! { struct T { #[column(primary_key)] a: i64, #[column(primary_key)] b: i64 } },
        &krate(),
    ));
    assert!(
        two.contains("only one primary key") && two.contains("`a`"),
        "{two}"
    );
    assert!(
        error_of(ex02_table::derive(quote! { struct T(i64); }, &krate())).contains("named fields")
    );
    assert!(error_of(ex02_table::derive(quote! { enum T { A } }, &krate())).contains("structs"));
    assert!(
        error_of(ex02_table::derive(
            quote! { struct T<X> { #[column(primary_key)] id: X } },
            &krate()
        ))
        .contains("generic")
    );
    assert!(
        error_of(ex02_table::derive(
            quote! { struct T { #[column(primary_key, skip)] id: i64 } },
            &krate()
        ))
        .contains("skipped")
    );
}

#[test]
fn ex3_parsing_and_errors() {
    let machine: ex03_state_machine::Machine =
        syn::parse2(quote! { machine M { initial A; A -> B on Go; B -> A on Back; } }).unwrap();
    assert_eq!(machine.name, "M");
    assert_eq!(machine.initial, "A");
    assert_eq!(machine.transitions.len(), 2);
    assert_eq!(
        (
            machine.transitions[1].from.to_string(),
            machine.transitions[1].to.to_string(),
            machine.transitions[1].event.to_string()
        ),
        ("B".into(), "A".into(), "Back".into())
    );

    let out = ex03_state_machine::expand(
        quote! { machine Door { initial Closed; Closed -> Open on Push; } },
    )
    .unwrap()
    .to_string();
    for item in [
        "enum DoorState",
        "enum DoorEvent",
        "struct DoorError",
        "struct Door",
        "fn fire",
        "TRANSITIONS",
    ] {
        assert!(out.contains(item), "{item} missing in {out}");
    }

    assert!(
        error_of(ex03_state_machine::expand(
            quote! { machine M { A -> B on Go; } }
        ))
        .contains("initial")
    );
    assert!(
        error_of(ex03_state_machine::expand(
            quote! { machine M { initial A; } }
        ))
        .contains("at least one transition")
    );
    let dup = error_of(ex03_state_machine::expand(
        quote! { machine M { initial A; A -> B on Go; A -> C on Go; } },
    ));
    assert!(dup.contains("already has a transition on `Go`"), "{dup}");
    assert!(
        ex03_state_machine::expand(quote! { machine M { initial A; A => B on Go; } }).is_err(),
        "a syntax error"
    );
}

#[test]
fn ex4_expansion_and_errors() {
    let out = ex04_memoize::expand(
        quote! {},
        quote! { pub fn square(x: u32) -> u64 { x as u64 * x as u64 } },
    )
    .unwrap()
    .to_string();
    assert!(
        out.contains("pub fn square")
            && out.contains("fn square_cache_len")
            && out.contains("thread_local"),
        "{out}"
    );
    assert!(
        error_of(ex04_memoize::expand(
            quote! { size = 3 },
            quote! { fn f(x: u8) -> u8 { x } }
        ))
        .contains("no arguments")
    );
    assert!(
        error_of(ex04_memoize::expand(
            quote! {},
            quote! { fn f(&self, x: u8) -> u8 { x } }
        ))
        .contains("not methods")
    );
    assert!(
        error_of(ex04_memoize::expand(quote! {}, quote! { fn f(x: u8) { } }))
            .contains("returns a value")
    );
    assert!(
        error_of(ex04_memoize::expand(
            quote! {},
            quote! { async fn f(x: u8) -> u8 { x } }
        ))
        .contains("async")
    );
    assert!(
        error_of(ex04_memoize::expand(
            quote! {},
            quote! { fn f<T>(x: T) -> T { x } }
        ))
        .contains("generic")
    );
    assert!(
        error_of(ex04_memoize::expand(
            quote! {},
            quote! { fn f((a, b): (u8, u8)) -> u8 { a } }
        ))
        .contains("identifier")
    );
}

#[test]
fn bonus_expansion_and_errors() {
    let out = bonus_enum_iter::derive(quote! { enum Color { Red, Green } })
        .unwrap()
        .to_string();
    assert!(out.contains("COUNT") && out.contains("\"Green\""), "{out}");
    assert!(
        error_of(bonus_enum_iter::derive(quote! { enum E { A(u8) } })).contains("unit variants")
    );
    assert!(error_of(bonus_enum_iter::derive(quote! { struct S; })).contains("enums"));
}
