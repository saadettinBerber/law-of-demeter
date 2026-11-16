use std::fs;
use std::path::Path;

const WORKSPACE_PREFIX: &str = "law-of-demeter-";
const CORE: &str = "core";
const ADAPTERS: [&str; 2] = ["class-file", "java-source"];

#[test]
fn core_depends_on_no_other_crate() {
    let dependencies = dependencies_of(CORE);

    assert!(
        dependencies.is_empty(),
        "{CORE} must have no dependencies but depends on {dependencies:?}"
    );
}

#[test]
fn adapters_depend_on_core_and_on_no_other_adapter() {
    let violations: Vec<String> = ADAPTERS
        .iter()
        .flat_map(|adapter| {
            let workspace = workspace_dependencies_of(adapter);
            let missing = (!workspace.contains(&CORE.to_string()))
                .then(|| format!("{adapter} does not depend on {CORE}"));
            let foreign = workspace
                .into_iter()
                .filter(|dependency| dependency != CORE)
                .map(|dependency| format!("{adapter} depends on {dependency}"));
            missing.into_iter().chain(foreign).collect::<Vec<_>>()
        })
        .collect();

    assert!(
        violations.is_empty(),
        "adapters break the dependency rule:\n{}",
        violations.join("\n")
    );
}

#[test]
fn only_cli_depends_on_the_adapters() {
    let violations: Vec<String> = [CORE, ADAPTERS[0], ADAPTERS[1]]
        .iter()
        .flat_map(|name| {
            workspace_dependencies_of(name)
                .into_iter()
                .filter(|dependency| ADAPTERS.contains(&dependency.as_str()))
                .map(|dependency| format!("{name} depends on the adapter {dependency}"))
                .collect::<Vec<_>>()
        })
        .collect();

    assert!(
        violations.is_empty(),
        "only cli may depend on the adapters:\n{}",
        violations.join("\n")
    );
}

fn workspace_dependencies_of(crate_dir: &str) -> Vec<String> {
    dependencies_of(crate_dir)
        .into_iter()
        .filter_map(|name| name.strip_prefix(WORKSPACE_PREFIX).map(str::to_string))
        .collect()
}

fn dependencies_of(crate_dir: &str) -> Vec<String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(crate_dir)
        .join("Cargo.toml");
    let text = fs::read_to_string(&manifest)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", manifest.display()));

    text.lines()
        .map(str::trim)
        .skip_while(|line| *line != "[dependencies]")
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split(['=', '.', ' ']).next())
        .map(str::to_string)
        .collect()
}
