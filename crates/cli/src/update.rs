use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::run::{CLEAN, FAILURE};

const SKILL: &str = "law-of-demeter/SKILL.md";
const SOURCE_SKILL: &str = "skill/law-of-demeter/SKILL.md";
const CLI_PACKAGE: &str = "crates/cli";
const FIRST_INSTALLATION: &str = "npx skills add saadettinBerber/law-of-demeter --skill law-of-demeter --agent claude-code codex --global --copy";
const AGENT_DIRECTORIES: [&str; 3] = [".claude/skills", ".codex/skills", ".agents/skills"];

pub type Runner<'a> = dyn Fn(&Path, &str, &[&str]) -> Result<(), String> + 'a;

pub struct Installation {
    pub home: Option<PathBuf>,
    pub codex_home: Option<PathBuf>,
    pub claude_config_dir: Option<PathBuf>,
    pub working_directory: PathBuf,
    pub built_from: PathBuf,
}

impl Installation {
    pub fn current() -> Self {
        Self {
            home: env::var_os("HOME").map(PathBuf::from),
            codex_home: env::var_os("CODEX_HOME").map(PathBuf::from),
            claude_config_dir: env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from),
            working_directory: PathBuf::new(),
            built_from: Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        }
    }

    fn skills_directories(&self) -> Vec<PathBuf> {
        let user = self.home.as_ref().map(|home| home.join(".agents/skills"));
        let claude = self
            .claude_config_dir
            .clone()
            .or_else(|| self.home.as_ref().map(|home| home.join(".claude")))
            .map(|directory| directory.join("skills"));
        let codex = self
            .codex_home
            .clone()
            .or_else(|| self.home.as_ref().map(|home| home.join(".codex")))
            .map(|directory| directory.join("skills"));
        let project = AGENT_DIRECTORIES.map(|directory| self.working_directory.join(directory));
        [claude, codex, user]
            .into_iter()
            .flatten()
            .chain(project)
            .collect()
    }
}

pub fn run_command(directory: &Path, program: &str, arguments: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .status()
        .map_err(|error| format!("{program} could not start: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} {} failed: {status}",
            arguments.join(" ")
        ))
    }
}

pub fn update_usage() -> &'static str {
    "Usage: law-of-demeter update [OPTIONS]

Installs the CLI again from its source checkout and refreshes the agent skill
where it is already installed. The first installation of the skill is done with:
  npx skills add saadettinBerber/law-of-demeter --skill law-of-demeter

Options:
      --source <DIR>        Source checkout (default: the checkout the CLI was built from)
      --pull                Runs git pull --ff-only in the source checkout first
      --skills-dir <DIR>    Skills directory to refresh; can be given more than once;
                            replaces the defaults: the user and project .claude/skills,
                            .codex/skills and .agents/skills (CLAUDE_CONFIG_DIR and
                            CODEX_HOME replace the user .claude and .codex)
  -h, --help                Shows this help
"
}

struct Request {
    source: Option<PathBuf>,
    pull: bool,
    skills_directories: Vec<PathBuf>,
}

enum Parsed {
    Update(Request),
    Help,
}

pub fn update<I: IntoIterator<Item = OsString>>(
    arguments: I,
    installation: &Installation,
    runner: &Runner<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let result = match parse(arguments) {
        Ok(Parsed::Help) => {
            write!(out, "{}", update_usage()).ok();
            return CLEAN;
        }
        Ok(Parsed::Update(request)) => perform(request, installation, runner, out),
        Err(message) => Err(format!("{message}\n\n{}", update_usage())),
    };
    match result {
        Ok(()) => CLEAN,
        Err(message) => {
            writeln!(err, "Error: {message}").ok();
            FAILURE
        }
    }
}

fn parse<I: IntoIterator<Item = OsString>>(arguments: I) -> Result<Parsed, String> {
    let mut request = Request {
        source: None,
        pull: false,
        skills_directories: Vec::new(),
    };
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let option = argument.to_string_lossy();
        match option.as_ref() {
            "--help" | "-h" => return Ok(Parsed::Help),
            "--pull" => request.pull = true,
            "--source" => request.source = Some(value(&option, &mut arguments)?),
            "--skills-dir" => request
                .skills_directories
                .push(value(&option, &mut arguments)?),
            _ => return Err(format!("unknown argument: {option}")),
        }
    }
    Ok(Parsed::Update(request))
}

fn value(option: &str, arguments: &mut impl Iterator<Item = OsString>) -> Result<PathBuf, String> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{option} needs a value"))
}

fn perform(
    request: Request,
    installation: &Installation,
    runner: &Runner<'_>,
    out: &mut dyn Write,
) -> Result<(), String> {
    let source = request
        .source
        .unwrap_or_else(|| installation.built_from.clone());
    if !source.join(SOURCE_SKILL).is_file()
        || !source.join(CLI_PACKAGE).join("Cargo.toml").is_file()
    {
        return Err(format!(
            "{} is not a law-of-demeter checkout; pass one with --source",
            source.display()
        ));
    }
    writeln!(out, "Source: {}", source.display()).ok();
    if request.pull {
        runner(&source, "git", &["pull", "--ff-only"])?;
    }
    runner(
        &source,
        "cargo",
        &["install", "--locked", "--path", CLI_PACKAGE],
    )?;
    let skill = fs::read_to_string(source.join(SOURCE_SKILL)).map_err(|error| error.to_string())?;
    let directories = if request.skills_directories.is_empty() {
        installation.skills_directories()
    } else {
        request.skills_directories
    };
    let refreshed = refresh(&skill, &directories, out).map_err(|error| error.to_string())?;
    if refreshed == 0 {
        writeln!(
            out,
            "No installed skill found. Install it with:\n  {FIRST_INSTALLATION}"
        )
        .ok();
    }
    Ok(())
}

fn refresh(skill: &str, directories: &[PathBuf], out: &mut dyn Write) -> io::Result<usize> {
    let mut refreshed = 0;
    for installed in directories.iter().map(|directory| directory.join(SKILL)) {
        if !installed.is_file() {
            continue;
        }
        if fs::read_to_string(&installed)? == skill {
            writeln!(out, "Skill up to date: {}", installed.display()).ok();
        } else {
            fs::write(&installed, skill)?;
            writeln!(out, "Skill updated: {}", installed.display()).ok();
        }
        refreshed += 1;
    }
    Ok(refreshed)
}

#[cfg(test)]
mod tests;
