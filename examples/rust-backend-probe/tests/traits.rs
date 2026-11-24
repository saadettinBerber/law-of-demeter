mod support;

use std::path::Path;

use ra_ap_hir::{Adt, Function, GenericParam, ModuleDef, Mutability, PathResolution, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{AstNode, Edition, ast, ast::HasArgList, ast::HasGenericParams, ast::HasName};

use support::{
    LoadedFixture, ProbeResult, declared_function, declared_method, declared_struct, flag,
    load_fixture_at, method_call,
};

#[test]
fn a_generic_bound_keeps_its_projection_while_the_concrete_call_returns_u32() -> ProbeResult {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/traits")
        .canonicalize()?;
    let fixture = load_fixture_at(
        &directory,
        "../../../rust-method-resolution/selection.rs",
        true,
    )?;
    ra_ap_hir::attach_db(&fixture.db, || {
        let semantics = Semantics::new(&fixture.db);
        let source = fixture.source(&semantics);
        let quote = declared_trait(&source, "Quote");
        let quote_trait = semantics
            .to_def(&quote)
            .ok_or("Quote trait is unresolved")?;
        let trait_quote = semantics
            .to_def(&trait_method(&quote, "quote"))
            .ok_or("Quote::quote is unresolved")?;
        let output = quote
            .assoc_item_list()
            .ok_or("Quote items are missing")?
            .syntax()
            .children()
            .find_map(ast::TypeAlias::cast)
            .ok_or("Output declaration is missing")?;
        let output = semantics.to_def(&output).ok_or("Output is unresolved")?;
        let generic = declared_function(&source, "quote_from_bound");
        let generic_function = semantics
            .to_def(&generic)
            .ok_or("quote_from_bound is unresolved")?;
        let parameter = generic
            .generic_param_list()
            .ok_or("generic parameters are missing")?
            .syntax()
            .children()
            .find_map(ast::TypeParam::cast)
            .ok_or("T declaration is missing")?;
        let parameter = semantics.to_def(&parameter).ok_or("T is unresolved")?;
        let quote_source = semantics
            .to_def(&declared_struct(&source, "QuoteSource"))
            .ok_or("QuoteSource is unresolved")?;
        let amount = quote_source
            .fields(&fixture.db)
            .into_iter()
            .find(|field| field.name(&fixture.db).as_str() == "amount")
            .ok_or("amount field is missing")?;
        let expected_u32 = amount.ty(&fixture.db);
        let expected_projection = parameter
            .ty(&fixture.db)
            .normalize_trait_assoc_type(&fixture.db, &[], output)
            .ok_or("T::Output normalization is unavailable")?;
        let inherent_quote = semantics
            .to_def(&declared_method(&source, "QuoteSource", "quote"))
            .ok_or("inherent quote is unresolved")?;
        let caller = declared_function(
            &source,
            "a_generic_bound_selects_the_trait_method_instead_of_the_concrete_inherent_method",
        );

        let bound_call = method_call(&generic, "quote");
        let bound_target = semantics
            .resolve_method_call(&bound_call)
            .ok_or("generic target is unresolved")?;
        let bound_result = semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(bound_call.clone()))
            .ok_or("generic result is unresolved")?;
        let concrete_call = initializer(&caller, "generic");
        let concrete_result = semantics
            .type_of_expr(&concrete_call)
            .ok_or("concrete result is unresolved")?;
        let concrete_call = call_expression(concrete_call)?;
        let callable = semantics
            .resolve_expr_as_callable(&concrete_call.expr().ok_or("concrete callee is missing")?)
            .ok_or("concrete callable is unresolved")?;
        let callable_result = callable.return_type();
        let params = callable.params();
        let actual_source = params
            .first()
            .ok_or("concrete parameter is missing")?
            .ty()
            .as_reference_inner()
            .ok_or("concrete parameter is not a reference")?;
        let normalized_output = actual_source
            .normalize_trait_assoc_type(&fixture.db, &[], output)
            .ok_or("concrete Output normalization is unavailable")?;

        assert_eq!(bound_target, trait_quote);
        assert_ne!(bound_target, inherent_quote);
        assert!(!bound_result.original.contains_unknown());
        assert_eq!(bound_result.original, expected_projection);
        assert_eq!(
            bound_result.original,
            generic_function.ret_type(&fixture.db)
        );
        assert_eq!(
            bound_result
                .original
                .as_associated_type_parent_trait(&fixture.db),
            Some(quote_trait)
        );
        assert_eq!(bound_result.original.generic_params(&fixture.db).len(), 1);
        assert!(
            bound_result
                .original
                .generic_params(&fixture.db)
                .contains(&GenericParam::TypeParam(parameter))
        );
        assert_ne!(bound_result.original, expected_u32);
        assert_eq!(callable.as_function(), Some(generic_function));
        assert!(!actual_source.contains_unknown());
        assert_eq!(actual_source.as_adt(), Some(Adt::Struct(quote_source)));
        assert!(!concrete_result.original.contains_unknown());
        assert!(!callable_result.contains_unknown());
        assert!(!normalized_output.contains_unknown());
        assert_eq!(concrete_result.original, expected_u32);
        assert_eq!(normalized_output, expected_u32);
        assert_eq!(
            callable_result.as_associated_type_parent_trait(&fixture.db),
            Some(quote_trait)
        );
        assert!(callable_result.generic_params(&fixture.db).is_empty());
        fixture.assert_location(&semantics, bound_call.syntax(), 218)?;
        fixture.assert_location(&semantics, concrete_call.syntax(), 65)?;
        assert_test_context(&fixture, &semantics, &caller)?;
        Ok(())
    })
}

#[test]
fn qualified_method_paths_keep_self_while_an_associated_function_has_only_an_argument()
-> ProbeResult {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/traits")
        .canonicalize()?;
    let fixture = load_fixture_at(
        &directory,
        "../../../rust-method-resolution/selection.rs",
        true,
    )?;
    ra_ap_hir::attach_db(&fixture.db, || {
        let semantics = Semantics::new(&fixture.db);
        let source = fixture.source(&semantics);
        let quote = declared_trait(&source, "Quote");
        let trait_quote = semantics
            .to_def(&trait_method(&quote, "quote"))
            .ok_or("Quote::quote is unresolved")?;
        let inherent_quote = semantics
            .to_def(&declared_method(&source, "QuoteSource", "quote"))
            .ok_or("inherent quote is unresolved")?;
        let implemented_quote = semantics
            .to_def(&implementation_method(
                &trait_implementation(&source, "QuoteSource"),
                "quote",
            ))
            .ok_or("implemented quote is unresolved")?;
        let forwarded = semantics
            .to_def(&declared_method(&source, "QuoteSource", "forwarded_wallet"))
            .ok_or("forwarded_wallet is unresolved")?;
        let quote_source = semantics
            .to_def(&declared_struct(&source, "QuoteSource"))
            .ok_or("QuoteSource is unresolved")?;
        let caller = declared_function(
            &source,
            "trait_qualification_can_differ_from_dot_and_type_qualified_receiver_calls",
        );
        let forwarded_caller = declared_function(
            &source,
            "a_receiverless_associated_function_can_return_an_existing_argument",
        );

        let ast::Expr::MethodCallExpr(dot) = initializer(&caller, "from_dot") else {
            return Err("dot expression is not a method call".into());
        };
        let type_call = call_expression(initializer(&caller, "from_type"))?;
        let trait_call = call_expression(initializer(&caller, "from_trait"))?;
        let forwarded_call = call_expression(initializer(&forwarded_caller, "returned"))?;
        let dot_target = semantics
            .resolve_method_call(&dot)
            .ok_or("dot target is unresolved")?;
        let type_target = path_target(&semantics, &type_call)?;
        let trait_target = path_target(&semantics, &trait_call)?;
        let dot_callable = semantics
            .resolve_method_call_as_callable(&dot)
            .ok_or("dot callable is unresolved")?;
        let type_callable = semantics
            .resolve_expr_as_callable(&type_call.expr().ok_or("type callee is missing")?)
            .ok_or("type callable is unresolved")?;
        let trait_callable = semantics
            .resolve_expr_as_callable(&trait_call.expr().ok_or("trait callee is missing")?)
            .ok_or("trait callable is unresolved")?;
        let forwarded_callable = semantics
            .resolve_expr_as_callable(&forwarded_call.expr().ok_or("forwarded callee is missing")?)
            .ok_or("forwarded callable is unresolved")?;

        assert_eq!(dot_target, inherent_quote);
        assert_eq!(type_target, inherent_quote);
        assert_eq!(trait_target, implemented_quote);
        assert_ne!(trait_target, inherent_quote);
        assert_eq!(dot_callable.as_function(), Some(inherent_quote));
        assert_eq!(type_callable.as_function(), Some(inherent_quote));
        assert_eq!(trait_callable.as_function(), Some(trait_quote));
        let (dot_self, dot_self_type) = dot_callable
            .receiver_param(&fixture.db)
            .ok_or("dot self is missing")?;
        assert_eq!(Some(dot_self), inherent_quote.self_param(&fixture.db));
        assert!(!dot_self_type.contains_unknown());
        assert_eq!(
            dot_self_type
                .as_reference_inner()
                .ok_or("dot self is not borrowed")?
                .as_adt(),
            Some(Adt::Struct(quote_source))
        );
        assert_eq!(dot_callable.n_params(), 0);
        assert!(type_callable.receiver_param(&fixture.db).is_none());
        assert!(trait_callable.receiver_param(&fixture.db).is_none());
        for (call, callable, declaration) in [
            (&type_call, &type_callable, inherent_quote),
            (&trait_call, &trait_callable, trait_quote),
        ] {
            let params = callable.params();
            assert_eq!(params.len(), 1);
            let self_param = params[0]
                .as_local(&fixture.db)
                .and_then(|local| local.as_self_param(&fixture.db));
            assert_eq!(self_param, declaration.self_param(&fixture.db));
            assert!(self_param.is_some());
            let argument = call
                .arg_list()
                .and_then(|args| args.args().next())
                .ok_or("qualified self argument is missing")?;
            let argument_type = semantics
                .type_of_expr(&argument)
                .ok_or("qualified self argument type is unresolved")?;
            assert!(!argument_type.original.contains_unknown());
            assert!(!params[0].ty().contains_unknown());
            assert_eq!(argument_type.original, *params[0].ty());
            assert_eq!(
                argument_type
                    .original
                    .as_reference_inner()
                    .ok_or("qualified self is not borrowed")?
                    .as_adt(),
                Some(Adt::Struct(quote_source))
            );
        }
        assert_eq!(forwarded_callable.as_function(), Some(forwarded));
        assert!(forwarded.self_param(&fixture.db).is_none());
        assert!(forwarded_callable.receiver_param(&fixture.db).is_none());
        let forwarded_params = forwarded_callable.params();
        assert_eq!(forwarded_params.len(), 1);
        assert!(
            !forwarded_params[0]
                .as_local(&fixture.db)
                .ok_or("forwarded argument has no binding")?
                .is_self(&fixture.db)
        );
        assert!(!forwarded_params[0].ty().contains_unknown());
        assert_test_context(&fixture, &semantics, &caller)?;
        Ok(())
    })
}

#[test]
fn a_dyn_call_resolves_the_trait_declaration_while_the_override_is_a_distinct_body() -> ProbeResult
{
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/traits")
        .canonicalize()?;
    let fixture = load_fixture_at(&directory, "../../../rust-visibility/helpers.rs", true)?;
    ra_ap_hir::attach_db(&fixture.db, || {
        let semantics = Semantics::new(&fixture.db);
        let source = fixture.source(&semantics);
        let wallet_source = declared_trait(&source, "WalletSource");
        let trait_definition = semantics
            .to_def(&wallet_source)
            .ok_or("WalletSource is unresolved")?;
        let default_ast = trait_method(&wallet_source, "selected");
        let trait_selected = semantics
            .to_def(&default_ast)
            .ok_or("trait selected declaration is unresolved")?;
        let override_ast =
            implementation_method(&trait_implementation(&source, "Replaced<'_>"), "selected");
        let overridden = semantics
            .to_def(&override_ast)
            .ok_or("overridden selected is unresolved")?;
        let caller = declared_function(&source, "from_public_source");
        let runtime_case = declared_function(
            &source,
            "a_public_trait_object_uses_an_overridden_selected_method",
        );

        let call = method_call(&caller, "selected");
        let receiver = semantics
            .type_of_expr(&call.receiver().ok_or("dyn receiver is missing")?)
            .ok_or("dyn receiver type is unresolved")?;
        let target = semantics
            .resolve_method_call(&call)
            .ok_or("dyn declaration is unresolved")?;
        let callable = semantics
            .resolve_method_call_as_callable(&call)
            .ok_or("dyn callable is unresolved")?;

        assert!(!receiver.original.contains_unknown());
        let (referent, mutability) = receiver
            .original
            .as_reference()
            .ok_or("dyn receiver is not a reference")?;
        assert_eq!(mutability, Mutability::Shared);
        assert_eq!(referent.as_dyn_trait(), Some(trait_definition));
        assert_eq!(target, trait_selected);
        assert_eq!(callable.as_function(), Some(trait_selected));
        assert_ne!(trait_selected, overridden);
        assert!(default_ast.body().is_some());
        assert!(override_ast.body().is_some());
        assert_eq!(
            callable
                .receiver_param(&fixture.db)
                .ok_or("bound dyn self is missing")?
                .0,
            trait_selected
                .self_param(&fixture.db)
                .ok_or("trait self is missing")?
        );
        assert!(!callable.return_type().contains_unknown());
        fixture.assert_location(&semantics, call.syntax(), 109)?;
        fixture.assert_location(&semantics, override_ast.syntax(), 192)?;
        assert_test_context(&fixture, &semantics, &runtime_case)?;
        Ok(())
    })
}

fn assert_test_context(
    fixture: &LoadedFixture,
    semantics: &Semantics<'_, RootDatabase>,
    test: &ast::Fn,
) -> ProbeResult {
    assert_eq!(fixture.krate.edition(&fixture.db), Edition::Edition2024);
    assert!(fixture.krate.cfg(&fixture.db).check_atom(&flag("test")));
    assert!(
        !fixture
            .krate
            .cfg(&fixture.db)
            .check_atom(&flag("rust_analyzer"))
    );
    assert!(
        semantics
            .to_def(test)
            .ok_or("test function is inactive or unresolved")?
            .is_test(&fixture.db)
    );
    for module in fixture.krate.modules(&fixture.db) {
        let mut diagnostics = Vec::new();
        module.diagnostics(&fixture.db, &mut diagnostics, false);
        assert!(
            diagnostics.is_empty(),
            "unexpected test-context diagnostics: {diagnostics:?}"
        );
    }
    Ok(())
}

fn initializer(caller: &ast::Fn, name: &str) -> ast::Expr {
    caller
        .body()
        .expect("fixture caller has no body")
        .syntax()
        .descendants()
        .filter_map(ast::LetStmt::cast)
        .find(|statement| {
            statement
                .pat()
                .and_then(|pattern| ast::IdentPat::cast(pattern.syntax().clone()))
                .and_then(|pattern| pattern.name())
                .is_some_and(|binding| binding.text() == name)
        })
        .and_then(|statement| statement.initializer())
        .expect("fixture initializer is missing")
}

fn call_expression(expression: ast::Expr) -> ProbeResult<ast::CallExpr> {
    match expression {
        ast::Expr::CallExpr(call) => Ok(call),
        _ => Err("fixture expression is not a path call".into()),
    }
}

fn path_target(
    semantics: &Semantics<'_, RootDatabase>,
    call: &ast::CallExpr,
) -> ProbeResult<Function> {
    let Some(ast::Expr::PathExpr(path)) = call.expr() else {
        return Err("fixture callee is not a path".into());
    };
    match semantics.resolve_path(&path.path().ok_or("callee path is missing")?) {
        Some(PathResolution::Def(ModuleDef::Function(function))) => Ok(function),
        _ => Err("callee function is unresolved".into()),
    }
}

fn declared_trait(source: &ast::SourceFile, name: &str) -> ast::Trait {
    source
        .syntax()
        .children()
        .filter_map(ast::Trait::cast)
        .find(|trait_| {
            trait_
                .name()
                .is_some_and(|trait_name| trait_name.text() == name)
        })
        .expect("fixture trait declaration is missing")
}

fn trait_method(trait_: &ast::Trait, name: &str) -> ast::Fn {
    trait_
        .assoc_item_list()
        .expect("fixture trait items are missing")
        .syntax()
        .children()
        .filter_map(ast::Fn::cast)
        .find(|method| {
            method
                .name()
                .is_some_and(|method_name| method_name.text() == name)
        })
        .expect("fixture trait method is missing")
}

fn trait_implementation(source: &ast::SourceFile, owner: &str) -> ast::Impl {
    source
        .syntax()
        .children()
        .filter_map(ast::Impl::cast)
        .find(|implementation| {
            implementation.trait_().is_some()
                && implementation
                    .self_ty()
                    .is_some_and(|ty| ty.syntax().text() == owner)
        })
        .expect("fixture trait implementation is missing")
}

fn implementation_method(implementation: &ast::Impl, name: &str) -> ast::Fn {
    implementation
        .assoc_item_list()
        .expect("fixture implementation items are missing")
        .syntax()
        .children()
        .filter_map(ast::Fn::cast)
        .find(|method| {
            method
                .name()
                .is_some_and(|method_name| method_name.text() == name)
        })
        .expect("fixture implementation method is missing")
}
