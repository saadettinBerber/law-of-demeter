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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError(pub String);

pub fn parse<I: IntoIterator<Item = OsString>>(arguments: I) -> Result<Command, UsageError> {
    let mut parsed = Arguments::default();
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        if argument == "--source" || argument == "-s" {
            parsed.sources.extend(arguments.next().map(PathBuf::from));
        } else if argument == "--show-allowed" {
            parsed.show_allowed = true;
        } else if argument == "--jdk-home" {
            parsed.jdk_home = arguments.next().map(PathBuf::from);
        } else if argument == "--release" {
            parsed.release = arguments
                .next()
                .and_then(|value| value.to_str()?.parse().ok());
        } else if argument == "--classpath" || argument == "-c" {
            parsed.classpath.extend(
                arguments
                    .next()
                    .iter()
                    .flat_map(env::split_paths)
                    .filter(|entry| !entry.as_os_str().is_empty()),
            );
        }
    }
    Ok(Command::Check(parsed))
}

#[cfg(test)]
mod tests;
