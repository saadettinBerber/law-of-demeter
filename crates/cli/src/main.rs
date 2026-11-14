use std::io::{stderr, stdout};
use std::process::ExitCode;

use law_of_demeter_cli::{Environment, run};

fn main() -> ExitCode {
    let code = run(
        std::env::args_os().skip(1),
        &Environment::current(),
        None,
        &mut stdout().lock(),
        &mut stderr().lock(),
    );
    ExitCode::from(code)
}
