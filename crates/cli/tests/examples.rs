use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const EXPECTED_REPORT: &str = "expected-report.txt";
const SOURCE_ROOT: &str = "src/main/java";
const CLASSPATH_FILE: &str = "classpath.txt";
const NO_VIOLATIONS: &str = "No violations found.";
const CONTEXT_LINES: usize = 2;

#[test]
#[ignore = "needs Maven, a JDK and a warm ~/.m2; run with --ignored"]
fn every_example_project_reports_what_its_expected_report_says() {
    let examples = example_directories();
    assert!(
        !examples.is_empty(),
        "no directory under examples/ has an {EXPECTED_REPORT}"
    );

    let failures: Vec<String> = examples
        .iter()
        .filter_map(|example| check(example).err())
        .collect();

    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

fn example_directories() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut found: Vec<PathBuf> = fs::read_dir(&root)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default();
    found.retain(|directory| directory.join(EXPECTED_REPORT).is_file());
    found.sort();
    found
}

fn check(example: &Path) -> Result<(), String> {
    let name = example
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    let expected = fs::read_to_string(example.join(EXPECTED_REPORT))
        .map_err(|error| format!("{name}: cannot read {EXPECTED_REPORT}: {error}"))?;
    build(example).map_err(|error| format!("{name}: {error}"))?;
    let output = run_tool(example).map_err(|error| format!("{name}: {error}"))?;

    let problems = problems_with(&expected, &output);

    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!("{name}:\n{}", problems.join("\n")))
    }
}

fn build(example: &Path) -> Result<(), String> {
    let output = Command::new("mvn")
        .current_dir(example)
        .args(["-o", "-q", "clean", "compile", "dependency:build-classpath"])
        .arg(format!("-Dmdep.outputFile=target/{CLASSPATH_FILE}"))
        .output()
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => {
                "Maven (mvn) was not found on PATH; the examples need Maven, a JDK and a warm ~/.m2"
                    .to_owned()
            }
            _ => format!("cannot run Maven: {error}"),
        })?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "the offline Maven build failed ({}); the examples need a warm ~/.m2\n{}{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    ))
}

fn run_tool(example: &Path) -> Result<Output, String> {
    let layout = Layout::of(example)
        .map_err(|error| format!("cannot scan the example directory: {error}"))?;
    if layout.sources.is_empty() {
        return Err(format!("no {SOURCE_ROOT} directory found"));
    }
    let mut command = Command::new(env!("CARGO_BIN_EXE_law-of-demeter"));
    command.current_dir(example).arg("--show-allowed");
    for source in &layout.sources {
        command.arg("--source").arg(source);
    }
    for entry in &layout.classpath {
        command.arg("--classpath").arg(entry);
    }
    command
        .output()
        .map_err(|error| format!("cannot run the tool: {error}"))
}

fn problems_with(expected: &str, output: &Output) -> Vec<String> {
    let actual = text(&output.stdout);
    let mut problems = Vec::new();
    if actual != expected {
        problems.push(format!(
            "the report differs (- expected, + actual):\n{}",
            line_diff(expected, &actual)
        ));
    }
    let implied = implied_exit_code(expected);
    if output.status.code() != Some(implied) {
        problems.push(format!(
            "exit code {:?}, but the expected report implies {implied}\nstderr:\n{}",
            output.status.code(),
            text(&output.stderr)
        ));
    }
    problems
}

fn implied_exit_code(expected_report: &str) -> i32 {
    if expected_report.starts_with(NO_VIOLATIONS) {
        0
    } else {
        1
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

struct Layout {
    sources: Vec<PathBuf>,
    classpath: Vec<PathBuf>,
}

impl Layout {
    fn of(example: &Path) -> io::Result<Self> {
        let mut layout = Self {
            sources: Vec::new(),
            classpath: Vec::new(),
        };
        layout.scan(example, Path::new(""))?;
        layout.classpath = without_duplicates(example, std::mem::take(&mut layout.classpath));
        Ok(layout)
    }

    fn scan(&mut self, example: &Path, relative: &Path) -> io::Result<()> {
        let mut entries = fs::read_dir(example.join(relative))?.collect::<io::Result<Vec<_>>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries.iter().filter(|entry| is_directory(entry)) {
            let child = relative.join(entry.file_name());
            if child.ends_with(SOURCE_ROOT) {
                self.sources.push(child);
            } else if entry.file_name() == OsStr::new("target") {
                self.read_target(example, &child)?;
            } else {
                self.scan(example, &child)?;
            }
        }
        Ok(())
    }

    fn read_target(&mut self, example: &Path, target: &Path) -> io::Result<()> {
        let classes = target.join("classes");
        if example.join(&classes).is_dir() {
            self.classpath.push(classes);
        }
        let dependencies = example.join(target).join(CLASSPATH_FILE);
        if dependencies.is_file() {
            let listed = fs::read_to_string(dependencies)?;
            self.classpath.extend(
                env::split_paths(listed.trim()).filter(|entry| !entry.as_os_str().is_empty()),
            );
        }
        Ok(())
    }
}

fn is_directory(entry: &fs::DirEntry) -> bool {
    entry.file_type().is_ok_and(|kind| kind.is_dir())
}

fn without_duplicates(example: &Path, entries: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    entries
        .into_iter()
        .filter(|entry| {
            let location = example.join(entry);
            seen.insert(fs::canonicalize(&location).unwrap_or(location))
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Change<'a> {
    Same(&'a str),
    Removed(&'a str),
    Added(&'a str),
}

fn line_diff(expected: &str, actual: &str) -> String {
    let changes = changes_between(
        &expected.lines().collect::<Vec<_>>(),
        &actual.lines().collect::<Vec<_>>(),
    );
    if changes
        .iter()
        .all(|change| matches!(change, Change::Same(_)))
    {
        return "  the lines are the same; the final newline or the line endings differ".to_owned();
    }
    let near_change = |index: usize| {
        let from = index.saturating_sub(CONTEXT_LINES);
        let to = (index + CONTEXT_LINES + 1).min(changes.len());
        changes[from..to]
            .iter()
            .any(|change| !matches!(change, Change::Same(_)))
    };
    let mut lines = Vec::new();
    let mut skipping = false;
    for (index, change) in changes.iter().enumerate() {
        if !near_change(index) {
            if !skipping {
                lines.push("  ...".to_owned());
            }
            skipping = true;
            continue;
        }
        skipping = false;
        lines.push(match change {
            Change::Same(line) => format!("  {line}"),
            Change::Removed(line) => format!("- {line}"),
            Change::Added(line) => format!("+ {line}"),
        });
    }
    lines.join("\n")
}

fn changes_between<'a>(expected: &[&'a str], actual: &[&'a str]) -> Vec<Change<'a>> {
    let mut common = vec![vec![0usize; actual.len() + 1]; expected.len() + 1];
    for i in (0..expected.len()).rev() {
        for j in (0..actual.len()).rev() {
            common[i][j] = if expected[i] == actual[j] {
                common[i + 1][j + 1] + 1
            } else {
                common[i + 1][j].max(common[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut changes = Vec::new();
    while i < expected.len() && j < actual.len() {
        if expected[i] == actual[j] {
            changes.push(Change::Same(expected[i]));
            i += 1;
            j += 1;
        } else if common[i + 1][j] >= common[i][j + 1] {
            changes.push(Change::Removed(expected[i]));
            i += 1;
        } else {
            changes.push(Change::Added(actual[j]));
            j += 1;
        }
    }
    changes.extend(expected[i..].iter().map(|line| Change::Removed(line)));
    changes.extend(actual[j..].iter().map(|line| Change::Added(line)));
    changes
}
