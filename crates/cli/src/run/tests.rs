use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use law_of_demeter_core::application::ReadError;
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Call, Expression, Method};
use law_of_demeter_core::domain::type_shape::TypeRef;
use tempfile::TempDir;

use super::{CLEAN, Environment, FAILURE, VIOLATIONS, run};
use crate::arguments::usage;
use crate::source_files::{Declarations, Mapper};

fn nothing() -> Environment {
    Environment {
        java_home: None,
        path: None,
        working_directory: PathBuf::new(),
    }
}

struct Outcome {
    code: u8,
    out: String,
    err: String,
}

fn execute(arguments: &[&str]) -> Outcome {
    execute_with(arguments, None)
}

fn execute_with(arguments: &[&str], mapper: Option<&Mapper>) -> Outcome {
    execute_in(&nothing(), arguments, mapper)
}

fn execute_in(environment: &Environment, arguments: &[&str], mapper: Option<&Mapper>) -> Outcome {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        arguments.iter().map(OsString::from),
        environment,
        mapper,
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
            format!("Error: unknown option: --verbose\n\n{}", usage())
        )
    );
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../class-file/tests/fixtures")
}

fn project(files: &[(&str, &str)]) -> TempDir {
    let root = TempDir::new().expect("temp dir");
    for (name, text) in files {
        let path = root.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create directories");
        }
        fs::write(path, text).expect("write source");
    }
    root
}

fn arguments_for(root: &Path) -> Vec<String> {
    let fixtures = fixtures();
    [
        "--source",
        &root.display().to_string(),
        "--classpath",
        &fixtures.join("classes").display().to_string(),
        "--jdk-home",
        &fixtures.join("jdk").display().to_string(),
        "--release",
        "8",
    ]
    .map(str::to_owned)
    .to_vec()
}

fn check(root: &Path, extra: &[&str], mapper: Option<&Mapper>) -> Outcome {
    let mut arguments = arguments_for(root);
    arguments.extend(extra.iter().map(|argument| (*argument).to_owned()));
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    execute_with(&arguments, mapper)
}

fn order() -> Expression {
    Expression::parameter("order", TypeRef::class("shop.Order"))
}

fn clean_method() -> Method {
    Method::new("app.Shipping", "total", 3).call(Call::new(order(), "total", 4))
}

fn lines() -> TypeRef {
    TypeRef::generic("java.util.List", vec![TypeRef::class("shop.OrderLine")])
}

fn violating_method() -> Method {
    Method::new("app.Shipping", "discount", 3)
        .for_each(
            "line",
            Expression::return_of(Call::new(order(), "getLines", 4).returning(lines())),
        )
        .call(Call::new(Expression::local("line"), "applyDiscount", 5))
}

fn mapper_by_text(path: &Path, text: &str, types: &Types) -> Result<Declarations, ReadError> {
    match text {
        "probe" => Err(ReadError::new(
            path,
            &format!(
                "jdk={} classpath={} unknown={}",
                types.shape("sample.util.Box").is_some(),
                types.shape("shop.Order").is_some(),
                types.shape("app.Missing").is_some()
            ),
        )),
        "broken" => Err(ReadError::at_line(path, 1, "unexpected token")),
        "unresolved" => Ok(Declarations {
            classes: Vec::new(),
            methods: vec![
                Method::new("app.Shipping", "size", 3).call(
                    Call::new(
                        Expression::parameter("missing", TypeRef::class("app.Missing")),
                        "get",
                        4,
                    )
                    .then("size", 4),
                ),
            ],
        }),
        "hybrid" => Ok(Declarations {
            classes: vec!["shop.Account".to_owned()],
            methods: Vec::new(),
        }),
        "violation" => Ok(Declarations {
            classes: Vec::new(),
            methods: vec![violating_method()],
        }),
        _ => Ok(Declarations {
            classes: Vec::new(),
            methods: vec![clean_method()],
        }),
    }
}

#[test]
fn a_source_root_that_is_not_a_directory_fails() {
    let root = project(&[]);
    let missing = root.path().join("missing");

    let outcome = check(&missing, &[], Some(&mapper_by_text));

    assert_eq!(
        (outcome.code, outcome.out, outcome.err),
        (
            FAILURE,
            String::new(),
            format!("Error: source root not found: {}\n", missing.display())
        )
    );
}

#[test]
fn a_missing_jdk_fails() {
    let root = project(&[]);
    let source = root.path().display().to_string();

    let outcome = execute_with(&["--source", &source], Some(&mapper_by_text));

    assert_eq!(
        (outcome.code, outcome.err),
        (
            FAILURE,
            "Error: no JDK found: pass --jdk-home or set JAVA_HOME\n".to_owned()
        )
    );
}

#[test]
fn a_binary_without_a_source_mapper_fails_instead_of_pretending() {
    let root = project(&[("Order.java", "x")]);

    let outcome = check(root.path(), &[], None);

    assert_eq!(
        (outcome.code, outcome.out, outcome.err),
        (
            FAILURE,
            String::new(),
            "Error: no source parser is connected\n".to_owned()
        )
    );
}

#[test]
fn a_project_without_violations_reports_so_and_succeeds() {
    let root = project(&[("Order.java", "clean")]);

    let outcome = check(root.path(), &[], Some(&mapper_by_text));

    assert_eq!(
        (
            outcome.code,
            outcome.out.starts_with("No violations found.\n")
        ),
        (CLEAN, true)
    );
}

#[test]
fn a_violation_is_listed_and_fails_with_exit_code_one() {
    let root = project(&[("Shipping.java", "violation")]);
    let file = root.path().join("Shipping.java");

    let outcome = check(root.path(), &[], Some(&mapper_by_text));

    assert_eq!(
        (outcome.code, outcome.out.contains(&format!(
            "Violations (1):\n{}:5  app.Shipping.discount  applyDiscount()  Violation: loop variable line: the object returned by getLines() is not a friend of f\n",
            file.display()
        ))),
        (VIOLATIONS, true)
    );
}

#[test]
fn a_hybrid_class_is_listed_and_fails_with_exit_code_one() {
    let root = project(&[("Account.java", "hybrid")]);

    let outcome = check(root.path(), &[], Some(&mapper_by_text));

    assert_eq!(
        (outcome.code, outcome.out.contains("Hybrid classes (1):\n")),
        (VIOLATIONS, true)
    );
}

#[test]
fn unresolved_types_alone_do_not_fail_the_run() {
    let root = project(&[("Size.java", "unresolved")]);

    let outcome = check(root.path(), &[], Some(&mapper_by_text));

    assert_eq!(
        (
            outcome.code,
            outcome.out.contains("Unresolved types (1):\n")
        ),
        (CLEAN, true)
    );
}

#[test]
fn show_allowed_adds_the_allowed_calls_to_the_report() {
    let root = project(&[("Order.java", "clean")]);

    let hidden = check(root.path(), &[], Some(&mapper_by_text));
    let shown = check(root.path(), &["--show-allowed"], Some(&mapper_by_text));

    assert_eq!(
        (
            hidden.out.contains("Allowed calls"),
            shown.out.contains("Allowed calls (1):\n")
        ),
        (false, true)
    );
}

#[test]
fn a_file_that_does_not_parse_is_reported_and_the_others_are_still_checked() {
    let root = project(&[("Broken.java", "broken"), ("Order.java", "clean")]);
    let broken = root.path().join("Broken.java");

    let outcome = check(root.path(), &[], Some(&mapper_by_text));

    assert_eq!(
        (
            outcome.code,
            outcome.out.contains(&format!(
                "Source errors (1):\n{}:1  Error: unexpected token\n",
                broken.display()
            )),
            outcome.out.contains("  Files: 1\n")
        ),
        (CLEAN, true, true)
    );
}

#[test]
fn every_source_root_is_walked() {
    let first = project(&[("Order.java", "clean")]);
    let second = project(&[("Shipping.java", "violation")]);
    let mut arguments = arguments_for(first.path());
    arguments.extend(["--source".to_owned(), second.path().display().to_string()]);
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();

    let outcome = execute_with(&arguments, Some(&mapper_by_text));

    assert_eq!(
        (outcome.code, outcome.out.contains("  Files: 2\n")),
        (VIOLATIONS, true)
    );
}

#[test]
fn the_source_mapper_is_told_which_types_the_classpath_and_the_jdk_provide() {
    let root = project(&[("Probe.java", "probe")]);

    let outcome = check(root.path(), &[], Some(&mapper_by_text));

    assert!(
        outcome
            .out
            .contains("Error: jdk=true classpath=true unknown=false\n")
    );
}

#[cfg(unix)]
#[test]
fn a_directory_that_cannot_be_walked_shows_up_among_the_source_errors() {
    use std::os::unix::fs::PermissionsExt;

    let root = project(&[("Order.java", "clean")]);
    let locked = root.path().join("locked");
    fs::create_dir(&locked).expect("create directory");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("lock directory");
    let walkable = fs::read_dir(&locked).is_ok();

    let outcome = check(root.path(), &[], Some(&mapper_by_text));

    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("unlock directory");
    assert!(
        walkable
            || outcome.out.contains(&format!(
                "Source errors (1):\n{}  Error: ",
                locked.display()
            ))
    );
}

fn check_in_working_directory(directory: &Path) -> Outcome {
    let jdk = fixtures().join("jdk").display().to_string();
    let environment = Environment {
        working_directory: directory.to_path_buf(),
        ..nothing()
    };

    execute_in(
        &environment,
        &["--jdk-home", &jdk, "--release", "8"],
        Some(&mapper_by_text),
    )
}

#[test]
fn without_a_source_every_maven_source_root_under_the_working_directory_is_checked() {
    let root = project(&[
        ("src/main/java/app/Order.java", "clean"),
        ("orders/src/main/java/app/Shipping.java", "violation"),
    ]);

    let outcome = check_in_working_directory(root.path());

    assert!(outcome.out.contains("  Files: 2\n"), "{}", outcome.out);
}

#[test]
fn without_a_source_hidden_directories_are_not_searched() {
    let root = project(&[
        ("src/main/java/Order.java", "clean"),
        (".copy/src/main/java/Shipping.java", "violation"),
    ]);

    let outcome = check_in_working_directory(root.path());

    assert!(outcome.out.contains("  Files: 1\n"), "{}", outcome.out);
}

#[test]
fn without_a_source_the_compiled_classes_of_the_project_are_on_the_classpath() {
    let root = project(&[("src/main/java/Probe.java", "probe")]);
    let compiled = root.path().join("target/classes/shop");
    fs::create_dir_all(&compiled).expect("create class directory");
    fs::copy(
        fixtures().join("classes/shop/Order.class"),
        compiled.join("Order.class"),
    )
    .expect("copy class file");

    let outcome = check_in_working_directory(root.path());

    assert!(outcome.out.contains("classpath=true"), "{}", outcome.out);
}

#[test]
fn without_a_source_the_dependencies_maven_listed_are_on_the_classpath() {
    let listed = fixtures().join("classes").display().to_string();
    let root = project(&[
        ("src/main/java/Probe.java", "probe"),
        ("target/classpath.txt", &listed),
    ]);

    let outcome = check_in_working_directory(root.path());

    assert!(outcome.out.contains("classpath=true"), "{}", outcome.out);
}

#[test]
fn without_a_source_a_directory_without_a_maven_source_root_fails() {
    let root = project(&[("Order.java", "clean")]);

    let outcome = check_in_working_directory(root.path());

    assert_eq!(
        (outcome.code, outcome.out, outcome.err),
        (
            FAILURE,
            String::new(),
            "Error: no src/main/java under the working directory; pass a source root with --source\n"
                .to_owned()
        )
    );
}

#[test]
fn the_checked_source_roots_the_classpath_and_the_jdk_are_announced_first() {
    let root = project(&[
        ("src/main/java/Order.java", "clean"),
        ("orders/src/main/java/Shipping.java", "clean"),
    ]);
    fs::create_dir_all(root.path().join("target/classes")).expect("create class directory");

    let outcome = check_in_working_directory(root.path());

    let expected = format!(
        "Source root: {}\nSource root: {}\nClasspath: 1 entry\nJDK: {} (release 8)\n",
        root.path().join("orders/src/main/java").display(),
        root.path().join("src/main/java").display(),
        fixtures().join("jdk").display()
    );
    assert!(outcome.err.starts_with(&expected), "{}", outcome.err);
}
