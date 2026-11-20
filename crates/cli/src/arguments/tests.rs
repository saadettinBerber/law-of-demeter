use std::ffi::OsString;
use std::path::PathBuf;

use law_of_demeter_core::application::Target;

use super::{Arguments, Command, UsageError, parse, usage};

fn parsed(arguments: &[&str]) -> Command {
    parse(arguments.iter().map(OsString::from)).expect("arguments parse")
}

#[test]
fn source_roots_can_be_repeated_in_long_and_short_form() {
    let command = parsed(&["--source", "src/main/java", "-s", "src/test/java"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![
                PathBuf::from("src/main/java"),
                PathBuf::from("src/test/java")
            ],
            ..Arguments::default()
        })
    );
}

#[cfg(unix)]
#[test]
fn classpath_entries_can_be_repeated_and_joined_with_colons() {
    let command = parsed(&[
        "-s",
        "src",
        "--classpath",
        "a.jar::lib/classes",
        "-c",
        "b.jar",
    ]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("src")],
            classpath: vec![
                PathBuf::from("a.jar"),
                PathBuf::from("lib/classes"),
                PathBuf::from("b.jar")
            ],
            ..Arguments::default()
        })
    );
}

#[test]
fn the_jdk_home_and_the_release_are_read() {
    let command = parsed(&["-s", "src", "--jdk-home", "/opt/jdk-21", "--release", "17"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("src")],
            jdk_home: Some(PathBuf::from("/opt/jdk-21")),
            release: Some(17),
            ..Arguments::default()
        })
    );
}

#[test]
fn show_allowed_is_a_flag() {
    let command = parsed(&["-s", "src", "--show-allowed"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("src")],
            show_allowed: true,
            ..Arguments::default()
        })
    );
}

fn rejected(arguments: &[&str]) -> UsageError {
    parse(arguments.iter().map(OsString::from)).expect_err("arguments are rejected")
}

#[test]
fn the_source_roots_may_be_left_out() {
    let command = parsed(&["--classpath", "lib"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            classpath: vec![PathBuf::from("lib")],
            ..Arguments::default()
        })
    );
}

#[test]
fn an_unknown_option_is_rejected() {
    let error = rejected(&["-s", "src", "--verbose"]);

    assert_eq!(error, UsageError("unknown option: --verbose".to_owned()));
}

#[test]
fn an_option_without_its_value_is_rejected() {
    let error = rejected(&["-s", "src", "--classpath"]);

    assert_eq!(error, UsageError("--classpath needs a value".to_owned()));
}

#[test]
fn a_release_that_is_not_a_number_is_rejected() {
    let error = rejected(&["-s", "src", "--release", "twenty"]);

    assert_eq!(
        error,
        UsageError("--release must be a version number: twenty".to_owned())
    );
}

#[test]
fn a_stray_positional_argument_is_rejected() {
    let error = rejected(&["-s", "src", "Order.java"]);

    assert_eq!(
        error,
        UsageError("unexpected argument: Order.java".to_owned())
    );
}

#[test]
fn an_option_can_carry_its_value_after_an_equals_sign() {
    let command = parsed(&["--source=src", "--release=21", "--jdk-home=/opt/jdk"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("src")],
            jdk_home: Some(PathBuf::from("/opt/jdk")),
            release: Some(21),
            ..Arguments::default()
        })
    );
}

#[test]
fn help_is_requested_with_either_spelling_and_needs_no_source() {
    let commands = [parsed(&["--help"]), parsed(&["-h"])];

    assert_eq!(commands, [Command::Help, Command::Help]);
}

#[test]
fn the_usage_text_names_every_option() {
    let text = usage();

    let missing: Vec<_> = [
        "--source",
        "--classpath",
        "--jdk-home",
        "--release",
        "--show-allowed",
        "--only",
        "--skill",
        "--help",
    ]
    .into_iter()
    .filter(|option| !text.contains(option))
    .collect();

    assert_eq!(missing, Vec::<&str>::new());
}

#[test]
fn classes_and_methods_can_be_selected_more_than_once() {
    let command = parsed(&["--only", "shop.Order", "--only=shop.Customer#getWallet"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            targets: vec![
                Target::class("shop.Order"),
                Target::method("shop.Customer", "getWallet")
            ],
            ..Arguments::default()
        })
    );
}

#[test]
fn a_target_without_a_class_or_a_method_name_is_rejected() {
    let errors = [
        rejected(&["--only", "#pay"]),
        rejected(&["--only", "shop.Order#"]),
    ];

    assert_eq!(
        errors,
        [
            UsageError("--only needs a class, optionally followed by #method: #pay".to_owned()),
            UsageError(
                "--only needs a class, optionally followed by #method: shop.Order#".to_owned()
            )
        ]
    );
}

#[test]
fn a_target_with_more_than_one_method_separator_is_rejected() {
    let error = rejected(&["--only", "shop.Order#pay#now"]);

    assert_eq!(
        error,
        UsageError(
            "--only needs a class, optionally followed by #method: shop.Order#pay#now".to_owned()
        )
    );
}
