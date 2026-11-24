use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use law_of_demeter_core::application::{
    AnalysisIssue, AnalysisIssueKind, ReadError, SourceFile, Target,
};
use law_of_demeter_core::domain::method::{Call, Expression, Method};
use ra_ap_hir::{
    Adt, AsAssocItem, AssocItem, Function, Impl, ModuleDef, PathResolution, Semantics, Type,
};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast, ast::HasArgList};

use crate::snapshot::Snapshot;

pub(super) fn read(
    snapshot: &Snapshot,
) -> (Vec<Result<SourceFile, ReadError>>, Vec<AnalysisIssue>) {
    let semantics = Semantics::new(&snapshot.db);
    let mut files: BTreeMap<PathBuf, (Vec<String>, Vec<Method>)> = BTreeMap::new();
    let mut issues = Vec::new();
    let mut owners = HashMap::new();
    for module in snapshot.selected.modules(&snapshot.db) {
        for declaration in module.declarations(&snapshot.db) {
            if let ModuleDef::Trait(trait_) = declaration {
                for item in trait_.items(&snapshot.db) {
                    let AssocItem::Function(function) = item else {
                        continue;
                    };
                    if !function.has_body(&snapshot.db) {
                        continue;
                    }
                    let message = "trait default method bodies are outside the supported scope";
                    let location = semantics.source(function)
                        .and_then(|source| source.value.body())
                        .and_then(|body| snapshot.location(&semantics, body.syntax()).ok())
                        .map(|(path, line)| ReadError::at_line(&path, line, message))
                        .unwrap_or_else(|| ReadError::new(&snapshot.source_root, "trait default body has no physical source location and is unsupported"));
                    let mut issue = AnalysisIssue::new(AnalysisIssueKind::Unsupported, location);
                    if let Some(owner) = snapshot.definition_identity(declaration) {
                        issue = issue
                            .in_body(Target::method(&owner, function.name(&snapshot.db).as_str()));
                    }
                    issues.push(issue);
                }
            }
            if let ModuleDef::Function(function) = declaration
                && let Some(source) = semantics.source(function)
                && let Ok((path, line)) = snapshot.location(&semantics, source.value.syntax())
            {
                issues.push(AnalysisIssue::new(
                    AnalysisIssueKind::Unsupported,
                    ReadError::at_line(
                        &path,
                        line,
                        "free functions are outside the first source slice",
                    ),
                ));
            }
            if let ModuleDef::Adt(adt) = declaration
                && let Some(source) = semantics.source(adt)
                && let Ok((path, line)) = snapshot.location(&semantics, source.value.syntax())
            {
                files
                    .entry(path.clone())
                    .or_default()
                    .0
                    .push(snapshot.identity(adt));
                if !matches!(adt, Adt::Struct(_)) {
                    issues.push(
                        AnalysisIssue::new(
                            AnalysisIssueKind::Unsupported,
                            ReadError::at_line(
                                &path,
                                line,
                                "aggregate kind is outside the first source slice",
                            ),
                        )
                        .in_body(Target::class(&snapshot.identity(adt))),
                    );
                }
            }
        }
    }
    for implementation in Impl::all_in_crate(&snapshot.db, snapshot.selected) {
        let Some(adt) = implementation.self_ty(&snapshot.db).as_adt() else {
            issues.push(AnalysisIssue::new(
                AnalysisIssueKind::Unsupported,
                ReadError::new(
                    &snapshot.source_root,
                    "implementation has no supported concrete receiver type",
                ),
            ));
            continue;
        };
        let class = snapshot.identity(adt);
        for item in implementation.items(&snapshot.db) {
            let AssocItem::Function(function) = item else {
                continue;
            };
            let owner = Target::method(&class, function.name(&snapshot.db).as_str());
            let Some(source) = semantics.source(function) else {
                issues.push(
                    AnalysisIssue::new(
                        AnalysisIssueKind::Unsupported,
                        ReadError::new(&snapshot.source_root, "method source is unavailable"),
                    )
                    .in_body(owner),
                );
                continue;
            };
            let (path, line) = match snapshot.location(&semantics, source.value.syntax()) {
                Ok(location) => location,
                Err(message) => {
                    issues.push(
                        AnalysisIssue::new(
                            AnalysisIssueKind::Read,
                            ReadError::new(&snapshot.source_root, &message),
                        )
                        .in_body(owner),
                    );
                    continue;
                }
            };
            owners.insert(
                function,
                Target::method(&class, function.name(&snapshot.db).as_str()),
            );
            let reader = BodyReader {
                snapshot,
                semantics: &semantics,
                function,
                owner: Target::method(&class, function.name(&snapshot.db).as_str()),
                path: &path,
            };
            match reader.read(&source.value, line) {
                Ok(method) => files.entry(path).or_default().1.push(method),
                Err(issue) => issues.push(issue),
            }
        }
    }
    let (errors, diagnostics) = crate::diagnostics::read(snapshot, &owners);
    for issue in diagnostics {
        if !issues.contains(&issue) {
            issues.push(issue);
        }
    }
    let invalid = crate::diagnostics::invalid_paths(&errors);
    let files = files
        .into_iter()
        .filter(|(path, _)| !invalid.contains(path))
        .map(|(path, (classes, methods))| Ok(SourceFile::new(&path, classes, methods)))
        .chain(errors.into_iter().map(Err))
        .collect();
    (files, issues)
}

struct BodyReader<'a, 'db> {
    snapshot: &'db Snapshot,
    semantics: &'a Semantics<'db, RootDatabase>,
    function: Function,
    owner: Target,
    path: &'a std::path::Path,
}

impl BodyReader<'_, '_> {
    fn read(&self, function: &ast::Fn, line: usize) -> Result<Method, AnalysisIssue> {
        let mut method = Method::new(
            self.owner.class_name(),
            self.function.name(&self.snapshot.db).as_str(),
            line,
        );
        if self.function.is_async(&self.snapshot.db)
            || self.function.is_unsafe(&self.snapshot.db)
            || self.function.self_param(&self.snapshot.db).is_none()
        {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                function.syntax(),
                "only normal receiver methods are supported",
            ));
        }
        let body = function
            .body()
            .and_then(|body| body.stmt_list())
            .ok_or_else(|| {
                self.issue(
                    AnalysisIssueKind::Unsupported,
                    function.syntax(),
                    "method body is unavailable",
                )
            })?;
        for statement in body.statements() {
            let ast::Stmt::ExprStmt(statement) = statement else {
                return Err(self.issue(
                    AnalysisIssueKind::Unsupported,
                    statement.syntax(),
                    "statement is outside the first source slice",
                ));
            };
            let expression = statement.expr().ok_or_else(|| {
                self.issue(
                    AnalysisIssueKind::Parse,
                    statement.syntax(),
                    "statement expression is missing",
                )
            })?;
            method = method.evaluate(self.expression(&expression)?);
        }
        if let Some(expression) = body.tail_expr() {
            method = method.give_back(self.expression(&expression)?);
        }
        Ok(method)
    }

    fn expression(&self, expression: &ast::Expr) -> Result<Expression, AnalysisIssue> {
        match expression {
            ast::Expr::MethodCallExpr(call) => self.call(call).map(Expression::return_of),
            ast::Expr::Literal(_) => Ok(Expression::literal()),
            ast::Expr::PathExpr(path) => {
                let resolution = path
                    .path()
                    .and_then(|path| self.semantics.resolve_path(&path));
                if let Some(PathResolution::Local(local)) = resolution {
                    if local.is_self(&self.snapshot.db) {
                        return Ok(Expression::This);
                    }
                    let parameters = self.function.params_without_self(&self.snapshot.db);
                    if let Some(parameter) = parameters
                        .iter()
                        .find(|parameter| parameter.as_local(&self.snapshot.db) == Some(local))
                    {
                        let ty = self.snapshot.type_ref(parameter.ty()).map_err(|message| {
                            self.issue(
                                AnalysisIssueKind::TypeResolution,
                                expression.syntax(),
                                &message,
                            )
                        })?;
                        return Ok(Expression::parameter(
                            local.name(&self.snapshot.db).as_str(),
                            ty,
                        ));
                    }
                }
                Err(self.issue(
                    AnalysisIssueKind::Origin,
                    expression.syntax(),
                    "expression does not resolve to a direct parameter",
                ))
            }
            _ => Err(self.issue(
                AnalysisIssueKind::Unsupported,
                expression.syntax(),
                "expression is outside the first source slice",
            )),
        }
    }

    fn call(&self, call: &ast::MethodCallExpr) -> Result<Call, AnalysisIssue> {
        let target = self.semantics.resolve_method_call(call).ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::TargetResolution,
                call.syntax(),
                "method target is unresolved",
            )
        })?;
        let associated = target.as_assoc_item(&self.snapshot.db).ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Unsupported,
                call.syntax(),
                "method target has no associated declaration",
            )
        })?;
        if associated
            .container_or_implemented_trait(&self.snapshot.db)
            .is_some()
        {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                call.syntax(),
                "trait method targets are outside the first source slice",
            ));
        }
        let receiver = call.receiver().ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Parse,
                call.syntax(),
                "method receiver is missing",
            )
        })?;
        let receiver_type = self.semantics.type_of_expr(&receiver).ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::TypeResolution,
                receiver.syntax(),
                "receiver type is unresolved",
            )
        })?;
        self.snapshot
            .type_ref(&receiver_type.original)
            .map_err(|message| {
                self.issue(
                    AnalysisIssueKind::TypeResolution,
                    receiver.syntax(),
                    &message,
                )
            })?;
        let declared = associated.implementing_ty(&self.snapshot.db);
        let same_receiver = ra_ap_hir::attach_db(&self.snapshot.db, || {
            let original = without_references(receiver_type.original.clone());
            let adjusted = without_references(
                receiver_type
                    .adjusted
                    .as_ref()
                    .unwrap_or(&receiver_type.original)
                    .clone(),
            );
            original == adjusted
                && declared.is_some_and(|declared| original == without_references(declared))
        });
        if !same_receiver {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                call.syntax(),
                "receiver projection is outside the first source slice",
            ));
        }
        let result_type = self
            .semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(call.clone()))
            .ok_or_else(|| {
                self.issue(
                    AnalysisIssueKind::TypeResolution,
                    call.syntax(),
                    "result type is unresolved",
                )
            })?;
        let result = self
            .snapshot
            .type_ref(&result_type.original)
            .map_err(|message| {
                self.issue(AnalysisIssueKind::TypeResolution, call.syntax(), &message)
            })?;
        let (_, line) = self
            .snapshot
            .location(self.semantics, call.syntax())
            .map_err(|message| self.issue(AnalysisIssueKind::Read, call.syntax(), &message))?;
        let mut translated = Call::new(
            self.expression(&receiver)?,
            target.name(&self.snapshot.db).as_str(),
            line,
        )
        .returning(result);
        let arguments = call.arg_list().ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Parse,
                call.syntax(),
                "argument list is missing",
            )
        })?;
        for argument in arguments.args() {
            translated = translated.with(self.expression(&argument)?);
        }
        Ok(translated)
    }

    fn issue(&self, kind: AnalysisIssueKind, node: &SyntaxNode, message: &str) -> AnalysisIssue {
        let location = match self.snapshot.location(self.semantics, node) {
            Ok((path, line)) => ReadError::at_line(&path, line, message),
            Err(_) => ReadError::new(self.path, message),
        };
        AnalysisIssue::new(kind, location).in_body(self.owner.clone())
    }
}

fn without_references(mut ty: Type<'_>) -> Type<'_> {
    while let Some(inner) = ty.as_reference_inner() {
        ty = inner;
    }
    ty
}
