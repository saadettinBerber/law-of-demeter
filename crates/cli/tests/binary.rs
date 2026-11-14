use std::path::PathBuf;
use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_law-of-demeter"))
        .args(arguments)
        .env_remove("JAVA_HOME")
        .output()
        .expect("binary runs")
}

fn fixture_jdk() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../class-file/tests/fixtures/jdk")
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
            text(&output.stdout).starts_with("Kullanım: law-of-demeter")
        ),
        (Some(0), true)
    );
}

#[test]
fn a_usage_error_goes_to_standard_error_with_exit_code_two() {
    let output = run(&[]);

    assert_eq!(
        (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr).starts_with("Hata: en az bir --source gerekli\n")
        ),
        (Some(2), String::new(), true)
    );
}

#[test]
fn until_a_source_mapper_is_wired_in_a_valid_run_says_so_and_fails() {
    let source = env!("CARGO_MANIFEST_DIR");
    let jdk = fixture_jdk();

    let output = run(&["--source", source, "--jdk-home", &jdk.display().to_string()]);

    assert_eq!(
        (output.status.code(), text(&output.stderr)),
        (Some(2), "Hata: kaynak ayrıştırıcı bağlı değil\n".to_owned())
    );
}
