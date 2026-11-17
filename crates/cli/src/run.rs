use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use law_of_demeter_class_file::Jdk;
use law_of_demeter_core::application::{CheckProject, Counts, Selection, Target};

use crate::arguments::{Arguments, Command, UsageError, parse, usage};
use crate::class_file_types::ClassFileTypes;
use crate::jdk_choice::choose_jdk;
use crate::maven_layout::maven_layout;
use crate::report_text::render;
use crate::source_files::{Mapper, SourceFiles};
use crate::sources::java_files;

pub const CLEAN: u8 = 0;
pub const VIOLATIONS: u8 = 1;
pub const FAILURE: u8 = 2;

pub struct Environment {
    pub java_home: Option<OsString>,
    pub path: Option<OsString>,
    pub working_directory: PathBuf,
}

impl Environment {
    pub fn current() -> Self {
        Self {
            java_home: std::env::var_os("JAVA_HOME"),
            path: std::env::var_os("PATH"),
            working_directory: PathBuf::new(),
        }
    }
}

pub fn run<I: IntoIterator<Item = OsString>>(
    arguments: I,
    environment: &Environment,
    mapper: Option<&Mapper>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let (code, result) = match parse(arguments) {
        Ok(Command::Help) => {
            write!(out, "{}", usage()).ok();
            return CLEAN;
        }
        Ok(Command::Check(arguments)) => match check(arguments, environment, mapper, out, err) {
            Ok(counts) => {
                writeln!(err).ok();
                verdict(&counts)
            }
            Err(message) => {
                writeln!(err, "Error: {message}").ok();
                failed()
            }
        },
        Err(UsageError(message)) => {
            write!(err, "Error: {message}\n\n{}", usage()).ok();
            failed()
        }
    };
    writeln!(err, "Result: {result} (exit code {code})").ok();
    code
}

fn verdict(counts: &Counts) -> (u8, String) {
    if counts.violations + counts.hybrid_classes > 0 {
        let found = format!(
            "{}, {}",
            counted(counts.violations, "violation", "violations"),
            counted(counts.hybrid_classes, "hybrid class", "hybrid classes")
        );
        (VIOLATIONS, found)
    } else {
        (CLEAN, "no violations".to_owned())
    }
}

fn failed() -> (u8, String) {
    (FAILURE, "check did not run".to_owned())
}

fn counted(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

fn check(
    mut arguments: Arguments,
    environment: &Environment,
    mapper: Option<&Mapper>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<Counts, String> {
    if arguments.sources.is_empty() {
        let layout = maven_layout(&environment.working_directory);
        if layout.sources.is_empty() {
            return Err(
                "no src/main/java under the working directory; pass a source root with --source"
                    .to_owned(),
            );
        }
        arguments.sources = layout.sources;
        arguments.classpath.extend(layout.classpath);
    }
    if let Some(root) = arguments.sources.iter().find(|root| !root.is_dir()) {
        return Err(format!("source root not found: {}", root.display()));
    }
    let (jdk, release) = choose_jdk(
        &arguments,
        environment.java_home.clone(),
        environment.path.clone(),
    )
    .map_err(|UsageError(message)| message)?;
    let mapper = mapper.ok_or("no source parser is connected")?;
    announce(&arguments, &jdk, release, err);
    let walked = java_files(&arguments.sources);
    let sources = SourceFiles::new(walked.files, mapper).with_errors(walked.errors);
    let types = ClassFileTypes::new(arguments.classpath, Some((jdk, release)));
    let selection = if arguments.targets.is_empty() {
        Selection::Everything
    } else {
        Selection::Only(arguments.targets)
    };
    let report = CheckProject::new(&sources, &types)
        .with_selection(selection)
        .execute();
    if !report.unmatched_targets().is_empty() {
        let unmatched: Vec<String> = report.unmatched_targets().iter().map(target_text).collect();
        return Err(format!("nothing matches --only: {}", unmatched.join(", ")));
    }
    write!(out, "{}", render(&report, arguments.show_allowed)).ok();
    Ok(report.counts())
}

fn target_text(target: &Target) -> String {
    match target.method_name() {
        Some(method) => format!("{}#{method}", target.class_name()),
        None => target.class_name().to_owned(),
    }
}

fn announce(arguments: &Arguments, jdk: &Jdk, release: u32, err: &mut dyn Write) {
    for root in &arguments.sources {
        writeln!(err, "Source root: {}", root.display()).ok();
    }
    writeln!(
        err,
        "Classpath: {}",
        counted(arguments.classpath.len(), "entry", "entries")
    )
    .ok();
    writeln!(err, "JDK: {} (release {release})\n", jdk.home().display()).ok();
}

#[cfg(test)]
mod tests;
