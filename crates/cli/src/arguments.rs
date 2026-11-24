use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use law_of_demeter_core::application::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Arguments {
    pub rust: Option<Box<RustArguments>>,
    pub sources: Vec<PathBuf>,
    pub classpath: Vec<PathBuf>,
    pub jdk_home: Option<PathBuf>,
    pub release: Option<u32>,
    pub show_allowed: bool,
    pub targets: Vec<Target>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RustArguments {
    pub manifest: PathBuf,
    pub package: String,
    pub features: Vec<String>,
    pub no_default_features: bool,
    pub target: Option<String>,
    pub target_dir: Option<PathBuf>,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Check(Arguments),
    Help,
    Skill,
}

pub fn usage() -> &'static str {
    "Usage: law-of-demeter [OPTIONS]
       law-of-demeter update [OPTIONS]

Finds Law of Demeter violations in Java source code and a limited Rust source slice.

Without --rust or --source, the Maven project in the working directory is checked: every
src/main/java directory is a source root, and every target/classes directory and
the entries of every target/classpath.txt file join the classpath. Hidden
directories are skipped.

With --only, every source root is still read, so the types of the whole project
are known; only the selected classes and methods are judged and counted.

Options:
      --rust <Cargo.toml>   Check a Rust library (requires --package; no JDK or Maven)
      --package <NAME>      Cargo package with one normal library target
      --features <LIST>     Cargo features, comma/space separated; repeatable
      --no-default-features Disable default Cargo features
      --target <TRIPLE>     Cargo target platform
      --target-dir <DIR>    Writable analysis/build output (default: workspace target/law-of-demeter)
  -s, --source <DIR>        Source root; can be given more than once
  -c, --classpath <PATH>    Directory or jar; can be given more than once or as a list joined by
                            the platform's path separator (':', on Windows ';')
      --jdk-home <DIR>      JDK directory (default: JAVA_HOME, then javac on PATH)
      --release <VERSION>   Java release to read (default: the JDK's own release)
      --show-allowed        Also lists the allowed calls with the rule they follow
      --only <TARGET>       Checks only a class (shop.Order) with its nested classes, or a
                            method (shop.Order#pay); can be given more than once; quote
                            names that contain $
      --skill               Shows the guide for coding agents that run this tool
  -h, --help                Shows this help

Commands:
  update                    Installs the CLI again from its source checkout and
                            refreshes the installed agent skill; see update --help

For Java, the source roots, classpath and JDK are written to standard error before
the check, and the result with its exit code comes last. Exit code: 0 no
violations, 1 violations found, 2 usage or fatal error.
Rust additionally returns 3 for incomplete analysis, retaining known findings.
Rust --only accepts package/target::module::Type#method or a unique suffix.
Rust options cannot be combined with Java source/classpath/JDK/release options.
"
}

pub fn skill() -> &'static str {
    include_str!("../../../skill/law-of-demeter/SKILL.md")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError(pub String);

pub fn parse<I: IntoIterator<Item = OsString>>(arguments: I) -> Result<Command, UsageError> {
    let mut parsed = Arguments::default();
    let mut rust = RustArguments::default();
    let mut rust_selected = false;
    let mut rust_configured = false;
    let mut java_configured = false;
    let mut targets = Vec::new();
    let mut arguments = arguments.into_iter().flat_map(split_at_equals);
    while let Some(argument) = arguments.next() {
        let option = argument.text.to_string_lossy();
        match option.as_ref() {
            "--help" | "-h" => return Ok(Command::Help),
            "--skill" => return Ok(Command::Skill),
            "--rust" => {
                rust.manifest = value(&option, &mut arguments)?.into();
                rust_selected = true;
            }
            "--package" => {
                rust.package = value(&option, &mut arguments)?
                    .to_string_lossy()
                    .into_owned();
                rust_configured = true;
            }
            "--features" => {
                rust.features.extend(
                    value(&option, &mut arguments)?
                        .to_string_lossy()
                        .split(|character: char| character == ',' || character.is_whitespace())
                        .filter(|feature| !feature.is_empty())
                        .map(str::to_owned),
                );
                rust_configured = true;
            }
            "--no-default-features" => {
                rust.no_default_features = true;
                rust_configured = true;
            }
            "--target" => {
                rust.target = Some(
                    value(&option, &mut arguments)?
                        .to_string_lossy()
                        .into_owned(),
                );
                rust_configured = true;
            }
            "--target-dir" => {
                rust.target_dir = Some(value(&option, &mut arguments)?.into());
                rust_configured = true;
            }
            "--source" | "-s" => {
                java_configured = true;
                parsed.sources.push(value(&option, &mut arguments)?.into());
            }
            "--classpath" | "-c" => {
                java_configured = true;
                parsed.classpath.extend(
                    env::split_paths(&value(&option, &mut arguments)?)
                        .filter(|entry| !entry.as_os_str().is_empty()),
                );
            }
            "--jdk-home" => {
                java_configured = true;
                parsed.jdk_home = Some(value(&option, &mut arguments)?.into());
            }
            "--release" => {
                java_configured = true;
                parsed.release = Some(release(&value(&option, &mut arguments)?)?);
            }
            "--show-allowed" => parsed.show_allowed = true,
            "--only" => targets.push(value(&option, &mut arguments)?),
            _ if option.starts_with('-') => {
                return Err(UsageError(format!("unknown option: {option}")));
            }
            _ => return Err(UsageError(format!("unexpected argument: {option}"))),
        }
    }
    if rust_selected {
        if java_configured {
            return Err(UsageError(
                "--rust cannot be combined with Java options".into(),
            ));
        }
        if rust.package.is_empty() {
            return Err(UsageError("--rust requires --package <NAME>".into()));
        }
        rust.targets = targets
            .into_iter()
            .map(|target| target.to_string_lossy().into_owned())
            .collect();
        if rust.targets.iter().any(String::is_empty) {
            return Err(UsageError("--only needs a nonempty target".into()));
        }
        parsed.rust = Some(Box::new(rust));
    } else {
        if rust_configured {
            return Err(UsageError(
                "Rust options require --rust <Cargo.toml>".into(),
            ));
        }
        parsed.targets = targets.iter().map(target).collect::<Result<_, _>>()?;
    }
    Ok(Command::Check(parsed))
}

struct Token {
    text: OsString,
    attached: bool,
}

fn split_at_equals(argument: OsString) -> Vec<Token> {
    let split = argument
        .to_str()
        .filter(|text| text.starts_with("--"))
        .and_then(|text| text.split_once('='));
    match split {
        Some((option, value)) => vec![
            Token {
                text: option.into(),
                attached: false,
            },
            Token {
                text: value.into(),
                attached: true,
            },
        ],
        None => vec![Token {
            text: argument,
            attached: false,
        }],
    }
}

fn value(
    option: &str,
    arguments: &mut impl Iterator<Item = Token>,
) -> Result<OsString, UsageError> {
    arguments
        .next()
        .filter(|value| value.attached || !value.text.to_string_lossy().starts_with('-'))
        .map(|value| value.text)
        .ok_or_else(|| UsageError(format!("{option} needs a value")))
}

fn target(value: &OsString) -> Result<Target, UsageError> {
    let text = value.to_string_lossy();
    match text.split_once('#') {
        None if !text.is_empty() => Ok(Target::class(&text)),
        Some((class, method))
            if !class.is_empty() && !method.is_empty() && !method.contains('#') =>
        {
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
