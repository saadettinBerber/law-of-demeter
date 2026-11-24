use std::collections::HashMap;
use std::path::{Path, PathBuf};

use law_of_demeter_core::application::{AnalysisIssue, AnalysisIssueKind, ReadError, Target};
use ra_ap_hir::{AnyDiagnostic, Function, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast};

use crate::snapshot::Snapshot;

pub(super) fn read(
    snapshot: &Snapshot,
    owners: &HashMap<Function, Target>,
) -> (Vec<ReadError>, Vec<AnalysisIssue>) {
    ra_ap_hir::attach_db(&snapshot.db, || {
        let semantics = Semantics::new(&snapshot.db);
        let mut errors = Vec::new();
        let mut issues = Vec::new();
        let mut files = Vec::new();
        for module in snapshot.selected.modules(&snapshot.db) {
            let source = semantics.module_definition_node(module);
            let Ok((path, _)) = snapshot.location(&semantics, &source.value) else {
                continue;
            };
            let file = semantics.hir_file_for(&source.value);
            if !files.contains(&file) {
                files.push(file);
                if let Some(range) = semantics.original_range_opt(&source.value)
                    && let Some(parse_errors) = range.file_id.parse_errors(&snapshot.db)
                {
                    let source_file = semantics.parse(range.file_id);
                    for error in parse_errors.iter() {
                        let text = source_file.syntax().text().to_string();
                        let offset = u32::from(error.range().start()) as usize;
                        let line = text[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1;
                        let error = ReadError::at_line(&path, line, &error.to_string());
                        issues.push(AnalysisIssue::new(AnalysisIssueKind::Parse, error.clone()));
                        errors.push(error);
                    }
                }
            }
            let mut diagnostics = Vec::new();
            module.diagnostics(&snapshot.db, &mut diagnostics, false);
            for diagnostic in diagnostics {
                let (kind, node, message) = match diagnostic {
                    AnyDiagnostic::InactiveCode(_)
                    | AnyDiagnostic::IncorrectCase(_)
                    | AnyDiagnostic::RemoveTrailingReturn(_)
                    | AnyDiagnostic::RemoveUnnecessaryElse(_)
                    | AnyDiagnostic::UnusedMustUse(_)
                    | AnyDiagnostic::ReplaceFilterMapNextWithFindMap(_) => continue,
                    AnyDiagnostic::UnresolvedModule(diagnostic) => {
                        let root = semantics.parse_or_expand(diagnostic.decl.file_id);
                        let node = diagnostic.decl.value.to_node(&root);
                        let error = location(
                            snapshot,
                            &semantics,
                            node.syntax(),
                            &path,
                            "module source cannot be read",
                        );
                        let missing_path = diagnostic.candidates.first().and_then(|candidate| {
                            path.parent().map(|directory| directory.join(candidate))
                        });
                        errors.push(ReadError::new(
                            missing_path.as_deref().unwrap_or(&path),
                            "module source cannot be read",
                        ));
                        issues.push(AnalysisIssue::new(AnalysisIssueKind::Read, error));
                        continue;
                    }
                    AnyDiagnostic::TypeMismatch(diagnostic) => {
                        let root = semantics.parse_or_expand(diagnostic.expr_or_pat.file_id);
                        (
                            AnalysisIssueKind::Compilation,
                            diagnostic.expr_or_pat.value.to_node(&root).syntax().clone(),
                            "expression type does not match its expected type",
                        )
                    }
                    AnyDiagnostic::UnresolvedMethodCall(diagnostic) => {
                        let root = semantics.parse_or_expand(diagnostic.expr.file_id);
                        (
                            AnalysisIssueKind::TargetResolution,
                            diagnostic.expr.value.to_node(&root).syntax().clone(),
                            "method target is unresolved",
                        )
                    }
                    AnyDiagnostic::UnresolvedMacroCall(_)
                    | AnyDiagnostic::MacroError(_)
                    | AnyDiagnostic::MacroExpansionParseError(_)
                    | AnyDiagnostic::MacroDefError(_)
                    | AnyDiagnostic::UnimplementedBuiltinMacro(_) => (
                        AnalysisIssueKind::Expansion,
                        source.value.clone(),
                        "macro expansion is incomplete",
                    ),
                    _ => (
                        AnalysisIssueKind::Compilation,
                        source.value.clone(),
                        "source diagnostic prevents complete analysis",
                    ),
                };
                let error = location(snapshot, &semantics, &node, &path, message);
                let mut issue = AnalysisIssue::new(kind, error);
                if let Some(owner) = node
                    .ancestors()
                    .find_map(ast::Fn::cast)
                    .and_then(|function| semantics.to_def(&function))
                    .and_then(|function| owners.get(&function))
                {
                    issue = issue.in_body(owner.clone());
                }
                issues.push(issue);
            }
        }
        (errors, issues)
    })
}

fn location(
    snapshot: &Snapshot,
    semantics: &Semantics<'_, RootDatabase>,
    node: &SyntaxNode,
    path: &Path,
    message: &str,
) -> ReadError {
    match snapshot.location(semantics, node) {
        Ok((path, line)) => ReadError::at_line(&path, line, message),
        Err(_) => ReadError::new(path, message),
    }
}

pub(super) fn invalid_paths(errors: &[ReadError]) -> Vec<PathBuf> {
    errors
        .iter()
        .map(|error| error.path().to_path_buf())
        .collect()
}
