#![cfg(unix)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn a_parameter_call_is_allowed_and_returns_zero() {
    let project = ContainerProject::compiled();
    let arguments = ["--only", "shop.OrderService#pay", "--show-allowed"];

    let result = project.analyze(&arguments);
    let report = text(&result.stdout);
    let context = text(&result.stderr);

    assert_eq!(result.status.code(), Some(0), "{report}\n{context}");
    assert!(
        report.contains("OrderService.java:5  shop.OrderService.pay  charge()  Allowed:"),
        "{report}"
    );
    assert!(
        context.contains("Result: no violations (exit code 0)"),
        "{context}"
    );
    assert_complete(&report);
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn a_returned_wallet_call_is_reported_with_its_reason_and_returns_one() {
    let project = ContainerProject::compiled();
    let arguments = [
        "--source",
        "src/main/java",
        "--classpath",
        "target/classes",
        "--only",
        "shop.OrderService#reachThrough",
    ];

    let result = project.analyze(&arguments);
    let report = text(&result.stdout);
    let context = text(&result.stderr);

    assert_eq!(result.status.code(), Some(1), "{report}\n{context}");
    assert!(
        report.contains(
            "OrderService.java:9  shop.OrderService.reachThrough  withdraw()  Violation: \
             the object returned by getWallet() is not a friend of f"
        ),
        "{report}"
    );
    assert!(
        context.contains("Result: 1 violation, 0 hybrid classes (exit code 1)"),
        "{context}"
    );
    assert_complete(&report);
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn an_unmatched_selection_returns_two() {
    let project = ContainerProject::compiled();
    let arguments = ["--only", "shop.Missing"];

    let result = project.analyze(&arguments);
    let context = text(&result.stderr);

    assert_eq!(
        result.status.code(),
        Some(2),
        "{}\n{context}",
        text(&result.stdout)
    );
    assert!(
        context.contains("nothing matches --only: shop.Missing"),
        "{context}"
    );
    assert!(
        context.contains("Result: check did not run (exit code 2)"),
        "{context}"
    );
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn an_inline_java_source_path_can_begin_with_a_dash() {
    let project = ContainerProject::compiled();
    fs::rename(
        project.path.join("src/main/java"),
        project.path.join("-src"),
    )
    .expect("source directory with a leading dash");
    let arguments = [
        "--classpath=target/classes",
        "--only=shop.OrderService#pay",
        "--show-allowed",
    ];

    let relative = project.analyze(&[&["--source=./-src"][..], &arguments].concat());
    let inline = project.analyze(&[&["--source=-src"][..], &arguments].concat());

    assert_eq!(
        relative.status.code(),
        Some(0),
        "{}",
        text(&relative.stderr)
    );
    assert_eq!(inline.status.code(), Some(0), "{}", text(&inline.stderr));
    let report = text(&inline.stdout);
    assert!(
        report.contains("OrderService.java:5  shop.OrderService.pay  charge()  Allowed:"),
        "{report}"
    );
    assert_complete(&report);
}

fn assert_complete(report: &str) {
    assert!(report.contains("unresolved 0)"), "{report}");
    assert!(report.contains("Source errors: 0"), "{report}");
    assert!(report.contains("Type reading errors: 0"), "{report}");
}

struct ContainerProject {
    _temporary: tempfile::TempDir,
    path: PathBuf,
    image: OsString,
    user: String,
}

impl ContainerProject {
    fn compiled() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let temporary_root = root.join("target/container-rust-smoke");
        fs::create_dir_all(&temporary_root).expect("temporary project root is writable");
        let temporary = tempfile::tempdir_in(temporary_root).expect("temporary project directory");
        let path = temporary.path().join("project with spaces");
        let source = path.join("src/main/java");
        fs::create_dir_all(&source).expect("source directory is writable");
        fs::copy(
            root.join("tests/container/OrderService.java"),
            source.join("OrderService.java"),
        )
        .expect("container fixture is readable");
        let uid = host_id("-u");
        let project = Self {
            _temporary: temporary,
            path: path.canonicalize().expect("project has an absolute path"),
            image: std::env::var_os("CONTAINER_IMAGE")
                .unwrap_or_else(|| OsString::from("law-of-demeter:local")),
            user: format!("{uid}:{}", host_id("-g")),
        };

        let compiled = project
            .command()
            .args(["--mount", &project.mount(), "--entrypoint", "javac"])
            .arg(&project.image)
            .args(["-d", "target/classes", "src/main/java/OrderService.java"])
            .output()
            .expect("Docker runs; these tests require Docker engine and a built image");

        assert!(
            compiled.status.success(),
            "container compilation failed ({}):\n{}{}",
            compiled.status,
            text(&compiled.stdout),
            text(&compiled.stderr)
        );
        for class in ["OrderService", "Order", "Wallet"] {
            let output = project
                .path
                .join(format!("target/classes/shop/{class}.class"));
            assert_eq!(
                fs::metadata(&output).expect("compiled fixture class").uid(),
                uid,
                "container created {} under a different user",
                output.display()
            );
        }
        project
    }

    fn analyze(&self, arguments: &[&str]) -> Output {
        self.command()
            .args(["--mount", &format!("{},readonly", self.mount())])
            .arg(&self.image)
            .args(arguments)
            .output()
            .expect("Docker runs; these tests require Docker engine and a built image")
    }

    fn command(&self) -> Command {
        let mut command = Command::new("docker");
        command.args(["run", "--rm", "--pull=never", "--user", &self.user]);
        command
    }

    fn mount(&self) -> String {
        format!("type=bind,src={},dst=/workspace", self.path.display())
    }
}

fn host_id(argument: &str) -> u32 {
    let result = Command::new("id")
        .arg(argument)
        .output()
        .expect("container tests require a macOS or Linux host with id on PATH");
    assert!(result.status.success(), "{}", text(&result.stderr));
    text(&result.stdout)
        .trim()
        .parse()
        .expect("host user and group IDs are numeric")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
