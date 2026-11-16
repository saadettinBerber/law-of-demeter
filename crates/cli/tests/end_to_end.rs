use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../class-file/tests/fixtures")
}

fn report_line<'a>(report: &'a str, call: &str) -> Option<&'a str> {
    report.lines().find(|line| line.contains(call))
}

#[test]
fn a_java_project_is_judged_from_source_and_compiled_classes() {
    let sources = tempfile::tempdir().expect("temporary source directory");
    fs::create_dir(sources.path().join("app")).expect("package directory");
    fs::write(
        sources.path().join("app/Checkout.java"),
        "package app;\n\
         \n\
         import shop.Order;\n\
         \n\
         public class Checkout {\n\
         \x20   public String nameOf(Order order) {\n\
         \x20       return order.getCustomer().getName();\n\
         \x20   }\n\
         \n\
         \x20   public void reset(Order order) {\n\
         \x20       order.getLines().clear();\n\
         \x20   }\n\
         }\n",
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
        .arg("--show-allowed")
        .output()
        .expect("the binary runs");
    let report = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        (
            output.status.code(),
            report_line(&report, "clear()").map(|line| line.contains("Violation")),
            report_line(&report, "getName()").map(|line| line.contains("Allowed")),
        ),
        (Some(1), Some(true), Some(true)),
        "stdout:\n{report}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
