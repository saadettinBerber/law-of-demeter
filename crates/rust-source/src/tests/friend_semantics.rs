use std::path::Path;

use ra_ap_hir::{Adt, Semantics, Variant};
use ra_ap_syntax::{AstNode, ast, ast::HasName};

use crate::{LoadOptions, RustProject};

#[test]
fn ra_resolves_the_same_private_field_on_self_and_on_another_instance() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project = RustProject::load(
        &root.join("tests/fixtures/friends/Cargo.toml"),
        LoadOptions::new(
            "friend-origins",
            &root.join("../../target/rust-friends-journey/analyzed/field-learning"),
        ),
    )
    .expect("friend source loads");
    let snapshot = &project._snapshot;
    let semantics = Semantics::new(&snapshot.db);
    let source = semantics.parse(ra_ap_hir::EditionedFileId::new(
        &snapshot.db,
        snapshot.selected.root_file(&snapshot.db),
        snapshot.selected.edition(&snapshot.db),
    ));
    let service = source
        .syntax()
        .descendants()
        .filter_map(ast::Struct::cast)
        .find(|structure| {
            structure
                .name()
                .is_some_and(|name| name.text() == "PaymentService")
        })
        .and_then(|structure| semantics.to_def(&structure))
        .expect("service declaration resolves");
    let declared = service
        .fields(&snapshot.db)
        .into_iter()
        .find(|field| field.name(&snapshot.db).as_str() == "wallet")
        .expect("private field is declared");
    let owned = field_in(&source, "on_held_field");
    let other = field_in(&source, "on_peer_field");

    let owned_target = semantics
        .resolve_field(&owned)
        .and_then(|field| field.left())
        .expect("self field resolves");
    let other_target = semantics
        .resolve_field(&other)
        .and_then(|field| field.left())
        .expect("peer field resolves");
    let receiver = semantics
        .type_of_expr(&other.expr().expect("peer expression exists"))
        .expect("peer type resolves");

    assert_eq!(owned_target, declared);
    assert_eq!(other_target, declared);
    assert_eq!(declared.parent_def(&snapshot.db), Variant::Struct(service));
    assert_eq!(
        receiver
            .original
            .as_reference_inner()
            .expect("peer is borrowed")
            .as_adt(),
        Some(Adt::Struct(service))
    );
}

fn field_in(source: &ast::SourceFile, method: &str) -> ast::FieldExpr {
    source
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|function| function.name().is_some_and(|name| name.text() == method))
        .expect("caller exists")
        .syntax()
        .descendants()
        .find_map(ast::FieldExpr::cast)
        .expect("caller has a field")
}
