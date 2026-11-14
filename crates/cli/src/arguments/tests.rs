use std::ffi::OsString;
use std::path::PathBuf;

use super::{Arguments, Command, UsageError, parse};

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
fn at_least_one_source_root_is_required() {
    let error = rejected(&["--classpath", "lib"]);

    assert_eq!(error, UsageError("en az bir --source gerekli".to_owned()));
}

#[test]
fn an_unknown_option_is_rejected() {
    let error = rejected(&["-s", "src", "--verbose"]);

    assert_eq!(
        error,
        UsageError("bilinmeyen seçenek: --verbose".to_owned())
    );
}

#[test]
fn an_option_without_its_value_is_rejected() {
    let error = rejected(&["-s", "src", "--classpath"]);

    assert_eq!(
        error,
        UsageError("--classpath için değer gerekli".to_owned())
    );
}

#[test]
fn a_release_that_is_not_a_number_is_rejected() {
    let error = rejected(&["-s", "src", "--release", "yirmi"]);

    assert_eq!(
        error,
        UsageError("--release bir sürüm numarası olmalı: yirmi".to_owned())
    );
}

#[test]
fn a_stray_positional_argument_is_rejected() {
    let error = rejected(&["-s", "src", "Order.java"]);

    assert_eq!(
        error,
        UsageError("beklenmeyen argüman: Order.java".to_owned())
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
