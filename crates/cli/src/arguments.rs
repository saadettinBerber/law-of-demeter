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
        }
    }
    Ok(Command::Check(parsed))
}

#[cfg(test)]
mod tests;
