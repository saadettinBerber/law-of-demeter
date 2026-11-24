use super::{
    BuiltFixture, ProbeResult, active_function, assert_local_diagnostics, declared_method,
    declared_struct, flag, integer_value, load_prepared, method_call, prepare_fixture,
};
use std::path::{Path, PathBuf};

use ra_ap_hir::{Adt, EditionedFileId, FileRange, HirFileId, InFile, InMacroFile, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_load_cargo::ProcMacroServerChoice;
use ra_ap_project_model::RustLibSource;
use ra_ap_syntax::{AstNode, SyntaxNodePtr, ast, ast::HasArgList, ast::HasName};

#[test]
fn one_declarative_invocation_keeps_two_distinct_semantic_charge_sites() -> ProbeResult {
    let fixture = load_test_context()?;

    ra_ap_hir::attach_db(&fixture.db, || {
        let semantics = Semantics::new(&fixture.db);
        let source = fixture.source(&semantics);
        let expected_charge = semantics
            .to_def(&declared_method(&source, "Wallet", "charge"))
            .ok_or("Wallet::charge is unresolved")?;
        let wallet = semantics
            .to_def(&declared_struct(&source, "Wallet"))
            .ok_or("Wallet is unresolved")?;
        let caller = source_function(
            &source,
            "one_declarative_invocation_generates_two_distinct_charges",
        )?;
        let invocation = invocation(&caller, "expansion_legacy::charge_twice")?;
        let id = semantics
            .to_def(&invocation)
            .ok_or("invocation is unresolved")?;
        let definition = semantics
            .resolve_macro_call(&invocation)
            .ok_or("macro is unresolved")?;
        let provider = definition.module(&fixture.db).krate(&fixture.db);
        let provider_file = EditionedFileId::new(
            &fixture.db,
            provider.root_file(&fixture.db),
            provider.edition(&fixture.db),
        );
        assert!(provider_file.parse_errors(&fixture.db).is_none());
        let provider_source = semantics.parse(provider_file);
        let declared_macro = provider_source
            .syntax()
            .children()
            .filter_map(ast::MacroRules::cast)
            .find(|item| {
                item.name()
                    .is_some_and(|name| name.text() == "charge_twice")
            })
            .ok_or("provider declaration is missing")?;
        let expected_macro = semantics
            .to_def(&ast::Macro::MacroRules(declared_macro.clone()))
            .ok_or("provider macro is unresolved")?;
        let macro_source = semantics
            .source(definition)
            .ok_or("macro definition has no source")?;
        let consumer_file = EditionedFileId::new(
            &fixture.db,
            fixture.consumer.root_file(&fixture.db),
            fixture.consumer.edition(&fixture.db),
        );
        let invocation_range = invocation
            .path()
            .ok_or("invocation path is missing")?
            .syntax()
            .text_range()
            .cover(
                invocation
                    .excl_token()
                    .ok_or("invocation ! is missing")?
                    .text_range(),
            );
        let expansion = semantics.expand(id);
        assert!(expansion.err.is_none(), "declarative expansion failed");
        let calls: Vec<_> = expansion
            .value
            .descendants()
            .filter_map(ast::MethodCallExpr::cast)
            .filter(|call| call.name_ref().is_some_and(|name| name.text() == "charge"))
            .collect();
        assert_eq!(calls.len(), 2);
        let repeated_id = semantics
            .to_def(&invocation)
            .ok_or("repeated invocation is unresolved")?;
        let repeated = semantics.expand(repeated_id);
        assert!(repeated.err.is_none(), "repeated expansion failed");
        let repeated_calls: Vec<_> = repeated
            .value
            .descendants()
            .filter_map(ast::MethodCallExpr::cast)
            .filter(|call| call.name_ref().is_some_and(|name| name.text() == "charge"))
            .collect();
        let keys: Vec<_> = calls
            .iter()
            .map(|call| call_key(&semantics, call))
            .collect();
        let repeated_keys: Vec<_> = repeated_calls
            .iter()
            .map(|call| call_key(&semantics, call))
            .collect();

        assert_eq!(definition, expected_macro);
        assert_eq!(macro_source.file_id, HirFileId::from(provider_file));
        assert_eq!(
            SyntaxNodePtr::new(macro_source.value.syntax()),
            SyntaxNodePtr::new(declared_macro.syntax())
        );
        assert_eq!(
            semantics.hir_file_for(invocation.syntax()),
            HirFileId::from(consumer_file)
        );
        assert!(
            semantics
                .to_def(&caller)
                .ok_or("declarative test is inactive")?
                .is_test(&fixture.db)
        );
        assert_ne!(provider, fixture.consumer);
        assert_eq!(
            physical_path(&fixture, provider_file)?,
            fixture_path("legacy/src/lib.rs")?
        );
        assert_eq!(id, repeated_id);
        assert_eq!(keys, repeated_keys);
        assert_ne!(keys[0], keys[1]);
        for (index, (call, amount)) in calls.iter().zip([7, 11]).enumerate() {
            assert_eq!(
                semantics.hir_file_for(call.syntax()),
                HirFileId::MacroFile(id)
            );
            assert_eq!(semantics.resolve_method_call(call), Some(expected_charge));
            let receiver = semantics
                .type_of_expr(&call.receiver().ok_or("receiver is missing")?)
                .ok_or("receiver type is unresolved")?;
            assert!(!receiver.original.contains_unknown());
            assert_eq!(
                receiver
                    .original
                    .as_reference_inner()
                    .ok_or("receiver is not borrowed")?
                    .as_adt(),
                Some(Adt::Struct(wallet))
            );
            assert_eq!(
                integer_value(
                    &call
                        .arg_list()
                        .and_then(|args| args.args().next())
                        .ok_or("amount is missing")?
                )?,
                amount
            );
            let token = call.name_ref().ok_or("expanded method token is missing")?;
            let (token_origin, context) = InMacroFile::new(id, token.syntax().text_range().start())
                .original_file_range(&fixture.db);
            let expected_token = declared_macro
                .syntax()
                .descendants_with_tokens()
                .filter_map(|item| item.into_token())
                .filter(|item| item.text() == "charge")
                .nth(index)
                .ok_or("definition charge token is missing")?;
            assert_eq!(
                token_origin,
                FileRange {
                    file_id: provider_file,
                    range: expected_token.text_range()
                }
            );
            assert!(!context.is_root());
            assert_eq!(semantics.original_range_opt(call.syntax()), None);
            assert_eq!(
                semantics.original_range(call.syntax()),
                FileRange {
                    file_id: consumer_file,
                    range: invocation_range
                }
            );
            eprintln!(
                "declarative site: key={:?}, opt={:?}, rooted={:?}, nonroot={:?}",
                call_key(&semantics, call),
                semantics.original_range_opt(call.syntax()),
                semantics.original_range(call.syntax()),
                InFile::new(HirFileId::MacroFile(id), call.syntax().text_range())
                    .original_node_file_range(&fixture.db)
            );
        }
        assert_local_diagnostics(&fixture);
        Ok(())
    })
}

#[test]
fn proc_generated_calls_keep_the_invocation_and_provider_definition_separate() -> ProbeResult {
    let fixture = load_test_context()?;

    ra_ap_hir::attach_db(&fixture.db, || {
        let semantics = Semantics::new(&fixture.db);
        let source = fixture.source(&semantics);
        let caller = source_function(
            &source,
            "a_function_like_proc_macro_generates_a_charge_in_the_consumer",
        )?;
        let invocation = invocation(&caller, "expansion_generator::generated_charge")?;
        let id = semantics
            .to_def(&invocation)
            .ok_or("proc invocation is unresolved")?;
        let definition = semantics
            .resolve_macro_call(&invocation)
            .ok_or("proc macro is unresolved")?;
        let definition_source = semantics
            .source(definition)
            .ok_or("proc definition has no source")?;
        let provider = definition.module(&fixture.db).krate(&fixture.db);
        let provider_file = EditionedFileId::new(
            &fixture.db,
            provider.root_file(&fixture.db),
            provider.edition(&fixture.db),
        );
        assert!(provider_file.parse_errors(&fixture.db).is_none());
        let provider_source = semantics.parse(provider_file);
        let expected_definition = super::declared_function(&provider_source, "generated_charge");
        let expansion = semantics.expand(id);
        assert!(expansion.err.is_none(), "proc expansion failed");
        let calls: Vec<_> = expansion
            .value
            .descendants()
            .filter_map(ast::MethodCallExpr::cast)
            .collect();

        let consumer_file = EditionedFileId::new(
            &fixture.db,
            fixture.consumer.root_file(&fixture.db),
            fixture.consumer.edition(&fixture.db),
        );
        let receiver_input = invocation
            .token_tree()
            .ok_or("proc input is missing")?
            .syntax()
            .descendants_with_tokens()
            .filter_map(|item| item.into_token())
            .find(|item| item.text() == "wallet")
            .ok_or("wallet input is missing")?;
        let wallet = semantics
            .to_def(&declared_struct(&source, "Wallet"))
            .ok_or("consumer Wallet is unresolved")?;

        assert!(
            semantics
                .to_def(&caller)
                .ok_or("proc test is inactive")?
                .is_test(&fixture.db)
        );
        assert!(definition.is_proc_macro());
        assert_ne!(provider, fixture.consumer);
        assert_eq!(definition_source.file_id, HirFileId::from(provider_file));
        assert_eq!(
            SyntaxNodePtr::new(definition_source.value.syntax()),
            SyntaxNodePtr::new(expected_definition.syntax())
        );
        assert_eq!(
            physical_path(&fixture, provider_file)?,
            fixture_path("generator/src/lib.rs")?
        );
        assert_eq!(calls.len(), 2);
        assert_ne!(
            call_key(&semantics, &calls[0]),
            call_key(&semantics, &calls[1])
        );
        for (call, name) in calls.iter().zip(["charge", "balance"]) {
            let expected_target = semantics
                .to_def(&declared_method(&source, "Wallet", name))
                .ok_or("Wallet target is unresolved")?;
            assert_eq!(semantics.resolve_method_call(call), Some(expected_target));
            assert_eq!(
                semantics.hir_file_for(call.syntax()),
                HirFileId::MacroFile(id)
            );
            let receiver = semantics
                .type_of_expr(&call.receiver().ok_or("proc receiver is missing")?)
                .ok_or("proc receiver type is unresolved")?;
            assert!(!receiver.original.contains_unknown());
            assert_eq!(receiver.original.as_adt(), Some(Adt::Struct(wallet)));
            let result = semantics
                .type_of_expr(&ast::Expr::MethodCallExpr(call.clone()))
                .ok_or("proc result is unresolved")?;
            assert!(!result.original.contains_unknown());
            assert_eq!(result.original, expected_target.ret_type(&fixture.db));
            assert_eq!(semantics.original_range_opt(call.syntax()), None);
            assert_eq!(
                semantics.original_range(call.syntax()),
                FileRange {
                    file_id: consumer_file,
                    range: receiver_input.text_range()
                }
            );
            let token = call.name_ref().ok_or("proc method token is missing")?;
            let (origin, context) = InMacroFile::new(id, token.syntax().text_range().start())
                .original_file_range(&fixture.db);
            let expected_anchor = expected_definition
                .name()
                .ok_or("proc definition name is missing")?;
            assert_eq!(
                origin,
                FileRange {
                    file_id: provider_file,
                    range: expected_anchor.syntax().text_range()
                }
            );
            assert!(!context.is_root());
            eprintln!(
                "proc method token: path={:?}, text={:?}, context={context:?}",
                physical_path(&fixture, origin.file_id)?,
                origin.text(&fixture.db)
            );
            eprintln!(
                "proc site: key={:?}, opt={:?}, rooted={:?}, nonroot={:?}",
                call_key(&semantics, call),
                semantics.original_range_opt(call.syntax()),
                semantics.original_range(call.syntax()),
                InFile::new(HirFileId::MacroFile(id), call.syntax().text_range())
                    .original_node_file_range(&fixture.db)
            );
        }
        assert_eq!(
            integer_value(
                &calls[0]
                    .arg_list()
                    .and_then(|args| args.args().next())
                    .ok_or("proc charge amount is missing")?
            )?,
            13
        );
        assert_local_diagnostics(&fixture);
        Ok(())
    })
}

#[test]
fn an_included_generated_call_has_an_identity_separate_from_its_physical_parse() -> ProbeResult {
    let (workspace, config) = prepare_fixture("base", false, false)?;
    let fixture = load_prepared(workspace, &config, ProcMacroServerChoice::None)?;

    ra_ap_hir::attach_db(&fixture.db, || {
        let semantics = Semantics::new(&fixture.db);
        let generated = active_function(&fixture, "generated_charge")?;
        let active_source = semantics
            .source(generated)
            .ok_or("active generated source is missing")?;
        let active_call = method_call(&active_source.value, "charge");
        let out_dir = fixture
            .consumer
            .base()
            .env(&fixture.db)
            .get("OUT_DIR")
            .ok_or("OUT_DIR is missing")?;
        let generated_path = PathBuf::from(out_dir.as_str()).join("generated.rs");
        let file = fixture
            .vfs
            .file_id(&ra_ap_vfs::VfsPath::new_real_path(
                generated_path
                    .to_str()
                    .ok_or("generated path is not UTF-8")?
                    .to_owned(),
            ))
            .ok_or("generated source is missing from VFS")?
            .0;
        let physical_file =
            EditionedFileId::new(&fixture.db, file, fixture.consumer.edition(&fixture.db));
        assert!(physical_file.parse_errors(&fixture.db).is_none());
        let physical_source = semantics.parse(physical_file);
        let physical_function = super::declared_function(&physical_source, "generated_charge");
        let physical_call = method_call(&physical_function, "charge");
        let active_key = call_key(&semantics, &active_call);
        let physical_key = call_key(&semantics, &physical_call);
        let repeated_source = semantics
            .source(generated)
            .ok_or("repeated generated source is missing")?;
        let repeated_key = call_key(&semantics, &method_call(&repeated_source.value, "charge"));
        let mapped = semantics.original_range_opt(active_call.syntax());
        let source = fixture.source(&semantics);
        let expected_charge = semantics
            .to_def(&declared_method(&source, "Wallet", "charge"))
            .ok_or("Wallet::charge is unresolved")?;
        let expected_wallet = semantics
            .to_def(&declared_struct(&source, "Wallet"))
            .ok_or("Wallet is unresolved")?;
        let receiver = semantics
            .type_of_expr(
                &active_call
                    .receiver()
                    .ok_or("generated receiver is missing")?,
            )
            .ok_or("generated receiver is unresolved")?;
        let include_id = active_key
            .file_id
            .macro_file()
            .ok_or("active source is not an expansion")?;
        let include_expansion = semantics.expand(include_id);
        assert!(include_expansion.err.is_none(), "include expansion failed");
        let raw_token = physical_call
            .name_ref()
            .ok_or("physical charge name is missing")?;
        let raw_span = HirFileId::from(physical_file)
            .span_map(&fixture.db)
            .span_for_range(raw_token.syntax().text_range());
        let info = include_id.expansion_info(&fixture.db);
        let mapped_tokens = info
            .map_range_down_exact(raw_span)
            .ok_or("include down-map is unavailable")?;
        let mapped_file = mapped_tokens.file_id;
        assert_eq!(mapped_file, include_id);
        let down_keys: Vec<_> = mapped_tokens
            .value
            .filter_map(|(token, _)| token.parent_ancestors().find_map(ast::MethodCallExpr::cast))
            .map(|call| {
                let file_id = HirFileId::MacroFile(mapped_file);
                assert_eq!(semantics.hir_file_for(call.syntax()), file_id);
                InFile::new(file_id, SyntaxNodePtr::new(call.syntax()))
            })
            .collect();

        let raw_include = source
            .syntax()
            .children()
            .find_map(ast::MacroCall::cast)
            .ok_or("raw include invocation is missing")?;
        let include_call_source = include_id.call_node(&fixture.db);
        let consumer_file = EditionedFileId::new(
            &fixture.db,
            fixture.consumer.root_file(&fixture.db),
            fixture.consumer.edition(&fixture.db),
        );

        assert_eq!(include_call_source.file_id, HirFileId::from(consumer_file));
        assert_eq!(
            SyntaxNodePtr::new(&include_call_source.value),
            SyntaxNodePtr::new(raw_include.syntax())
        );
        assert_eq!(active_key.file_id.original_file(&fixture.db), consumer_file);
        assert_eq!(active_key, repeated_key);
        assert_eq!(
            generated.module(&fixture.db).krate(&fixture.db),
            fixture.consumer
        );
        assert!(generated_path.starts_with(Path::new(fixture.target.as_str())));
        assert_ne!(active_key, physical_key);
        assert!(include_id.is_include_macro(&fixture.db));
        assert_eq!(
            active_key
                .file_id
                .original_file_respecting_includes(&fixture.db),
            physical_file
        );
        assert_eq!(
            mapped,
            Some(FileRange {
                file_id: physical_file,
                range: physical_call.syntax().text_range()
            })
        );
        assert_eq!(
            semantics.original_range(active_call.syntax()),
            mapped.ok_or("generated mapping is missing")?
        );
        assert_eq!(down_keys, [active_key]);
        assert_eq!(
            semantics.resolve_method_call(&active_call),
            Some(expected_charge)
        );
        assert!(!receiver.original.contains_unknown());
        assert_eq!(
            receiver
                .original
                .as_reference_inner()
                .ok_or("generated receiver is not borrowed")?
                .as_adt(),
            Some(Adt::Struct(expected_wallet))
        );
        assert_eq!(semantics.resolve_method_call(&physical_call), None);
        assert_eq!(semantics.to_def(&physical_function), None);
        assert_eq!(physical_key.file_id, HirFileId::from(physical_file));
        assert_eq!(
            integer_value(
                &active_call
                    .arg_list()
                    .and_then(|args| args.args().next())
                    .ok_or("active amount is missing")?
            )?,
            7
        );
        eprintln!(
            "generated: active={active_key:?}, physical={physical_key:?}, opt={mapped:?}, rooted={:?}",
            semantics.original_range(active_call.syntax())
        );
        assert_local_diagnostics(&fixture);
        Ok(())
    })
}

fn load_test_context() -> ProbeResult<BuiltFixture> {
    let (workspace, config) = prepare_fixture("base", false, true)?;
    let Some(RustLibSource::Path(sysroot)) = &config.sysroot else {
        return Err("explicit sysroot is missing".into());
    };
    let fixture = load_prepared(
        workspace,
        &config,
        ProcMacroServerChoice::Explicit(sysroot.join("libexec/rust-analyzer-proc-macro-srv")),
    )?;
    assert!(fixture.proc_client.is_some());
    assert!(fixture.consumer.cfg(&fixture.db).check_atom(&flag("test")));
    assert!(
        !fixture
            .consumer
            .cfg(&fixture.db)
            .check_atom(&flag("rust_analyzer"))
    );
    Ok(fixture)
}

fn source_function(source: &ast::SourceFile, name: &str) -> ProbeResult<ast::Fn> {
    source
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|item| {
            item.name()
                .is_some_and(|item_name| item_name.text() == name)
        })
        .ok_or_else(|| format!("source function {name} is missing").into())
}

fn invocation(caller: &ast::Fn, path: &str) -> ProbeResult<ast::MacroCall> {
    let mut invocations = caller
        .syntax()
        .descendants()
        .filter_map(ast::MacroCall::cast)
        .filter(|call| {
            call.path()
                .is_some_and(|call_path| call_path.syntax().text() == path)
        });
    let invocation = invocations.next().ok_or("source invocation is missing")?;
    assert!(
        invocations.next().is_none(),
        "source invocation is ambiguous"
    );
    Ok(invocation)
}

fn call_key(
    semantics: &Semantics<'_, RootDatabase>,
    call: &ast::MethodCallExpr,
) -> InFile<SyntaxNodePtr> {
    InFile::new(
        semantics.hir_file_for(call.syntax()),
        SyntaxNodePtr::new(call.syntax()),
    )
}

fn physical_path(fixture: &BuiltFixture, file: EditionedFileId) -> ProbeResult<PathBuf> {
    fixture
        .vfs
        .file_path(file.file_id(&fixture.db))
        .as_path()
        .map(|path| PathBuf::from(path.as_str()))
        .ok_or_else(|| "source file has no physical path".into())
}

fn fixture_path(relative: &str) -> ProbeResult<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rust-expansion")
        .join(relative)
        .canonicalize()?)
}
