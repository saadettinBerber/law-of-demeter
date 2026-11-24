use super::{ProbeResult, function, initializer, load_model, macro_definition};
use ra_ap_hir::{AnyDiagnostic, Semantics};
use ra_ap_syntax::{AstNode, Edition, ast};

pub(super) fn check_call_orders() -> ProbeResult {
    let model = load_model(true)?;

    ra_ap_hir::attach_db(&model.db, || {
        let semantics = Semantics::new(&model.db);
        let source = model.source(&semantics, model.consumer);
        let provider = model.source(&semantics, model.provider);
        let borrowed_reference = initializer(&function(&source, "definition_count"), "reference")?;
        let owned_reference = initializer(&function(&source, "callsite_count"), "reference")?;
        let borrowed_type = semantics
            .type_of_expr(&borrowed_reference)
            .ok_or("borrowed control is unresolved")?
            .original;
        let owned_type = semantics
            .type_of_expr(&owned_reference)
            .ok_or("owned control is unresolved")?
            .original;
        let borrowed_call = ast::MethodCallExpr::cast(borrowed_reference.syntax().clone())
            .ok_or("borrowed control call is missing")?;
        let owned_call = ast::MethodCallExpr::cast(owned_reference.syntax().clone())
            .ok_or("owned control call is missing")?;
        let borrowed_target = semantics
            .resolve_method_call(&borrowed_call)
            .ok_or("borrowed control target is unresolved")?;
        let owned_target = semantics
            .resolve_method_call(&owned_call)
            .ok_or("owned control target is unresolved")?;
        assert!(!borrowed_type.contains_unknown());
        assert!(!owned_type.contains_unknown());
        assert_ne!(borrowed_type, owned_type);
        assert_ne!(borrowed_target, owned_target);
        for name in [
            "a_legacy_token_does_not_change_the_following_modern_token",
            "a_modern_token_does_not_change_the_following_legacy_token",
        ] {
            let caller = function(&source, name);
            let definition = semantics.to_def(&caller).ok_or("order test is inactive")?;
            assert!(definition.is_test(&model.db));
            for (binding, declaration, edition, expected_type, expected_target) in [
                (
                    "legacy",
                    "definition_items",
                    Edition::Edition2018,
                    &borrowed_type,
                    borrowed_target,
                ),
                (
                    "modern",
                    "callsite_items",
                    Edition::Edition2024,
                    &owned_type,
                    owned_target,
                ),
            ] {
                let expression = initializer(&caller, binding)?;
                let invocation = expression
                    .syntax()
                    .descendants()
                    .find_map(ast::MacroCall::cast)
                    .ok_or("order invocation is missing")?;
                let expected_macro = macro_definition(&semantics, &provider, declaration)?;
                assert_eq!(
                    semantics.resolve_macro_call(&invocation),
                    Some(expected_macro)
                );
                let id = semantics
                    .to_def(&invocation)
                    .ok_or("order invocation is unresolved")?;
                let expansion = semantics.expand(id);
                assert!(expansion.err.is_none(), "order expansion failed");
                let mut calls = expansion
                    .value
                    .descendants()
                    .filter_map(ast::MethodCallExpr::cast);
                let call = calls.next().ok_or("order expanded call is missing")?;
                assert!(calls.next().is_none());
                let method = call.name_ref().ok_or("order method token is missing")?;
                let span = id
                    .expansion_span_map(&model.db)
                    .span_at(method.syntax().text_range().start());
                let result = semantics
                    .type_of_expr(&expression)
                    .ok_or("order result is unresolved")?;
                let target = semantics
                    .resolve_method_call(&call)
                    .ok_or("order target is unresolved")?;

                assert_eq!(span.ctx.edition(&model.db), edition);
                assert!(!result.original.contains_unknown());
                assert_eq!(&result.original, expected_type, "{name}: {binding} result");
                assert_eq!(target, expected_target, "{name}: {binding} target");
            }
            eprintln!("Order control passed: {name}");
        }
        let mut diagnostics = Vec::new();
        model
            .consumer
            .root_module(&model.db)
            .diagnostics(&model.db, &mut diagnostics, false);
        assert!(
            diagnostics
                .iter()
                .all(|item| matches!(item, AnyDiagnostic::InactiveCode(_))),
            "order diagnostics: {diagnostics:?}"
        );
        Ok(())
    })
}
