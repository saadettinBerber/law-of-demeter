use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_law-of-demeter"))
        .args(arguments)
        .env_remove("JAVA_HOME")
        .output()
        .expect("binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn help_goes_to_standard_output_with_exit_code_zero() {
    let output = run(&["--help"]);

    assert_eq!(
        (
            output.status.code(),
            text(&output.stdout).starts_with("Usage: law-of-demeter")
        ),
        (Some(0), true)
    );
}

#[test]
fn a_usage_error_goes_to_standard_error_with_exit_code_two() {
    let output = run(&["--verbose"]);

    assert_eq!(
        (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr).starts_with("Error: unknown option: --verbose\n")
        ),
        (Some(2), String::new(), true)
    );
}
