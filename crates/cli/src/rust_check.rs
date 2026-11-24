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
    let names = Names::new(&project, &metadata)?;
    let targets = arguments
        .targets
        .iter()
        .map(|target| names.resolve(target))
        .collect::<Result<Vec<_>, _>>()?;
    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(targets))
        .execute();
    let unmatched = report
        .unmatched_targets()
        .iter()
        .map(|target| {
            let class = names.display(target.class_name());
            match target.method_name() {
                Some(method) => format!("{class}#{method}"),
                None => class,
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut uncertainty = None;
    if !report.unmatched_targets().is_empty() && !incomplete(&report) {
        let all = CheckProject::new(&project, &project).execute();
        if !incomplete(&all) {
            return Err(format!("nothing matches --only: {}", unmatched));
        }
        uncertainty = Some(all);
    }
    if !report.unmatched_targets().is_empty() {
        writeln!(
            err,
            "Selection could not be verified because project analysis is incomplete: {}",
            unmatched
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

struct Names {
    types: Vec<(String, String)>,
    methods: Vec<Target>,
}

impl Names {
    fn new(project: &RustProject, metadata: &Value) -> Result<Self, String> {
        let packages = metadata["packages"]
            .as_array()
            .ok_or("Cargo metadata has no packages")?;
        let types = project
            .project_types()
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
        let mut methods = Vec::new();
        for shape in TypeSource::read(project).shapes {
            methods.extend(
                shape
                    .methods()
                    .iter()
                    .map(|method| Target::method(shape.name(), method.name())),
            );
        }
        Ok(Self { types, methods })
    }

    fn display(&self, identity: &str) -> String {
        let Some((_, readable)) = self.types.iter().find(|(name, _)| name == identity) else {
            return identity.to_owned();
        };
        if self
            .types
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
        let (class, method) = if !self.matching(text).is_empty() {
            (text, None)
        } else if let Some((separator, _)) = text
            .rmatch_indices('#')
            .find(|(index, _)| !self.matching(&text[..*index]).is_empty())
        {
            (&text[..separator], Some(&text[separator + 1..]))
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
        let matches = self.matching(class);
        if matches.len() > 1 {
            return Err(format!(
                "ambiguous --only: {text}; use one of: {}",
                matches
                    .iter()
                    .map(|identity| self.display(identity))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let identity = matches.first().copied().unwrap_or(class);
        Ok(match method {
            Some(method) => {
                let method = method
                    .strip_prefix("r#")
                    .filter(|name| {
                        self.methods.iter().any(|target| {
                            target.class_name() == identity && target.method_name() == Some(name)
                        })
                    })
                    .unwrap_or(method);
                Target::method(identity, method)
            }
            None => Target::class(identity),
        })
    }

    fn matching(&self, name: &str) -> Vec<&str> {
        let suffix = format!("::{name}");
        self.types
            .iter()
            .filter(|(identity, readable)| {
                identity == name || readable == name || identity.ends_with(&suffix)
            })
            .map(|(identity, _)| identity.as_str())
            .collect()
    }
}
