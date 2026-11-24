use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use law_of_demeter_core::application::{CheckProject, Selection, Target, TypeSource};
use law_of_demeter_rust_source::{LoadOptions, RustProject};
use serde_json::Value;

use crate::arguments::RustArguments;
use crate::report_text::{incomplete, render_rust};
use crate::run::Environment;

pub(crate) fn check(
    arguments: RustArguments,
    show_allowed: bool,
    environment: &Environment,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<(u8, String), String> {
    let manifest = absolute(&environment.working_directory, &arguments.manifest)?;
    let metadata = metadata(&manifest)?;
    let workspace = metadata["workspace_root"]
        .as_str()
        .ok_or("Cargo metadata has no workspace root")?;
    let output = match &arguments.target_dir {
        Some(path) => absolute(&environment.working_directory, path)?,
        None => Path::new(workspace).join("target/law-of-demeter"),
    };
    std::fs::create_dir_all(&output)
        .map_err(|error| format!("creating analysis output {}: {error}", output.display()))?;
    let mut options = LoadOptions::new(&arguments.package, &output)
        .with_features(arguments.features, arguments.no_default_features);
    if let Some(target) = &arguments.target {
        options = options.for_target(target);
    }
    writeln!(
        err,
        "Rust manifest: {}\nPackage: {}\nAnalysis output: {}\n",
        manifest.display(),
        arguments.package,
        output.display()
    )
    .ok();
    let project = RustProject::load(&manifest, options)
        .map_err(|error| crate::report_text::error_line(&error))?;
    let names = Names::new(project.project_types(), &metadata)?;
    let targets = arguments
        .targets
        .iter()
        .map(|target| names.resolve(target))
        .collect::<Result<Vec<_>, _>>()?;
    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(targets))
        .execute();
    let mut uncertainty = None;
    if !report.unmatched_targets().is_empty() && !incomplete(&report) {
        let all = CheckProject::new(&project, &project).execute();
        if !incomplete(&all) {
            return Err(format!(
                "nothing matches --only: {}",
                arguments.targets.join(", ")
            ));
        }
        uncertainty = Some(all);
    }
    if !report.unmatched_targets().is_empty() {
        writeln!(
            err,
            "Selection could not be verified because project analysis is incomplete: {}",
            arguments.targets.join(", ")
        )
        .ok();
    }
    write!(
        out,
        "{}",
        render_rust(
            &report,
            show_allowed,
            &|name| names.display(name),
            uncertainty.as_ref()
        )
    )
    .ok();
    let counts = report.counts();
    if incomplete(&report) || uncertainty.is_some() {
        Ok((
            3,
            format!(
                "incomplete analysis; {} violations, {} hybrid classes",
                counts.violations, counts.hybrid_classes
            ),
        ))
    } else if counts.violations + counts.hybrid_classes > 0 {
        Ok((
            1,
            format!(
                "{} violations, {} hybrid classes",
                counts.violations, counts.hybrid_classes
            ),
        ))
    } else {
        Ok((0, "no violations".into()))
    }
}

fn absolute(directory: &Path, path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let directory = std::env::current_dir()
        .map_err(|error| format!("reading working directory: {error}"))?
        .join(directory);
    Ok(directory.join(path))
}

fn metadata(manifest: &Path) -> Result<Value, String> {
    let mut command = Command::new("cargo");
    if let Some(directory) = manifest.parent() {
        command.current_dir(directory);
    }
    let output = command
        .args([
            "metadata",
            "--format-version=1",
            "--no-deps",
            "--locked",
            "--offline",
            "--manifest-path",
        ])
        .arg(manifest)
        .output()
        .map_err(|error| format!("starting Cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Cargo metadata for {}: {}",
            manifest.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("reading Cargo metadata: {error}"))
}

struct Names(Vec<(String, String)>);

impl Names {
    fn new(catalog: Vec<String>, metadata: &Value) -> Result<Self, String> {
        let packages = metadata["packages"]
            .as_array()
            .ok_or("Cargo metadata has no packages")?;
        let names = catalog
            .into_iter()
            .map(|identity| {
                let readable = packages
                    .iter()
                    .find_map(|package| {
                        let id = package["id"].as_str()?;
                        let name = package["name"].as_str()?;
                        identity
                            .strip_prefix(&format!("{id}::"))
                            .map(|suffix| format!("{name}/{suffix}"))
                    })
                    .unwrap_or_else(|| identity.clone());
                (identity, readable)
            })
            .collect::<Vec<_>>();
        Ok(Self(names))
    }

    fn display(&self, identity: &str) -> String {
        let Some((_, readable)) = self.0.iter().find(|(name, _)| name == identity) else {
            return identity.to_owned();
        };
        if self
            .0
            .iter()
            .filter(|(_, candidate)| candidate == readable)
            .count()
            == 1
        {
            readable.clone()
        } else {
            identity.to_owned()
        }
    }

    fn resolve(&self, text: &str) -> Result<Target, String> {
        let (class, method) = if self.0.iter().any(|(identity, _)| identity == text) {
            (text, None)
        } else {
            match text.rsplit_once('#') {
                Some((class, method)) if !method.contains("::") => (class, Some(method)),
                _ => (text, None),
            }
        };
        if class.is_empty() || method == Some("") {
            return Err(format!(
                "--only needs a type, optionally followed by #method: {text}"
            ));
        }
        let matches: Vec<_> = self
            .0
            .iter()
            .filter(|(identity, readable)| {
                identity == class || readable == class || identity.ends_with(&format!("::{class}"))
            })
            .collect();
        if matches.len() > 1 {
            return Err(format!(
                "ambiguous --only: {text}; use one of: {}",
                matches
                    .iter()
                    .map(|(identity, _)| self.display(identity))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let identity = matches.first().map_or(class, |(identity, _)| identity);
        Ok(match method {
            Some(method) => Target::method(identity, method),
            None => Target::class(identity),
        })
    }
}
