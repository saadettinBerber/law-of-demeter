use law_of_demeter_core::application::ReadError;
use law_of_demeter_core::domain::type_shape::{FieldShape, MethodShape, TypeKind, TypeShape};
use ra_ap_hir::{Adt, AssocItem, Impl, ModuleDef, Semantics};

use crate::snapshot::Snapshot;

pub(super) fn read(snapshot: &Snapshot) -> (Vec<TypeShape>, Vec<ReadError>, Vec<String>) {
    let semantics = Semantics::new(&snapshot.db);
    let mut shapes = Vec::new();
    let mut errors = Vec::new();
    let mut names = Vec::new();
    for krate in &snapshot.project_crates {
        for module in krate.modules(&snapshot.db) {
            for declaration in module.declarations(&snapshot.db) {
                let ModuleDef::Adt(adt) = declaration else {
                    continue;
                };
                let name = snapshot.identity(adt);
                names.push(name.clone());
                let Adt::Struct(structure) = adt else {
                    let error = semantics.source(adt).and_then(|source| {
                        snapshot
                            .location(&semantics, ra_ap_syntax::AstNode::syntax(&source.value))
                            .ok()
                    });
                    errors.push(match error {
                        Some((path, line)) => ReadError::at_line(
                            &path,
                            line,
                            "aggregate kind is outside the first source slice",
                        ),
                        None => ReadError::new(
                            &snapshot.source_root,
                            "aggregate declaration is unavailable",
                        ),
                    });
                    continue;
                };
                let result = (|| {
                    let mut shape = TypeShape::new(&name, TypeKind::Class);
                    for field in structure.fields(&snapshot.db) {
                        let field_type = snapshot.type_ref(&field.ty(&snapshot.db))?;
                        shape = shape.with_field(FieldShape::new(
                            field.name(&snapshot.db).as_str(),
                            field_type,
                        ));
                    }
                    for implementation in Impl::all_in_crate(&snapshot.db, *krate) {
                        if implementation.self_ty(&snapshot.db).as_adt()
                            != Some(Adt::Struct(structure))
                        {
                            continue;
                        }
                        for item in implementation.items(&snapshot.db) {
                            let AssocItem::Function(function) = item else {
                                continue;
                            };
                            let parameters = function
                                .params_without_self(&snapshot.db)
                                .into_iter()
                                .map(|parameter| snapshot.type_ref(parameter.ty()))
                                .collect::<Result<Vec<_>, _>>()?;
                            let mut method = MethodShape::new(function.name(&snapshot.db).as_str())
                                .with_parameters(parameters);
                            if function.self_param(&snapshot.db).is_none() {
                                method = method.as_static();
                            }
                            let returns = function.ret_type(&snapshot.db);
                            if !returns.is_unit() {
                                method = method.returning(snapshot.type_ref(&returns)?);
                            }
                            shape = shape.with_method(method);
                        }
                    }
                    Ok::<_, String>(shape)
                })();
                match result {
                    Ok(shape) => shapes.push(shape),
                    Err(error) => {
                        if let Some(source) = semantics.source(structure)
                            && let Ok((path, line)) = snapshot
                                .location(&semantics, ra_ap_syntax::AstNode::syntax(&source.value))
                        {
                            errors.push(ReadError::at_line(&path, line, &error));
                        } else {
                            errors.push(ReadError::new(&snapshot.source_root, &error));
                        }
                    }
                }
            }
        }
    }
    (shapes, errors, names)
}
