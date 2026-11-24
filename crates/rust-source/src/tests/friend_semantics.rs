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

#[test]
fn ra_distinguishes_a_tuple_constructor_alias_from_a_function_and_resolves_local_bindings() {
    use ra_ap_hir::{CallableKind, PathResolution};

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project = RustProject::load(
        &root.join("tests/fixtures/friends/Cargo.toml"),
        LoadOptions::new(
            "friend-origins",
            &root.join("../../target/rust-friends-journey/analyzed/creation-learning"),
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
    let pocket = source
        .syntax()
        .descendants()
        .filter_map(ast::Struct::cast)
        .find(|structure| structure.name().is_some_and(|name| name.text() == "Pocket"))
        .and_then(|structure| semantics.to_def(&structure))
        .expect("Pocket declaration resolves");
    let tuple_caller = function_in(&source, "create_tuple");
    let tuple = tuple_caller
        .syntax()
        .descendants()
        .find_map(ast::CallExpr::cast)
        .expect("tuple creation exists");
    let function_caller = function_in(&source, "from_function");
    let function_call = function_caller
        .syntax()
        .descendants()
        .find_map(ast::CallExpr::cast)
        .expect("function call exists");
    let constructor_function = semantics
        .to_def(&function_in(&source, "new"))
        .expect("associated function resolves");
    let named_caller = function_in(&source, "create_named");
    let binding = named_caller
        .body()
        .expect("named body exists")
        .syntax()
        .descendants()
        .find_map(ast::IdentPat::cast)
        .expect("local binding exists");
    let usage = named_caller
        .body()
        .expect("body exists")
        .syntax()
        .descendants()
        .filter_map(ast::MethodCallExpr::cast)
        .find(|call| {
            call.name_ref()
                .is_some_and(|name| name.text() == "withdraw")
        })
        .and_then(|call| call.receiver())
        .expect("local receiver exists");
    let holder_caller = function_in(&source, "new_holder");
    let shorthand = holder_caller
        .body()
        .expect("holder body exists")
        .syntax()
        .descendants()
        .find_map(ast::RecordExprField::cast)
        .expect("shorthand initializer exists");

    let tuple_target = semantics
        .resolve_expr_as_callable(&tuple.expr().expect("tuple callee exists"))
        .expect("tuple callee resolves");
    let function_target = semantics
        .resolve_expr_as_callable(&function_call.expr().expect("function callee exists"))
        .expect("function callee resolves");
    let local = semantics
        .to_def(&binding)
        .expect("local has a HIR identity");
    let ast::Expr::PathExpr(usage) = usage else {
        panic!("local receiver is a path")
    };
    let resolved_usage = semantics.resolve_path(&usage.path().expect("local path exists"));
    let (_, captured, _) = semantics
        .resolve_record_field(&shorthand)
        .expect("shorthand resolves");
    let holder_definition = semantics
        .to_def(&holder_caller)
        .expect("holder caller resolves");
    let holder_params = holder_definition.params_without_self(&snapshot.db);

    assert!(
        matches!(tuple_target.kind(), CallableKind::TupleStruct(structure) if structure == pocket)
    );
    assert!(matches!(function_target.kind(), CallableKind::Function(_)));
    assert_eq!(function_target.as_function(), Some(constructor_function));
    assert_eq!(resolved_usage, Some(PathResolution::Local(local)));
    assert_eq!(captured, holder_params[0].as_local(&snapshot.db));
}

#[test]
fn ra_distinguishes_reference_dereference_from_custom_projection_and_coercion() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project = RustProject::load(
        &root.join("tests/fixtures/friends/Cargo.toml"),
        LoadOptions::new(
            "friend-origins",
            &root.join("../../target/rust-friends-journey/analyzed/reference-learning"),
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
    let normal = function_in(&source, "parameter_reborrow")
        .body()
        .expect("normal body exists")
        .syntax()
        .descendants()
        .find_map(ast::PrefixExpr::cast)
        .expect("reference dereference exists");
    let custom = function_in(&source, "custom_projection")
        .body()
        .expect("custom body exists")
        .syntax()
        .descendants()
        .find_map(ast::PrefixExpr::cast)
        .expect("custom dereference exists");
    let coercion = function_in(&source, "coerced_projection")
        .body()
        .expect("coercion body exists")
        .syntax()
        .descendants()
        .find_map(ast::LetStmt::cast)
        .and_then(|statement| statement.initializer())
        .expect("coercion initializer exists");

    let normal_operand = semantics
        .type_of_expr(&normal.expr().expect("normal operand exists"))
        .expect("normal operand resolves");
    let custom_operand = semantics
        .type_of_expr(&custom.expr().expect("custom operand exists"))
        .expect("custom operand resolves");
    let coercion_type = semantics
        .type_of_expr(&coercion)
        .expect("coercion resolves");

    assert_eq!(normal.op_kind(), Some(ast::UnaryOp::Deref));
    assert!(normal_operand.original.as_reference_inner().is_some());
    assert_eq!(custom.op_kind(), Some(ast::UnaryOp::Deref));
    assert!(custom_operand.original.as_reference_inner().is_none());
    assert_eq!(
        custom_operand
            .original
            .as_adt()
            .expect("wrapper is an ADT")
            .name(&snapshot.db)
            .as_str(),
        "Wrapper"
    );
    assert_eq!(
        coercion_type
            .original
            .as_reference_inner()
            .and_then(|ty| ty.as_adt())
            .expect("original wrapper resolves")
            .name(&snapshot.db)
            .as_str(),
        "Wrapper"
    );
    assert_eq!(
        coercion_type
            .adjusted
            .and_then(|ty| ty.as_reference_inner())
            .and_then(|ty| ty.as_adt())
            .expect("adjusted wallet resolves")
            .name(&snapshot.db)
            .as_str(),
        "Wallet"
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

fn function_in(source: &ast::SourceFile, name: &str) -> ast::Fn {
    source
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|function| {
            function
                .name()
                .is_some_and(|declared| declared.text() == name)
        })
        .expect("caller exists")
}
