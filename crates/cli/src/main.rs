use std::io::{stderr, stdout};
use std::path::Path;
use std::process::ExitCode;

use law_of_demeter_cli::{Declarations, Environment, run};
use law_of_demeter_core::application::ReadError;
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_java_source::map_source;

fn main() -> ExitCode {
    let mapper = |path: &Path, source: &str, types: &Types| {
        map_source(path, source, types)
            .map(|file| Declarations {
                classes: file.classes,
                methods: file.methods,
            })
            .map_err(|error| ReadError::at_line(path, error.line, &error.message))
    };
    let code = run(
        std::env::args_os().skip(1),
        &Environment::current(),
        Some(&mapper),
        &mut stdout().lock(),
        &mut stderr().lock(),
    );
    ExitCode::from(code)
}
