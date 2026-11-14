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
