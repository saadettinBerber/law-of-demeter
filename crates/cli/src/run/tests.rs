use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use law_of_demeter_core::application::ReadError;
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
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        arguments.iter().map(OsString::from),
        &nothing(),
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
            format!("Hata: bilinmeyen seçenek: --verbose\n\n{}", usage())
        )
    );
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../class-file/tests/fixtures")
}

fn project(files: &[(&str, &str)]) -> TempDir {
    let root = TempDir::new().expect("temp dir");
    for (name, text) in files {
        fs::write(root.path().join(name), text).expect("write source");
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

fn violating_method() -> Method {
    Method::new("app.Shipping", "discount", 3)
        .for_each(
            "line",
            Expression::return_of(Call::new(order(), "getLines", 4)),
        )
        .call(Call::new(Expression::local("line"), "applyDiscount", 5))
}

fn mapper_by_text(
    path: &Path,
    text: &str,
    _: &dyn Fn(&str) -> bool,
) -> Result<Declarations, ReadError> {
    match text {
        "broken" => Err(ReadError::at_line(path, 1, "beklenmeyen simge")),
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
            format!("Hata: kaynak kökü bulunamadı: {}\n", missing.display())
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
            "Hata: JDK bulunamadı: --jdk-home verin ya da JAVA_HOME ayarlayın\n".to_owned()
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
            "Hata: kaynak ayrıştırıcı bağlı değil\n".to_owned()
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
            outcome.out.starts_with("İhlal bulunamadı.\n"),
            outcome.err
        ),
        (CLEAN, true, String::new())
    );
}

#[test]
fn a_violation_is_listed_and_fails_with_exit_code_one() {
    let root = project(&[("Shipping.java", "violation")]);
    let file = root.path().join("Shipping.java");

    let outcome = check(root.path(), &[], Some(&mapper_by_text));

    assert_eq!(
        (outcome.code, outcome.out.contains(&format!(
            "İhlaller (1):\n{}:5  app.Shipping.discount  applyDiscount()  İhlal: line döngü değişkeni: getLines() çağrısının döndürdüğü nesne f'nin arkadaşı değil\n",
            file.display()
        ))),
        (VIOLATIONS, true)
    );
}
