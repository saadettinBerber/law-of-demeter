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

#[test]
#[ignore = "requires a JDK in JAVA_HOME; run with --ignored"]
fn an_inline_java_source_path_can_begin_with_a_dash() {
    let project = tempfile::tempdir().expect("temporary Java project");
    let source = project.path().join("-src");
    fs::create_dir(&source).expect("source directory");
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/container/OrderService.java"),
        source.join("OrderService.java"),
    )
    .expect("Java fixture");
    let jdk = PathBuf::from(std::env::var_os("JAVA_HOME").expect("JAVA_HOME points to a JDK"));
    let compiled = Command::new(jdk.join("bin/javac"))
        .current_dir(project.path())
        .args(["-d", "out", "./-src/OrderService.java"])
        .output()
        .expect("Java compiler");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let analyze = |source| {
        Command::new(env!("CARGO_BIN_EXE_law-of-demeter"))
            .current_dir(project.path())
            .args([
                source,
                "--classpath=out",
                "--only=shop.OrderService#pay",
                "--show-allowed",
            ])
            .output()
            .expect("Java CLI")
    };

    let relative = analyze("--source=./-src");
    let inline = analyze("--source=-src");

    assert_eq!(
        relative.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&relative.stderr)
    );
    assert_eq!(
        inline.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&inline.stderr)
    );
    assert!(
        String::from_utf8_lossy(&inline.stdout)
            .contains("OrderService.java:5  shop.OrderService.pay  charge()  Allowed:")
    );
}
