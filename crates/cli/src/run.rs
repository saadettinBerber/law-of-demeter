use std::ffi::OsString;
use std::io::Write;

use crate::arguments::{Arguments, Command, UsageError, parse, usage};
use crate::jdk_choice::choose_jdk;
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
    match parse(arguments) {
        Ok(Command::Help) => {
            write!(out, "{}", usage()).ok();
            CLEAN
        }
        Ok(Command::Check(arguments)) => check(&arguments, environment, mapper, err),
        Err(UsageError(message)) => {
            write!(err, "Hata: {message}\n\n{}", usage()).ok();
            FAILURE
        }
    }
}

fn check(
    arguments: &Arguments,
    environment: &Environment,
    mapper: Option<&Mapper>,
    err: &mut dyn Write,
) -> u8 {
    let mut fail = |message: String| {
        writeln!(err, "Hata: {message}").ok();
        FAILURE
    };
    if let Some(root) = arguments.sources.iter().find(|root| !root.is_dir()) {
        return fail(format!("kaynak kökü bulunamadı: {}", root.display()));
    }
    if let Err(UsageError(message)) = choose_jdk(
        arguments,
        environment.java_home.clone(),
        environment.path.clone(),
    ) {
        return fail(message);
    }
    if mapper.is_none() {
        return fail("kaynak ayrıştırıcı bağlı değil".to_owned());
    }
    CLEAN
}

#[cfg(test)]
mod tests;
