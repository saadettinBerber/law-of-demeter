use std::path::Path;

use ra_ap_hir::{AssocItem, ModuleDef};

use crate::{LoadOptions, RustProject};

#[test]
fn hir_distinguishes_active_default_bodies_required_signatures_and_inactive_items() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project = RustProject::load(
        &root.join("tests/fixtures/trait-defaults/Cargo.toml"),
        LoadOptions::new(
            "trait-defaults",
            &root.join("../../target/rust-friends-journey/analyzed/trait-learning"),
        ),
    )
    .expect("trait source loads");
    let snapshot = &project._snapshot;
    let declaration = snapshot
        .selected
        .root_module(&snapshot.db)
        .declarations(&snapshot.db)
        .into_iter()
        .find_map(|declaration| match declaration {
            ModuleDef::Trait(trait_) => Some(trait_),
            _ => None,
        })
        .expect("trait is active");

    let functions: Vec<_> = declaration
        .items(&snapshot.db)
        .into_iter()
        .filter_map(|item| match item {
            AssocItem::Function(function) => Some((
                function.name(&snapshot.db).as_str().to_owned(),
                function.has_body(&snapshot.db),
            )),
            _ => None,
        })
        .collect();

    assert_eq!(
        functions,
        [("wallet".to_owned(), false), ("pay".to_owned(), true)]
    );
    assert_eq!(
        declaration.module(&snapshot.db).krate(&snapshot.db),
        snapshot.selected
    );
}
