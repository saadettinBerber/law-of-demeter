use std::ffi::OsString;
use std::path::PathBuf;

use super::{Arguments, Command, parse};

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
