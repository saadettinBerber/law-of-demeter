use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Arguments {
    pub sources: Vec<PathBuf>,
    pub classpath: Vec<PathBuf>,
    pub jdk_home: Option<PathBuf>,
    pub release: Option<u32>,
    pub show_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Check(Arguments),
    Help,
}

pub fn usage() -> &'static str {
    "Usage: law-of-demeter --source <DIR>... [OPTIONS]

Finds Law of Demeter violations in Java source code.

Options:
  -s, --source <DIR>        Source root; can be given more than once (required)
  -c, --classpath <PATH>    Directory or jar; can be given more than once or as a ':'-separated list
      --jdk-home <DIR>      JDK directory (default: JAVA_HOME, then javac on PATH)
      --release <VERSION>   Java release to read (default: the JDK's own release)
      --show-allowed        Also lists the allowed calls with the rule they follow
  -h, --help                Shows this help

Exit code: 0 no violations, 1 violations found, 2 usage or fatal error.
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
            _ if option.starts_with('-') => {
                return Err(UsageError(format!("unknown option: {option}")));
            }
            _ => return Err(UsageError(format!("unexpected argument: {option}"))),
        }
    }
    if parsed.sources.is_empty() {
        return Err(UsageError("at least one --source is required".to_owned()));
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
