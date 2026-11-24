mod free_function_semantics;
mod trait_semantics;

use std::path::Path;

use law_of_demeter_core::application::TypeSource;
use law_of_demeter_core::domain::type_shape::TypeRef;

use ra_ap_hir::{Adt, PathResolution, Semantics};
use ra_ap_syntax::{AstNode, ast, ast::HasName};

use crate::{LoadOptions, RustProject};

#[test]
fn ra_resolves_parameter_binding_and_both_chain_targets_in_the_port_snapshot() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = RustProject::load(
        &root.join("examples/rust-scenarios/Cargo.toml"),
        LoadOptions::new(
            "rust-scenario-core",
            &root.join("target/rust-source-journey/analyzed/snapshot"),
        ),
    )
    .expect("Cargo source loads");
    let snapshot = &project._snapshot;
    let semantics = Semantics::new(&snapshot.db);
    let source = semantics.parse(ra_ap_hir::EditionedFileId::new(
        &snapshot.db,
        snapshot.selected.root_file(&snapshot.db),
        snapshot.selected.edition(&snapshot.db),
    ));
    let structures: Vec<_> = source
        .syntax()
        .descendants()
        .filter_map(ast::Struct::cast)
        .collect();
    let wallet = semantics
        .to_def(
            &structures
                .iter()
                .find(|structure| structure.name().is_some_and(|name| name.text() == "Wallet"))
                .expect("Wallet is declared")
                .clone(),
        )
        .expect("Wallet resolves");
    let methods: Vec<_> = source
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .collect();
    let parameter = methods
        .iter()
        .find(|method| {
            method
                .name()
                .is_some_and(|name| name.text() == "on_parameter")
        })
        .expect("parameter body exists");
    let chain = methods
        .iter()
        .find(|method| {
            method
                .name()
                .is_some_and(|name| name.text() == "on_returned_chain")
        })
        .expect("chain body exists");
    let direct = parameter
        .syntax()
        .descendants()
        .find_map(ast::MethodCallExpr::cast)
        .expect("parameter call exists");
    let debit = chain
        .syntax()
        .descendants()
        .find_map(ast::MethodCallExpr::cast)
        .expect("chain call exists");

    let wallet_impl = source
        .syntax()
        .descendants()
        .filter_map(ast::Impl::cast)
        .find(|implementation| {
            implementation
                .self_ty()
                .is_some_and(|ty| ty.syntax().text() == "Wallet")
        })
        .expect("Wallet impl exists");
    let wallet_debit = wallet_impl
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|function| function.name().is_some_and(|name| name.text() == "debit"))
        .expect("Wallet debit declaration exists");
    let order_impl = source
        .syntax()
        .descendants()
        .filter_map(ast::Impl::cast)
        .find(|implementation| {
            implementation
                .self_ty()
                .is_some_and(|ty| ty.syntax().text() == "Order")
        })
        .expect("Order impl exists");
    let order_wallet = order_impl
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|function| function.name().is_some_and(|name| name.text() == "wallet"))
        .expect("Order wallet declaration exists");

    let direct_target = semantics
        .resolve_method_call(&direct)
        .expect("direct target resolves");
    let chain_target = semantics
        .resolve_method_call(&debit)
        .expect("chain target resolves");
    let receiver = semantics
        .type_of_expr(&direct.receiver().expect("direct receiver exists"))
        .expect("receiver type resolves");
    let ast::Expr::MethodCallExpr(projection) = debit.receiver().expect("chain receiver exists")
    else {
        panic!("chain receiver must be a method result")
    };
    let projection_target = semantics
        .resolve_method_call(&projection)
        .expect("projection target resolves");
    let result = semantics
        .type_of_expr(&ast::Expr::MethodCallExpr(projection))
        .expect("projection result resolves");

    let ast::Expr::PathExpr(receiver_path) = direct.receiver().expect("direct receiver exists")
    else {
        panic!("parameter receiver must be a path")
    };
    let parameter_definition = semantics
        .to_def(parameter)
        .expect("parameter method resolves");
    let parameters = parameter_definition.params_without_self(&snapshot.db);
    let binding = parameters[0]
        .as_local(&snapshot.db)
        .expect("parameter has a local identity");

    let reading = TypeSource::read(&project);
    let wallet_name = project
        .project_types()
        .into_iter()
        .find(|name| name.ends_with("::Wallet"))
        .expect("Wallet remains a project type");
    let order_shape = reading
        .shapes
        .iter()
        .find(|shape| shape.name().ends_with("::Order"))
        .expect("Order shape is read");
    let projection_shape = order_shape
        .methods()
        .iter()
        .find(|method| method.name() == "wallet")
        .expect("projection signature is read");
    let service_shape = reading
        .shapes
        .iter()
        .find(|shape| shape.name().ends_with("::PaymentService"))
        .expect("Service shape is read");
    let parameter_shape = service_shape
        .methods()
        .iter()
        .find(|method| method.name() == "on_parameter")
        .expect("parameter signature is read");

    assert_eq!(
        semantics.resolve_path(&receiver_path.path().expect("receiver has a path")),
        Some(PathResolution::Local(binding))
    );
    assert_eq!(
        direct_target,
        semantics
            .to_def(&wallet_debit)
            .expect("Wallet debit resolves")
    );
    assert_eq!(
        projection_target,
        semantics
            .to_def(&order_wallet)
            .expect("Order wallet resolves")
    );
    assert_eq!(direct_target, chain_target);
    assert_eq!(direct_target.name(&snapshot.db).as_str(), "debit");
    assert_eq!(projection_target.name(&snapshot.db).as_str(), "wallet");
    assert_eq!(
        receiver
            .original
            .as_reference_inner()
            .expect("parameter is borrowed")
            .as_adt(),
        Some(Adt::Struct(wallet))
    );
    assert_eq!(
        result
            .original
            .as_reference_inner()
            .expect("result is borrowed")
            .as_adt(),
        Some(Adt::Struct(wallet))
    );
    assert!(!result.original.contains_unknown());
    assert!(!receiver.original.contains_unknown());
    assert_eq!(
        projection_shape.returns(),
        Some(&TypeRef::class(&wallet_name))
    );
    assert_eq!(parameter_shape.parameters(), [TypeRef::class(&wallet_name)]);
    assert!(
        !snapshot
            .selected
            .cfg(&snapshot.db)
            .check_atom(&ra_ap_cfg::CfgAtom::Flag(ra_ap_intern::Symbol::intern(
                "test"
            )))
    );
    assert!(
        !snapshot
            .selected
            .cfg(&snapshot.db)
            .check_atom(&ra_ap_cfg::CfgAtom::Flag(ra_ap_intern::Symbol::intern(
                "rust_analyzer"
            )))
    );
    assert!(
        snapshot
            ._proc_client
            .server_path()
            .as_str()
            .ends_with("/libexec/rust-analyzer-proc-macro-srv")
    );
}

#[test]
fn ra_exposes_trait_ownership_and_the_actual_deref_receiver_separately_from_references() {
    use ra_ap_hir::AsAssocItem;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project = RustProject::load(
        &root.join("tests/fixtures/dispatch/Cargo.toml"),
        LoadOptions::new(
            "dispatch-boundary",
            &root.join("../../target/rust-source-journey/analyzed/dispatch-learning"),
        ),
    )
    .expect("Cargo source loads");
    let snapshot = &project._snapshot;
    let semantics = Semantics::new(&snapshot.db);
    let source = semantics.parse(ra_ap_hir::EditionedFileId::new(
        &snapshot.db,
        snapshot.selected.root_file(&snapshot.db),
        snapshot.selected.edition(&snapshot.db),
    ));
    let account = source
        .syntax()
        .descendants()
        .filter_map(ast::Struct::cast)
        .find(|structure| {
            structure
                .name()
                .is_some_and(|name| name.text() == "Account")
        })
        .and_then(|structure| semantics.to_def(&structure))
        .expect("Account declaration resolves");
    let wrapper = source
        .syntax()
        .descendants()
        .filter_map(ast::Struct::cast)
        .find(|structure| {
            structure
                .name()
                .is_some_and(|name| name.text() == "Wrapper")
        })
        .and_then(|structure| semantics.to_def(&structure))
        .expect("Wrapper declaration resolves");
    let trait_outer = dispatch_call(&source, "through_trait");
    let ast::Expr::MethodCallExpr(trait_call) = trait_outer
        .receiver()
        .expect("trait result is the receiver")
    else {
        panic!("trait receiver must be a method result")
    };
    let projected_call = dispatch_call(&source, "through_deref");
    let reference_call = dispatch_call(&source, "through_references");

    let trait_target = semantics
        .resolve_method_call(&trait_call)
        .expect("trait target resolves");
    let trait_owner = trait_target
        .as_assoc_item(&snapshot.db)
        .expect("trait target is associated")
        .container_or_implemented_trait(&snapshot.db)
        .expect("trait owns the selected target");
    let projected_target = semantics
        .resolve_method_call(&projected_call)
        .expect("projected target resolves");
    let declaring_type = projected_target
        .as_assoc_item(&snapshot.db)
        .expect("target is associated")
        .implementing_ty(&snapshot.db)
        .expect("Account declares the inherent target");
    let projected = semantics
        .type_of_expr(&projected_call.receiver().expect("wrapper receiver exists"))
        .expect("receiver type resolves");
    let original = projected
        .original
        .as_reference_inner()
        .expect("Wrapper is borrowed");
    let adjusted = projected
        .adjusted
        .expect("Deref adjusts the receiver")
        .as_reference_inner()
        .expect("Account is borrowed");
    let reference = semantics
        .type_of_expr(
            &reference_call
                .receiver()
                .expect("reference receiver exists"),
        )
        .expect("reference type resolves");
    let referent = reference
        .original
        .as_reference_inner()
        .expect("first reference exists")
        .as_reference_inner()
        .expect("second reference exists");
    let borrowed = reference
        .adjusted
        .expect("references are adjusted")
        .as_reference_inner()
        .expect("adjusted Account is borrowed");

    assert_eq!(trait_owner.name(&snapshot.db).as_str(), "HasAccount");
    assert_eq!(original.as_adt(), Some(Adt::Struct(wrapper)));
    assert_eq!(adjusted.as_adt(), Some(Adt::Struct(account)));
    assert_eq!(declaring_type.as_adt(), Some(Adt::Struct(account)));
    ra_ap_hir::attach_db(&snapshot.db, || {
        assert_ne!(original, adjusted);
        assert_eq!(adjusted, declaring_type);
        assert_eq!(referent, borrowed);
        assert_eq!(referent, declaring_type);
    });
}

fn dispatch_call(source: &ast::SourceFile, name: &str) -> ast::MethodCallExpr {
    source
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|function| {
            function
                .name()
                .is_some_and(|declared| declared.text() == name)
        })
        .expect("dispatch caller exists")
        .syntax()
        .descendants()
        .find_map(ast::MethodCallExpr::cast)
        .expect("dispatch caller has a call")
}

mod friend_semantics;
