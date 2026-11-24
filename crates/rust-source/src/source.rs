use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use law_of_demeter_core::application::{
    AnalysisIssue, AnalysisIssueKind, ReadError, SourceFile, Target,
};
use law_of_demeter_core::domain::method::{Call, Creation, Expression, Method};
use ra_ap_hir::{
    Adt, AsAssocItem, AssocItem, CallableKind, Function, Impl, Local, ModuleDef, PathResolution,
    Semantics, Type, Variant,
};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast, ast::HasArgList};

use crate::snapshot::Snapshot;

pub(super) fn read(
    snapshot: &Snapshot,
) -> (Vec<Result<SourceFile, ReadError>>, Vec<AnalysisIssue>) {
    ra_ap_hir::attach_db(&snapshot.db, || read_attached(snapshot))
}

fn read_attached(snapshot: &Snapshot) -> (Vec<Result<SourceFile, ReadError>>, Vec<AnalysisIssue>) {
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
                locals: HashMap::new(),
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
    locals: HashMap<Local<'db>, LocalBinding>,
}

struct LocalBinding {
    name: String,
    is_current_instance: bool,
}

impl<'db> BodyReader<'_, 'db> {
    fn read(mut self, function: &ast::Fn, line: usize) -> Result<Method, AnalysisIssue> {
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
            match statement {
                ast::Stmt::LetStmt(statement) => method = self.bind(method, &statement)?,
                ast::Stmt::ExprStmt(statement) => {
                    let expression = statement.expr().ok_or_else(|| {
                        self.issue(
                            AnalysisIssueKind::Parse,
                            statement.syntax(),
                            "statement expression is missing",
                        )
                    })?;
                    method = method.evaluate(self.expression(&expression)?);
                }
                _ => {
                    return Err(self.issue(
                        AnalysisIssueKind::Unsupported,
                        statement.syntax(),
                        "statement is outside the supported scope",
                    ));
                }
            }
        }
        if let Some(expression) = body.tail_expr() {
            method = method.give_back(self.expression(&expression)?);
        }
        Ok(method)
    }

    fn bind(&mut self, method: Method, statement: &ast::LetStmt) -> Result<Method, AnalysisIssue> {
        if statement.let_else().is_some() {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                statement.syntax(),
                "let else is outside the supported scope",
            ));
        }
        let Some(ast::Pat::IdentPat(pattern)) = statement.pat() else {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                statement.syntax(),
                "binding pattern is outside the supported scope",
            ));
        };
        if pattern.ref_token().is_some() || pattern.pat().is_some() {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                statement.syntax(),
                "binding pattern is outside the supported scope",
            ));
        }
        let local = self.semantics.to_def(&pattern).ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Origin,
                statement.syntax(),
                "local binding is unresolved",
            )
        })?;
        let name = local.name(&self.snapshot.db).as_str().to_owned();
        let shadows_parameter = self
            .function
            .params_without_self(&self.snapshot.db)
            .iter()
            .any(|parameter| {
                parameter
                    .name(&self.snapshot.db)
                    .is_some_and(|parameter| parameter.as_str() == name)
            });
        if shadows_parameter || self.locals.values().any(|binding| binding.name == name) {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                statement.syntax(),
                "shadowed bindings are outside the supported scope",
            ));
        }
        let initializer = statement.initializer().ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Unsupported,
                statement.syntax(),
                "uninitialized bindings are outside the supported scope",
            )
        })?;
        self.same_value(&initializer)?;
        let value = self.expression(&initializer)?;
        let binding_type = self
            .semantics
            .type_of_binding_in_pat(&pattern)
            .ok_or_else(|| {
                self.issue(
                    AnalysisIssueKind::TypeResolution,
                    statement.syntax(),
                    "binding type is unresolved",
                )
            })?;
        let declared = self.snapshot.type_ref(&binding_type).map_err(|message| {
            self.issue(
                AnalysisIssueKind::TypeResolution,
                statement.syntax(),
                &message,
            )
        })?;
        let is_current_instance = self.is_current_instance(&value);
        self.locals.insert(
            local,
            LocalBinding {
                name: name.clone(),
                is_current_instance,
            },
        );
        Ok(method.assign(&name, declared, value))
    }

    fn expression(&self, expression: &ast::Expr) -> Result<Expression, AnalysisIssue> {
        match expression {
            ast::Expr::MethodCallExpr(call) => self.call(call).map(Expression::return_of),
            ast::Expr::FieldExpr(field) => self.field(field),
            ast::Expr::RecordExpr(record) => self.record(record),
            ast::Expr::CallExpr(call) => self.tuple(call),
            ast::Expr::RefExpr(reference) => self.reference(reference),
            ast::Expr::PrefixExpr(prefix) => self.dereference(prefix),
            ast::Expr::ParenExpr(parenthesized) => {
                self.same_value(expression)?;
                let value = parenthesized.expr().ok_or_else(|| {
                    self.issue(
                        AnalysisIssueKind::Parse,
                        expression.syntax(),
                        "parenthesized value is missing",
                    )
                })?;
                self.expression(&value)
            }
            ast::Expr::Literal(_) => Ok(Expression::literal()),
            ast::Expr::PathExpr(path) => {
                self.same_value(expression)?;
                if let Some(PathResolution::Local(local)) = path
                    .path()
                    .and_then(|path| self.semantics.resolve_path(&path))
                {
                    return self.local(local, expression.syntax());
                }
                Err(self.issue(
                    AnalysisIssueKind::Origin,
                    expression.syntax(),
                    "expression does not resolve to a supported binding",
                ))
            }
            _ => Err(self.issue(
                AnalysisIssueKind::Unsupported,
                expression.syntax(),
                "expression is outside the supported scope",
            )),
        }
    }

    fn reference(&self, reference: &ast::RefExpr) -> Result<Expression, AnalysisIssue> {
        if reference.raw_token().is_some() {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                reference.syntax(),
                "raw references are outside the supported scope",
            ));
        }
        self.same_value(&ast::Expr::RefExpr(reference.clone()))?;
        let value = reference.expr().ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Parse,
                reference.syntax(),
                "borrowed value is missing",
            )
        })?;
        self.expression(&value)
    }

    fn dereference(&self, prefix: &ast::PrefixExpr) -> Result<Expression, AnalysisIssue> {
        let value = prefix.expr().ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Parse,
                prefix.syntax(),
                "dereferenced value is missing",
            )
        })?;
        let info = self.semantics.type_of_expr(&value).ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::TypeResolution,
                value.syntax(),
                "dereferenced value type is unresolved",
            )
        })?;
        if prefix.op_kind() != Some(ast::UnaryOp::Deref)
            || info.original.as_reference_inner().is_none()
        {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                prefix.syntax(),
                "only normal reference dereference is supported",
            ));
        }
        self.same_value(&ast::Expr::PrefixExpr(prefix.clone()))?;
        self.expression(&value)
    }

    fn local(&self, local: Local<'db>, node: &SyntaxNode) -> Result<Expression, AnalysisIssue> {
        if local.is_self(&self.snapshot.db) {
            return Ok(Expression::This);
        }
        let parameters = self.function.params_without_self(&self.snapshot.db);
        if let Some(parameter) = parameters
            .iter()
            .find(|parameter| parameter.as_local(&self.snapshot.db) == Some(local))
        {
            let ty = self
                .snapshot
                .type_ref(parameter.ty())
                .map_err(|message| self.issue(AnalysisIssueKind::TypeResolution, node, &message))?;
            return Ok(Expression::parameter(
                local.name(&self.snapshot.db).as_str(),
                ty,
            ));
        }
        if let Some(binding) = self.locals.get(&local) {
            return Ok(Expression::local(&binding.name));
        }
        Err(self.issue(
            AnalysisIssueKind::Origin,
            node,
            "local origin is unavailable",
        ))
    }

    fn is_current_instance(&self, value: &Expression) -> bool {
        match value {
            Expression::This => true,
            Expression::Local(name) => self
                .locals
                .values()
                .any(|binding| binding.name == *name && binding.is_current_instance),
            _ => false,
        }
    }

    fn same_value(&self, value: &ast::Expr) -> Result<(), AnalysisIssue> {
        let info = self.semantics.type_of_expr(value).ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::TypeResolution,
                value.syntax(),
                "value type is unresolved",
            )
        })?;
        if let Some(adjusted) = info.adjusted {
            let same = ra_ap_hir::attach_db(&self.snapshot.db, || {
                without_references(info.original) == without_references(adjusted)
            });
            if !same {
                return Err(self.issue(
                    AnalysisIssueKind::Unsupported,
                    value.syntax(),
                    "value projection is outside the supported scope",
                ));
            }
        }
        Ok(())
    }

    fn record(&self, record: &ast::RecordExpr) -> Result<Expression, AnalysisIssue> {
        let expression = ast::Expr::RecordExpr(record.clone());
        let info = self.semantics.type_of_expr(&expression).ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::TypeResolution,
                record.syntax(),
                "creation type is unresolved",
            )
        })?;
        let Some(Adt::Struct(structure)) = info.original.as_adt() else {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                record.syntax(),
                "only struct record creation is supported",
            ));
        };
        let type_ref = self.snapshot.type_ref(&info.original).map_err(|message| {
            self.issue(AnalysisIssueKind::TypeResolution, record.syntax(), &message)
        })?;
        let fields = record.record_expr_field_list().ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Parse,
                record.syntax(),
                "creation fields are missing",
            )
        })?;
        if fields.spread().is_some() {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                record.syntax(),
                "struct update is outside the supported scope",
            ));
        }
        let mut creation = Creation::new(type_ref);
        for field in fields.fields() {
            let (resolved, shorthand, _) =
                self.semantics.resolve_record_field(&field).ok_or_else(|| {
                    self.issue(
                        AnalysisIssueKind::TargetResolution,
                        field.syntax(),
                        "initializer field is unresolved",
                    )
                })?;
            if resolved.parent_def(&self.snapshot.db) != Variant::Struct(structure) {
                return Err(self.issue(
                    AnalysisIssueKind::Unsupported,
                    field.syntax(),
                    "initializer field has a different owner",
                ));
            }
            let value = if let Some(value) = field.expr() {
                self.same_value(&value)?;
                self.expression(&value)?
            } else if let Some(local) = shorthand {
                self.local(local, field.syntax())?
            } else {
                return Err(self.issue(
                    AnalysisIssueKind::Origin,
                    field.syntax(),
                    "initializer origin is unavailable",
                ));
            };
            creation = creation.with(value);
        }
        Ok(Expression::created(creation))
    }

    fn tuple(&self, call: &ast::CallExpr) -> Result<Expression, AnalysisIssue> {
        let target = call
            .expr()
            .and_then(|callee| self.semantics.resolve_expr_as_callable(&callee))
            .ok_or_else(|| {
                self.issue(
                    AnalysisIssueKind::TargetResolution,
                    call.syntax(),
                    "call target is unresolved",
                )
            })?;
        let CallableKind::TupleStruct(structure) = target.kind() else {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                call.syntax(),
                "only tuple struct constructor calls are supported",
            ));
        };
        let info = self
            .semantics
            .type_of_expr(&ast::Expr::CallExpr(call.clone()))
            .ok_or_else(|| {
                self.issue(
                    AnalysisIssueKind::TypeResolution,
                    call.syntax(),
                    "creation result type is unresolved",
                )
            })?;
        if info.original.as_adt() != Some(Adt::Struct(structure)) {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                call.syntax(),
                "constructor result has a different type",
            ));
        }
        let type_ref = self.snapshot.type_ref(&info.original).map_err(|message| {
            self.issue(AnalysisIssueKind::TypeResolution, call.syntax(), &message)
        })?;
        let mut creation = Creation::new(type_ref);
        let arguments = call.arg_list().ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Parse,
                call.syntax(),
                "constructor arguments are missing",
            )
        })?;
        for argument in arguments.args() {
            self.same_value(&argument)?;
            creation = creation.with(self.expression(&argument)?);
        }
        Ok(Expression::created(creation))
    }

    fn field(&self, field: &ast::FieldExpr) -> Result<Expression, AnalysisIssue> {
        let resolved = self
            .semantics
            .resolve_field(field)
            .and_then(|field| field.left())
            .ok_or_else(|| {
                self.issue(
                    AnalysisIssueKind::TargetResolution,
                    field.syntax(),
                    "field target is unresolved",
                )
            })?;
        let receiver = field.expr().ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::Parse,
                field.syntax(),
                "field receiver is missing",
            )
        })?;
        let receiver_type = self.semantics.type_of_expr(&receiver).ok_or_else(|| {
            self.issue(
                AnalysisIssueKind::TypeResolution,
                receiver.syntax(),
                "field receiver type is unresolved",
            )
        })?;
        let Variant::Struct(owner) = resolved.parent_def(&self.snapshot.db) else {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                field.syntax(),
                "field owner is outside the supported scope",
            ));
        };
        if without_references(receiver_type.original).as_adt() != Some(Adt::Struct(owner)) {
            return Err(self.issue(
                AnalysisIssueKind::Unsupported,
                field.syntax(),
                "field receiver projection is outside the supported scope",
            ));
        }
        let receiver = self.expression(&receiver)?;
        let name = resolved.name(&self.snapshot.db);
        if self.is_current_instance(&receiver) {
            let actual = self
                .semantics
                .type_of_expr(&ast::Expr::FieldExpr(field.clone()))
                .ok_or_else(|| {
                    self.issue(
                        AnalysisIssueKind::TypeResolution,
                        field.syntax(),
                        "field type is unresolved",
                    )
                })?;
            let type_ref = self
                .snapshot
                .type_ref(&actual.original)
                .map_err(|message| {
                    self.issue(AnalysisIssueKind::TypeResolution, field.syntax(), &message)
                })?;
            Ok(Expression::field(name.as_str(), type_ref))
        } else {
            Ok(Expression::field_of(receiver, name.as_str()))
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
            self.same_value(&argument)?;
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
