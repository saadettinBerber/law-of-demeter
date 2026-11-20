use std::io::{stderr, stdout};
use std::path::Path;
use std::process::ExitCode;

use law_of_demeter_cli::{Declarations, Environment, Installation, run, run_command, update};
use law_of_demeter_core::application::ReadError;
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_java_source::map_source;

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1).peekable();
    if arguments.next_if(|argument| argument == "update").is_some() {
        return ExitCode::from(update(
            arguments,
            &Installation::current(),
            &run_command,
            &mut stdout().lock(),
            &mut stderr().lock(),
        ));
    }
    let mapper = |path: &Path, source: &str, types: &Types| {
        map_source(path, source, types)
            .map(|file| Declarations {
                classes: file.classes,
                methods: file.methods,
            })
            .map_err(|error| ReadError::at_line(path, error.line, &error.message))
    };
    let code = run(
        arguments,
        &Environment::current(),
        Some(&mapper),
        &mut stdout().lock(),
        &mut stderr().lock(),
    );
    ExitCode::from(code)
}
