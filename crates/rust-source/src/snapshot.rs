use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::{Adt, Crate, ModuleDef, Semantics, Type};
use ra_ap_ide_db::RootDatabase;
use ra_ap_intern::Symbol;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace};
use ra_ap_proc_macro_api::ProcMacroClient;
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, CfgOverrides, ProjectManifest, ProjectWorkspace,
    ProjectWorkspaceKind, RustLibSource, TargetDirectoryConfig, TargetKind,
};
use ra_ap_syntax::{AstNode, SyntaxNode};
use ra_ap_vfs::{AbsPathBuf, Vfs};

use law_of_demeter_core::domain::type_shape::TypeRef;

type LoadResult<T> = Result<T, Box<dyn Error>>;

pub struct LoadOptions {
    package: String,
    output: PathBuf,
    features: Vec<String>,
    no_default_features: bool,
    target: Option<String>,
}

impl LoadOptions {
    pub fn new(package: &str, output: &Path) -> Self {
        Self {
            package: package.to_owned(),
            output: output.to_path_buf(),
            features: Vec::new(),
            no_default_features: false,
            target: None,
        }
    }

    pub fn with_features(mut self, features: Vec<String>, no_default_features: bool) -> Self {
        self.features = features;
        self.no_default_features = no_default_features;
        self
    }

    pub fn for_target(mut self, target: &str) -> Self {
        self.target = Some(target.to_owned());
        self
    }
}

pub(super) struct Snapshot {
    pub(super) db: RootDatabase,
    pub(super) vfs: Vfs,
    pub(super) selected: Crate,
    pub(super) project_crates: Vec<Crate>,
    identities: HashMap<Crate, String>,
    pub(super) source_root: PathBuf,
    pub(super) _proc_client: ProcMacroClient,
}

impl Snapshot {
    pub(super) fn load(manifest: &Path, options: LoadOptions) -> LoadResult<Self> {
        let manifest = absolute(manifest.canonicalize()?)?;
        let (config, server) = configuration(&options)?;
        let manifest = ProjectManifest::from_manifest_file(manifest)?;
        let mut workspace = ProjectWorkspace::load(manifest, &config, &|_| {})?;
        let ProjectWorkspaceKind::Cargo { cargo, error, .. } = &workspace.kind else {
            return Err("a Cargo workspace is required".into());
        };
        if let Some(error) = error {
            return Err(format!("partial Cargo metadata: {error:#}").into());
        }
        if let Some(error) = workspace.sysroot.error() {
            return Err(format!("loading sysroot: {error}").into());
        }
        let mut selected_roots = Vec::new();
        let mut roots = Vec::new();
        for package in cargo.packages() {
            for target in &cargo[package].targets {
                let target = &cargo[*target];
                if target.kind
                    != (TargetKind::Lib {
                        is_proc_macro: false,
                    })
                {
                    continue;
                }
                if cargo[package].is_member && cargo[package].name == options.package {
                    selected_roots.push(target.root.clone());
                }
                roots.push((
                    target.root.clone(),
                    format!("{}::{}", cargo[package].id, target.name),
                    cargo[package].is_member,
                ));
            }
        }
        if selected_roots.len() != 1 {
            return Err(format!(
                "package {} must have one normal library target",
                options.package
            )
            .into());
        }
        let build = workspace.run_build_scripts(&config, &|_| {})?;
        if let Some(error) = build.error() {
            return Err(format!("Cargo build failed: {error}").into());
        }
        workspace.set_build_scripts(build);
        let load_config = LoadCargoConfig {
            load_out_dirs_from_check: false,
            with_proc_macro_server: ProcMacroServerChoice::Explicit(server),
            prefill_caches: false,
            num_worker_threads: 1,
            proc_macro_processes: 1,
        };
        let (db, vfs, proc_client) = load_workspace(workspace, &config.extra_env, &load_config)?;
        let proc_client = proc_client.ok_or("loader did not retain the proc macro client")?;
        let mut identities = HashMap::new();
        let mut project_crates = Vec::new();
        let mut selected = Vec::new();
        for krate in Crate::all(&db) {
            let path = vfs.file_path(krate.root_file(&db));
            if path.as_path() == Some(selected_roots[0].as_path()) {
                selected.push(krate);
            }
            if let Some((_, identity, member)) = roots
                .iter()
                .find(|(root, _, _)| path.as_path() == Some(root.as_path()))
            {
                identities.insert(krate, identity.clone());
                if *member {
                    project_crates.push(krate);
                }
            }
        }
        if selected.len() != 1 {
            return Err("selected library has missing or ambiguous crate contexts".into());
        }
        Ok(Self {
            db,
            vfs,
            selected: selected[0],
            project_crates,
            identities,
            source_root: PathBuf::from(selected_roots[0].as_str()),
            _proc_client: proc_client,
        })
    }

    pub(super) fn identity(&self, adt: Adt) -> String {
        self.definition_identity(ModuleDef::Adt(adt))
            .unwrap_or_else(|| adt.name(&self.db).as_str().to_owned())
    }

    pub(super) fn definition_identity(&self, definition: ModuleDef) -> Option<String> {
        let krate = definition.module(&self.db)?.krate(&self.db);
        let path = definition
            .canonical_path(&self.db, krate.edition(&self.db))
            .or_else(|| {
                definition
                    .name(&self.db)
                    .map(|name| name.as_str().to_owned())
            })?;
        Some(match self.identities.get(&krate) {
            Some(prefix) => format!("{prefix}::{path}"),
            None => format!(
                "{}::{path}",
                krate
                    .display_name(&self.db)
                    .map(|name| name.to_string())
                    .unwrap_or_default()
            ),
        })
    }

    pub(super) fn type_ref(&self, ty: &Type<'_>) -> Result<TypeRef, String> {
        if ty.contains_unknown() {
            return Err("type contains unknown components".into());
        }
        if let Some(inner) = ty.as_reference_inner() {
            return self.type_ref(&inner);
        }
        if ty.is_unit() {
            return Ok(TypeRef::primitive("()"));
        }
        if let Some(builtin) = ty.as_builtin() {
            return Ok(TypeRef::primitive(builtin.name().as_str()));
        }
        if let Some(adt) = ty.as_adt() {
            if !matches!(adt, Adt::Struct(_)) {
                return Err("aggregate kind is outside the first source slice".into());
            }
            let arguments = ty
                .type_arguments()
                .map(|argument| self.type_ref(&argument))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(TypeRef::generic(&self.identity(adt), arguments));
        }
        Err("type is outside the first source slice".into())
    }

    pub(super) fn location(
        &self,
        semantics: &Semantics<'_, RootDatabase>,
        node: &SyntaxNode,
    ) -> Result<(PathBuf, usize), String> {
        let range = semantics
            .original_range_opt(node)
            .ok_or("source range is unavailable")?;
        let file = range.file_id.file_id(&self.db);
        let path = self
            .vfs
            .file_path(file)
            .as_path()
            .ok_or("source has no physical path")?
            .as_str()
            .into();
        let source = semantics.parse(range.file_id);
        let offset = u32::from(range.range.start()) as usize;
        let text = source.syntax().text().to_string();
        let prefix = text.get(..offset).ok_or("source range is invalid")?;
        Ok((
            path,
            prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
        ))
    }
}

fn configuration(options: &LoadOptions) -> LoadResult<(CargoConfig, AbsPathBuf)> {
    if !options.output.is_absolute() {
        return Err("analysis output directory must be absolute".into());
    }
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "querying rustc sysroot: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let sysroot = PathBuf::from(String::from_utf8(output.stdout)?.trim());
    let source = sysroot.join("lib/rustlib/src/rust/library");
    let server = sysroot.join("libexec/rust-analyzer-proc-macro-srv");
    if !source.is_dir() || !server.is_file() {
        return Err("matching rust-src and proc macro server are required".into());
    }
    let target_directory = options
        .output
        .to_str()
        .ok_or("output directory is not UTF-8")?;
    let mut config = CargoConfig {
        sysroot: Some(RustLibSource::Path(absolute(sysroot)?)),
        sysroot_src: Some(absolute(source)?),
        set_test: false,
        features: CargoFeatures::Selected {
            features: options.features.clone(),
            no_default_features: options.no_default_features,
        },
        target: options.target.clone(),
        cfg_overrides: CfgOverrides {
            global: CfgDiff::new(
                Vec::new(),
                vec![CfgAtom::Flag(Symbol::intern("rust_analyzer"))],
            ),
            ..CfgOverrides::default()
        },
        metadata_extra_args: vec!["--locked".into(), "--offline".into()],
        extra_args: vec![
            "--locked".into(),
            "--offline".into(),
            "--package".into(),
            options.package.clone(),
            "--lib".into(),
        ],
        target_dir_config: TargetDirectoryConfig::Directory(target_directory.into()),
        ..CargoConfig::default()
    };
    config
        .extra_env
        .insert("CARGO_TARGET_DIR".into(), Some(target_directory.into()));
    Ok((config, absolute(server)?))
}

fn absolute(path: PathBuf) -> LoadResult<AbsPathBuf> {
    AbsPathBuf::try_from(path.to_str().ok_or("path is not UTF-8")?)
        .map_err(|path| format!("path must be absolute: {path}").into())
}
