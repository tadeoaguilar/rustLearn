use crate::sut;
use crate::sut::ex07_builder::Command;

#[test]
fn builds_with_required_fields() {
    let cmd = Command::builder()
        .executable("cargo".to_string())
        .args(vec!["build".to_string()])
        .build()
        .unwrap();
    assert_eq!(
        cmd,
        Command {
            executable: "cargo".into(),
            args: vec!["build".into()],
            current_dir: None,
            verbose: false
        }
    );
}

#[test]
fn optional_and_default_fields() {
    let cmd = Command::builder()
        .executable("ls".into())
        .args(vec![])
        .current_dir("/tmp".into()) // takes String, not Option<String>
        .verbose(true)
        .build()
        .unwrap();
    assert_eq!(cmd.current_dir.as_deref(), Some("/tmp"));
    assert!(cmd.verbose);
}

#[test]
fn missing_required_field_is_an_error() {
    assert_eq!(
        Command::builder().build(),
        Err("missing field `executable`".to_string())
    );
    assert_eq!(
        Command::builder().executable("x".into()).build(),
        Err("missing field `args`".to_string())
    );
}

#[test]
fn builder_derive_works_in_another_crate() {
    #[derive(sut::Builder, Debug)]
    struct Config {
        name: String,
        port: Option<u16>,
    }
    let c = Config::builder().name("svc".into()).build().unwrap();
    assert_eq!((c.name.as_str(), c.port), ("svc", None));
}
