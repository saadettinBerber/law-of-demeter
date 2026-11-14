use std::ffi::OsString;

use super::{CLEAN, Environment, FAILURE, run};
use crate::arguments::usage;

fn nothing() -> Environment {
    Environment {
        java_home: None,
        path: None,
    }
}

struct Outcome {
    code: u8,
    out: String,
    err: String,
}

fn execute(arguments: &[&str]) -> Outcome {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        arguments.iter().map(OsString::from),
        &nothing(),
        None,
        &mut out,
        &mut err,
    );
    Outcome {
        code,
        out: String::from_utf8(out).expect("utf-8 output"),
        err: String::from_utf8(err).expect("utf-8 errors"),
    }
}

#[test]
fn help_prints_the_usage_and_succeeds() {
    let outcome = execute(&["--help"]);

    assert_eq!(
        (outcome.code, outcome.out, outcome.err),
        (CLEAN, usage().to_owned(), String::new())
    );
}

#[test]
fn a_usage_error_is_explained_with_the_usage_and_fails() {
    let outcome = execute(&["--verbose"]);

    assert_eq!(
        (outcome.code, outcome.out, outcome.err),
        (
            FAILURE,
            String::new(),
            format!("Hata: bilinmeyen seçenek: --verbose\n\n{}", usage())
        )
    );
}
