use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::{Crate, EditionedFileId, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_intern::Symbol;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace_at};
use ra_ap_project_model::{CargoConfig, CfgOverrides, RustLibSource, TargetDirectoryConfig};
use ra_ap_syntax::{AstNode, ast, ast::HasName};
use ra_ap_vfs::{AbsPathBuf, Vfs};

pub(super) type ProbeResult<T = ()> = Result<T, Box<dyn Error>>;

pub(super) struct LoadedFixture {
    pub(super) db: RootDatabase,
    pub(super) vfs: Vfs,
    pub(super) krate: Crate,
    source_path: AbsPathBuf,
}

impl LoadedFixture {
    pub(super) fn source(&self, semantics: &Semantics<'_, RootDatabase>) -> ast::SourceFile {
        let file = EditionedFileId::new(
            &self.db,
            self.krate.root_file(&self.db),
            self.krate.edition(&self.db),
        );
        let errors = file.parse_errors(&self.db);
        assert!(errors.is_none(), "fixture source parse errors: {errors:?}");
        semantics.parse(file)
    }

    pub(super) fn assert_location(
        &self,
        semantics: &Semantics<'_, RootDatabase>,
        node: &ra_ap_syntax::SyntaxNode,
        line: usize,
    ) -> ProbeResult {
        let range = semantics
            .original_range_opt(node)
            .ok_or("call has no original source range")?;
        let source = std::fs::read_to_string(&self.source_path)?;
        let offset = u32::from(range.range.start()) as usize;

        assert_eq!(self.source_path_of(semantics, node)?, self.source_path);
        assert_eq!(
            source[..offset]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1,
            line
        );
        Ok(())
    }

    pub(super) fn source_path_of(
        &self,
        semantics: &Semantics<'_, RootDatabase>,
        node: &ra_ap_syntax::SyntaxNode,
    ) -> ProbeResult<AbsPathBuf> {
        let range = semantics
            .original_range_opt(node)
            .ok_or("definition source range is missing")?;
        self.vfs
            .file_path(range.file_id.file_id(&self.db))
            .as_path()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| "definition has no physical source path".into())
    }
}

pub(super) fn load_fixture_at(
    fixture_dir: &Path,
    source_file: &str,
    set_test: bool,
) -> ProbeResult<LoadedFixture> {
    let source_path = AbsPathBuf::assert_utf8(fixture_dir.join(source_file).canonicalize()?);
    let sysroot = rustc_sysroot()?;
    let source = sysroot.join("lib/rustlib/src/rust/library");
    if !source.is_dir() {
        return Err(format!("matching rust-src is missing: {}", source.display()).into());
    }
    let target = PathBuf::from(std::env::var("CARGO_TARGET_DIR")?);
    if !target.is_absolute() {
        return Err("CARGO_TARGET_DIR must be an absolute probe output path".into());
    }
    let cargo_config = CargoConfig {
        sysroot: Some(RustLibSource::Path(AbsPathBuf::assert_utf8(sysroot))),
        sysroot_src: Some(AbsPathBuf::assert_utf8(source)),
        set_test,
        cfg_overrides: CfgOverrides {
            global: CfgDiff::new(Vec::new(), vec![flag("rust_analyzer")]),
            ..CfgOverrides::default()
        },
        metadata_extra_args: vec!["--locked".into(), "--offline".into()],
        target_dir_config: TargetDirectoryConfig::Directory(
            target
                .join("analyzed")
                .to_str()
                .ok_or("target path is not UTF-8")?
                .into(),
        ),
        ..CargoConfig::default()
    };
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };
    let (db, vfs, proc_client) =
        load_workspace_at(fixture_dir, &cargo_config, &load_config, &|message| {
            eprintln!("{message}")
        })
        .map_err(|error| {
            std::io::Error::other(format!(
                "loading fixture {}: {error:#}",
                fixture_dir.display()
            ))
        })?;
    assert!(proc_client.is_none());
    let krate = crate_for_root(&db, &vfs, &source_path)?;
    Ok(LoadedFixture {
        db,
        vfs,
        krate,
        source_path,
    })
}

pub(super) fn crate_for_root(
    db: &RootDatabase,
    vfs: &Vfs,
    source_path: &AbsPathBuf,
) -> ProbeResult<Crate> {
    let mut crates = Crate::all(db).into_iter().filter(|krate| {
        vfs.file_path(krate.root_file(db)).as_path() == Some(source_path.as_path())
    });
    let krate = crates.next().ok_or("fixture crate was not loaded")?;
    assert!(
        crates.next().is_none(),
        "multiple fixture crate contexts were loaded"
    );
    Ok(krate)
}

fn rustc_sysroot() -> ProbeResult<PathBuf> {
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "rustc sysroot query failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()))
}

pub(super) fn flag(name: &str) -> CfgAtom {
    CfgAtom::Flag(Symbol::intern(name))
}

pub(super) fn declared_function(source: &ast::SourceFile, name: &str) -> ast::Fn {
    source
        .syntax()
        .children()
        .filter_map(ast::Fn::cast)
        .find(|function| {
            function
                .name()
                .is_some_and(|function_name| function_name.text() == name)
        })
        .expect("fixture function declaration is missing")
}

pub(super) fn declared_struct(source: &ast::SourceFile, name: &str) -> ast::Struct {
    source
        .syntax()
        .children()
        .filter_map(ast::Struct::cast)
        .find(|item| {
            item.name()
                .is_some_and(|item_name| item_name.text() == name)
        })
        .expect("fixture struct declaration is missing")
}

pub(super) fn declared_method(source: &ast::SourceFile, owner: &str, name: &str) -> ast::Fn {
    source
        .syntax()
        .children()
        .filter_map(ast::Impl::cast)
        .find(|implementation| {
            implementation
                .self_ty()
                .is_some_and(|ty| ty.syntax().text() == owner)
        })
        .expect("fixture impl declaration is missing")
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|method| {
            method
                .name()
                .is_some_and(|method_name| method_name.text() == name)
        })
        .expect("fixture method declaration is missing")
}

pub(super) fn method_call(caller: &ast::Fn, name: &str) -> ast::MethodCallExpr {
    caller
        .body()
        .expect("fixture caller has no body")
        .syntax()
        .descendants()
        .filter_map(ast::MethodCallExpr::cast)
        .find(|call| {
            call.name_ref()
                .is_some_and(|method_name| method_name.text() == name)
        })
        .expect("fixture method call is missing")
}
