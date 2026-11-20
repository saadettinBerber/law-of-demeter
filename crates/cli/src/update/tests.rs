use std::cell::RefCell;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::{Installation, update};
use crate::run::{CLEAN, FAILURE};

const OLD_SKILL: &str = "old skill\n";
const NEW_SKILL: &str = "new skill\n";

struct Machine {
    root: TempDir,
}

impl Machine {
    fn new() -> Self {
        let machine = Self {
            root: tempfile::tempdir().expect("temporary directory"),
        };
        machine.write("source/skill/law-of-demeter/SKILL.md", NEW_SKILL);
        machine.write("source/crates/cli/Cargo.toml", "");
        machine
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.path().join(relative)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().expect("parent directory")).expect("directory");
        fs::write(path, content).expect("file");
    }

    fn read(&self, relative: &str) -> Option<String> {
        fs::read_to_string(self.path(relative)).ok()
    }

    fn installation(&self) -> Installation {
        Installation {
            home: Some(self.path("home")),
            codex_home: None,
            claude_config_dir: None,
            working_directory: self.path("project"),
            built_from: self.path("source"),
        }
    }
}

type Commands = Vec<(PathBuf, String)>;

struct Outcome {
    code: u8,
    commands: Commands,
    out: String,
}

fn execute(arguments: &[&str], installation: &Installation) -> Outcome {
    execute_failing(arguments, installation, None)
}

fn execute_failing(
    arguments: &[&str],
    installation: &Installation,
    failing_program: Option<&str>,
) -> Outcome {
    let commands = RefCell::new(Commands::new());
    let runner = |directory: &Path, program: &str, arguments: &[&str]| {
        commands.borrow_mut().push((
            directory.to_path_buf(),
            format!("{program} {}", arguments.join(" ")),
        ));
        match failing_program {
            Some(failing) if failing == program => Err(format!("{program} failed")),
            _ => Ok(()),
        }
    };
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = update(
        arguments.iter().map(OsString::from),
        installation,
        &runner,
        &mut out,
        &mut err,
    );
    Outcome {
        code,
        commands: commands.into_inner(),
        out: String::from_utf8_lossy(&out).into_owned(),
    }
}

#[test]
fn the_cli_is_installed_from_the_checkout_it_was_built_from() {
    let machine = Machine::new();

    let outcome = execute(&[], &machine.installation());

    assert_eq!(
        (outcome.code, outcome.commands),
        (
            CLEAN,
            vec![(
                machine.path("source"),
                "cargo install --locked --path crates/cli".to_owned()
            )]
        )
    );
}

#[test]
fn pull_fetches_the_checkout_before_installing() {
    let machine = Machine::new();

    let outcome = execute(&["--pull"], &machine.installation());

    assert_eq!(
        outcome
            .commands
            .into_iter()
            .map(|(_, command)| command)
            .collect::<Vec<_>>(),
        [
            "git pull --ff-only",
            "cargo install --locked --path crates/cli"
        ]
    );
}

#[test]
fn an_explicit_source_replaces_the_checkout_it_was_built_from() {
    let machine = Machine::new();
    machine.write("other/skill/law-of-demeter/SKILL.md", NEW_SKILL);
    machine.write("other/crates/cli/Cargo.toml", "");

    let outcome = execute(
        &["--source", &machine.path("other").to_string_lossy()],
        &machine.installation(),
    );

    assert_eq!(
        outcome
            .commands
            .first()
            .map(|(directory, _)| directory.clone()),
        Some(machine.path("other"))
    );
}

#[test]
fn a_source_without_the_skill_is_rejected_before_anything_runs() {
    let machine = Machine::new();
    let installation = Installation {
        built_from: machine.path("elsewhere"),
        ..machine.installation()
    };

    let outcome = execute(&[], &installation);

    assert_eq!((outcome.code, outcome.commands), (FAILURE, Commands::new()));
}

#[test]
fn an_installed_skill_is_replaced_with_the_skill_of_the_source() {
    let machine = Machine::new();
    machine.write("agents/law-of-demeter/SKILL.md", OLD_SKILL);

    let outcome = execute(
        &["--skills-dir", &machine.path("agents").to_string_lossy()],
        &machine.installation(),
    );

    assert_eq!(
        (outcome.code, machine.read("agents/law-of-demeter/SKILL.md")),
        (CLEAN, Some(NEW_SKILL.to_owned()))
    );
}

#[test]
fn a_skills_directory_without_the_skill_gets_no_first_installation() {
    let machine = Machine::new();
    machine.write("agents/other/SKILL.md", OLD_SKILL);

    let outcome = execute(
        &["--skills-dir", &machine.path("agents").to_string_lossy()],
        &machine.installation(),
    );

    assert_eq!(
        (outcome.code, machine.read("agents/law-of-demeter/SKILL.md")),
        (CLEAN, None)
    );
}

#[test]
fn without_an_installed_skill_the_first_installation_command_is_shown() {
    let machine = Machine::new();

    let outcome = execute(&[], &machine.installation());

    assert!(
        outcome
            .out
            .contains("npx skills add saadettinBerber/law-of-demeter --skill law-of-demeter"),
        "{}",
        outcome.out
    );
}

#[test]
fn a_failed_installation_leaves_the_installed_skills_unchanged() {
    let machine = Machine::new();
    machine.write("agents/law-of-demeter/SKILL.md", OLD_SKILL);

    let outcome = execute_failing(
        &["--skills-dir", &machine.path("agents").to_string_lossy()],
        &machine.installation(),
        Some("cargo"),
    );

    assert_eq!(
        (outcome.code, machine.read("agents/law-of-demeter/SKILL.md")),
        (FAILURE, Some(OLD_SKILL.to_owned()))
    );
}

#[test]
fn user_and_project_skill_directories_of_every_agent_are_refreshed_by_default() {
    let machine = Machine::new();
    let installed = [
        "home/.claude/skills/law-of-demeter/SKILL.md",
        "home/.codex/skills/law-of-demeter/SKILL.md",
        "home/.agents/skills/law-of-demeter/SKILL.md",
        "project/.claude/skills/law-of-demeter/SKILL.md",
        "project/.codex/skills/law-of-demeter/SKILL.md",
        "project/.agents/skills/law-of-demeter/SKILL.md",
    ];
    for skill in installed {
        machine.write(skill, OLD_SKILL);
    }

    execute(&[], &machine.installation());

    assert_eq!(
        installed.map(|skill| machine.read(skill)),
        installed.map(|_| Some(NEW_SKILL.to_owned()))
    );
}

#[test]
fn claude_and_codex_homes_replace_their_user_directories() {
    let machine = Machine::new();
    machine.write("claude/skills/law-of-demeter/SKILL.md", OLD_SKILL);
    machine.write("codex/skills/law-of-demeter/SKILL.md", OLD_SKILL);
    machine.write("home/.claude/skills/law-of-demeter/SKILL.md", OLD_SKILL);
    let installation = Installation {
        claude_config_dir: Some(machine.path("claude")),
        codex_home: Some(machine.path("codex")),
        ..machine.installation()
    };

    execute(&[], &installation);

    assert_eq!(
        [
            machine.read("claude/skills/law-of-demeter/SKILL.md"),
            machine.read("codex/skills/law-of-demeter/SKILL.md"),
            machine.read("home/.claude/skills/law-of-demeter/SKILL.md"),
        ],
        [
            Some(NEW_SKILL.to_owned()),
            Some(NEW_SKILL.to_owned()),
            Some(OLD_SKILL.to_owned())
        ]
    );
}
