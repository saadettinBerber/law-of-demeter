use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use law_of_demeter_core::application::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Arguments {
    pub sources: Vec<PathBuf>,
    pub classpath: Vec<PathBuf>,
    pub jdk_home: Option<PathBuf>,
    pub release: Option<u32>,
    pub show_allowed: bool,
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Check(Arguments),
    Help,
}

pub fn usage() -> &'static str {
    "Usage: law-of-demeter [OPTIONS]

Finds Law of Demeter violations in Java source code.

Without --source, the Maven project in the working directory is checked: every
src/main/java directory is a source root, and every target/classes directory and
the entries of every target/classpath.txt file join the classpath. Hidden
directories are skipped.

With --only, every source root is still read, so the types of the whole project
are known; only the selected classes and methods are judged and counted.

Options:
  -s, --source <DIR>        Source root; can be given more than once
  -c, --classpath <PATH>    Directory or jar; can be given more than once or as a ':'-separated list
      --jdk-home <DIR>      JDK directory (default: JAVA_HOME, then javac on PATH)
      --release <VERSION>   Java release to read (default: the JDK's own release)
      --show-allowed        Also lists the allowed calls with the rule they follow
      --only <TARGET>       Checks only a class (shop.Order) with its nested classes, or a
                            method (shop.Order#pay); can be given more than once
  -h, --help                Shows this help

The source roots, the classpath and the JDK are written to standard error before
the check, and the result with its exit code comes last. Exit code: 0 no
violations, 1 violations found, 2 usage or fatal error.
"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError(pub String);

pub fn parse<I: IntoIterator<Item = OsString>>(arguments: I) -> Result<Command, UsageError> {
    let mut parsed = Arguments::default();
    let mut arguments = arguments.into_iter().flat_map(split_at_equals);
    while let Some(argument) = arguments.next() {
        let option = argument.to_string_lossy();
        match option.as_ref() {
            "--help" | "-h" => return Ok(Command::Help),
            "--source" | "-s" => parsed.sources.push(value(&option, &mut arguments)?.into()),
            "--classpath" | "-c" => parsed.classpath.extend(
                env::split_paths(&value(&option, &mut arguments)?)
                    .filter(|entry| !entry.as_os_str().is_empty()),
            ),
            "--jdk-home" => parsed.jdk_home = Some(value(&option, &mut arguments)?.into()),
            "--release" => parsed.release = Some(release(&value(&option, &mut arguments)?)?),
            "--show-allowed" => parsed.show_allowed = true,
            "--only" => parsed
                .targets
                .push(target(&value(&option, &mut arguments)?)?),
            _ if option.starts_with('-') => {
                return Err(UsageError(format!("unknown option: {option}")));
            }
            _ => return Err(UsageError(format!("unexpected argument: {option}"))),
        }
    }
    Ok(Command::Check(parsed))
}

fn split_at_equals(argument: OsString) -> Vec<OsString> {
    let split = argument
        .to_str()
        .filter(|text| text.starts_with("--"))
        .and_then(|text| text.split_once('='));
    match split {
        Some((option, value)) => vec![option.into(), value.into()],
        None => vec![argument],
    }
}

fn value(
    option: &str,
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<OsString, UsageError> {
    arguments
        .next()
        .ok_or_else(|| UsageError(format!("{option} needs a value")))
}

fn target(value: &OsString) -> Result<Target, UsageError> {
    let text = value.to_string_lossy();
    match text.split_once('#') {
        None if !text.is_empty() => Ok(Target::class(&text)),
        Some((class, method)) if !class.is_empty() && !method.is_empty() => {
            Ok(Target::method(class, method))
        }
        _ => Err(UsageError(format!(
            "--only needs a class, optionally followed by #method: {text}"
        ))),
    }
}

fn release(value: &OsString) -> Result<u32, UsageError> {
    value
        .to_str()
        .and_then(|text| text.parse().ok())
        .ok_or_else(|| {
            UsageError(format!(
                "--release must be a version number: {}",
                value.to_string_lossy()
            ))
        })
}

#[cfg(test)]
mod tests;
