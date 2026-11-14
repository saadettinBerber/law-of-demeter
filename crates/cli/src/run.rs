use std::ffi::OsString;
use std::io::Write;

use crate::arguments::{Command, parse, usage};
use crate::source_files::Mapper;

pub const CLEAN: u8 = 0;
pub const VIOLATIONS: u8 = 1;
pub const FAILURE: u8 = 2;

pub struct Environment {
    pub java_home: Option<OsString>,
    pub path: Option<OsString>,
}

impl Environment {
    pub fn current() -> Self {
        Self {
            java_home: std::env::var_os("JAVA_HOME"),
            path: std::env::var_os("PATH"),
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
    let _ = (environment, mapper, err);
    match parse(arguments) {
        Ok(Command::Help) => {
            write!(out, "{}", usage()).ok();
            CLEAN
        }
        _ => FAILURE,
    }
}

#[cfg(test)]
mod tests;
