use std::path::Path;

use ra_ap_hir::{ModuleDef, Semantics};
use ra_ap_syntax::AstNode;

use crate::{LoadOptions, RustProject};

#[test]
fn hir_keeps_the_generated_function_body_and_source_when_its_physical_range_is_unavailable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project = RustProject::load(
        &root.join("tests/fixtures/free-function-coverage/Cargo.toml"),
        LoadOptions::new(
            "free-function-coverage",
            &root.join("../../target/rust-friends-journey/analyzed/free-function-learning"),
        ),
    )
    .expect("generated free function source loads");
    let snapshot = &project._snapshot;
    let semantics = Semantics::new(&snapshot.db);
    let functions: Vec<_> = snapshot
        .selected
        .root_module(&snapshot.db)
        .declarations(&snapshot.db)
        .into_iter()
        .filter_map(|declaration| match declaration {
            ModuleDef::Function(function) => Some(function),
            _ => None,
        })
        .collect();

    let states: Vec<_> = functions
        .into_iter()
        .map(|function| {
            let source = semantics.source(function);
            let location = source
                .as_ref()
                .map(|source| snapshot.location(&semantics, source.value.syntax()));
            (
                function.name(&snapshot.db).as_str().to_owned(),
                function.has_body(&snapshot.db),
                source.is_some(),
                location,
            )
        })
        .collect();

    assert_eq!(
        states,
        [(
            "pay".to_owned(),
            true,
            true,
            Some(Err("source range is unavailable".to_owned()))
        )],
    );
}
