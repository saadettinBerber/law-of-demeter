use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../class-file/tests/fixtures")
}

#[test]
fn a_declared_class_with_lombok_setters_and_behavior_is_reported_as_hybrid() {
    let sources = tempfile::tempdir().expect("temporary source directory");
    fs::create_dir(sources.path().join("shop")).expect("package directory");
    fs::write(
        sources.path().join("shop/Account.java"),
        "package shop;\n\npublic class Account {\n}\n",
    )
    .expect("java source");

    let output = Command::new(env!("CARGO_BIN_EXE_law-of-demeter"))
        .arg("--source")
        .arg(sources.path())
        .arg("--classpath")
        .arg(fixtures().join("classes"))
        .arg("--jdk-home")
        .arg(fixtures().join("jdk"))
        .arg("--release")
        .arg("8")
        .output()
        .expect("the binary runs");
    let report = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        (
            output.status.code(),
            report
                .lines()
                .any(|line| line.contains("shop.Account") && line.contains("Hibrit sınıf")),
        ),
        (Some(1), true),
        "stdout:\n{report}"
    );
}
