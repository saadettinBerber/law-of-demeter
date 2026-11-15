use tree_sitter::Node;

use crate::declarations::Declaration;
use crate::model::{
    Class, ClassKind, ClassOrigin, DeclaredType, Expression, Field, INSTANCE_INITIALIZER, Method,
    New, Parameter, STATIC_INITIALIZER, Statement,
};
use crate::reader::Reader;
use crate::scope::{Binding, Scope};
use crate::tree::{line_of, members, named_children};

impl Reader<'_> {
    pub(crate) fn fields(&self, declaration: &Declaration) -> Vec<Field> {
        let scope = Scope::for_class(declaration);
        let mut fields = self.record_components(declaration, &scope);
        let Some(body) = declaration.body else {
            return fields;
        };
        for member in members(body) {
            match member.kind() {
                "field_declaration" | "constant_declaration" => {
                    fields.extend(self.field_declaration(member, declaration, &scope));
                }
                "enum_constant" => fields.extend(self.enum_constant(member, declaration)),
                _ => {}
            }
        }
        fields
    }

    pub(crate) fn methods(
        &self,
        declaration: &Declaration,
        nested: &mut Vec<Class>,
    ) -> Vec<Method> {
        let Some(body) = declaration.body else {
            return Vec::new();
        };
        let mut methods: Vec<Method> = members(body)
            .into_iter()
            .filter_map(|member| match member.kind() {
                "method_declaration" | "constructor_declaration" => {
                    self.method(member, declaration, nested)
                }
                "compact_constructor_declaration" => {
                    self.compact_constructor(member, declaration, nested)
                }
                _ => None,
            })
            .collect();
        methods.extend(self.initializers(body, declaration, nested));
        methods
    }

    fn initializers(
        &self,
        body: Node,
        declaration: &Declaration,
        nested: &mut Vec<Class>,
    ) -> Vec<Method> {
        let mut static_part = Initializer::new(STATIC_INITIALIZER, true, declaration);
        let mut instance_part = Initializer::new(INSTANCE_INITIALIZER, false, declaration);
        for member in members(body) {
            let is_static = match member.kind() {
                "static_initializer" | "enum_constant" => true,
                "constant_declaration" => true,
                "field_declaration" => has_modifier(member, "static"),
                "block" => false,
                _ => continue,
            };
            let part = if is_static {
                &mut static_part
            } else {
                &mut instance_part
            };
            let statements = self.initialization(member, declaration, &mut part.scope);
            part.add(member, statements);
        }
        [static_part, instance_part]
            .into_iter()
            .filter_map(|part| part.into_method(nested))
            .collect()
    }

    fn initialization(
        &self,
        member: Node,
        declaration: &Declaration,
        scope: &mut Scope,
    ) -> Vec<Statement> {
        match member.kind() {
            "field_declaration" | "constant_declaration" => {
                let mut cursor = member.walk();
                let declarators: Vec<Node> = member
                    .children_by_field_name("declarator", &mut cursor)
                    .collect();
                declarators
                    .into_iter()
                    .filter_map(|declarator| {
                        let name = declarator.child_by_field_name("name")?;
                        let value = declarator.child_by_field_name("value")?;
                        Some(
                            self.initialization_of(
                                self.text.of(name),
                                self.expression(value, scope),
                            ),
                        )
                    })
                    .collect()
            }
            "enum_constant" => {
                let Some(name) = member.child_by_field_name("name") else {
                    return Vec::new();
                };
                let arguments = member.child_by_field_name("arguments");
                if arguments.is_none() && member.child_by_field_name("body").is_none() {
                    return Vec::new();
                }
                let arguments = arguments
                    .map(|arguments| self.operands(arguments, scope))
                    .unwrap_or_default();
                let created = Expression::New(New {
                    created: DeclaredType::class(&declaration.name),
                    arguments,
                    anonymous_class: self.read_nested(
                        member,
                        ClassKind::Class,
                        ClassOrigin::Anonymous,
                        scope,
                    ),
                    line: line_of(member),
                });
                vec![self.initialization_of(self.text.of(name), created)]
            }
            _ => {
                let block = if member.kind() == "block" {
                    Some(member)
                } else {
                    named_children(member)
                        .into_iter()
                        .find(|child| child.kind() == "block")
                };
                block
                    .map(|block| vec![Statement::Block(self.block_statements(block, scope))])
                    .unwrap_or_default()
            }
        }
    }

    fn initialization_of(&self, field: &str, value: Expression) -> Statement {
        Statement::Expression(Expression::Assignment {
            target: Box::new(Expression::Field(field.to_owned())),
            value: Box::new(value),
        })
    }

    fn record_components(&self, declaration: &Declaration, scope: &Scope) -> Vec<Field> {
        self.record_parameters(declaration, scope)
            .into_iter()
            .map(|parameter| Field {
                name: parameter.name,
                declared_type: parameter.declared_type,
                is_static: false,
            })
            .collect()
    }

    fn record_parameters(&self, declaration: &Declaration, scope: &Scope) -> Vec<Parameter> {
        declaration
            .node
            .child_by_field_name("parameters")
            .map(|components| self.parameters(components, scope))
            .unwrap_or_default()
    }

    fn field_declaration(
        &self,
        node: Node,
        declaration: &Declaration,
        scope: &Scope,
    ) -> Vec<Field> {
        let Some(declared) = node.child_by_field_name("type") else {
            return Vec::new();
        };
        let is_static = has_modifier(node, "static") || declaration.kind == ClassKind::Interface;
        let mut cursor = node.walk();
        node.children_by_field_name("declarator", &mut cursor)
            .filter_map(|declarator| {
                let name = declarator.child_by_field_name("name")?;
                Some(Field {
                    name: self.text.of(name).to_owned(),
                    declared_type: self.with_dimensions(
                        self.declared_type(declared, scope),
                        declarator.child_by_field_name("dimensions"),
                    ),
                    is_static,
                })
            })
            .collect()
    }

    fn enum_constant(&self, node: Node, declaration: &Declaration) -> Option<Field> {
        let name = node.child_by_field_name("name")?;
        Some(Field {
            name: self.text.of(name).to_owned(),
            declared_type: DeclaredType::class(&declaration.name),
            is_static: true,
        })
    }

    fn method(
        &self,
        node: Node,
        declaration: &Declaration,
        nested: &mut Vec<Class>,
    ) -> Option<Method> {
        let name = node.child_by_field_name("name")?;
        let mut scope =
            Scope::for_class(declaration).with_type_variables(self.text.type_parameters(node));
        let type_parameters = self.type_parameters(node, &scope);
        let returns = node
            .child_by_field_name("type")
            .map(|declared| self.declared_type(declared, &scope));
        let parameters = node
            .child_by_field_name("parameters")
            .map(|parameters| self.parameters(parameters, &scope))
            .unwrap_or_default();
        let body = self.body(node, &parameters, &mut scope);
        nested.extend(scope.take_nested());
        Some(Method {
            name: self.text.of(name).to_owned(),
            line: line_of(name),
            is_static: has_modifier(node, "static"),
            type_parameters,
            returns,
            body,
            parameters,
        })
    }

    fn compact_constructor(
        &self,
        node: Node,
        declaration: &Declaration,
        nested: &mut Vec<Class>,
    ) -> Option<Method> {
        let name = node.child_by_field_name("name")?;
        let mut scope = Scope::for_class(declaration);
        let parameters = self.record_parameters(declaration, &scope);
        let body = self.body(node, &parameters, &mut scope);
        nested.extend(scope.take_nested());
        Some(Method {
            name: self.text.of(name).to_owned(),
            line: line_of(name),
            is_static: false,
            type_parameters: Vec::new(),
            returns: None,
            body,
            parameters,
        })
    }

    fn body(
        &self,
        method: Node,
        parameters: &[Parameter],
        scope: &mut Scope,
    ) -> Option<Vec<Statement>> {
        let body = method.child_by_field_name("body")?;
        for parameter in parameters {
            scope.declare(
                &parameter.name,
                Binding::Parameter,
                parameter.declared_type.clone(),
            );
        }
        Some(self.block_statements(body, scope))
    }

    fn parameters(&self, parameters: Node, scope: &Scope) -> Vec<Parameter> {
        let mut cursor = parameters.walk();
        parameters
            .named_children(&mut cursor)
            .filter_map(|parameter| match parameter.kind() {
                "formal_parameter" => self.formal_parameter(parameter, scope),
                "spread_parameter" => self.spread_parameter(parameter, scope),
                _ => None,
            })
            .collect()
    }

    fn formal_parameter(&self, node: Node, scope: &Scope) -> Option<Parameter> {
        let name = node.child_by_field_name("name")?;
        let declared = node.child_by_field_name("type")?;
        Some(Parameter {
            name: self.text.of(name).to_owned(),
            declared_type: self.with_dimensions(
                self.declared_type(declared, scope),
                node.child_by_field_name("dimensions"),
            ),
        })
    }

    fn spread_parameter(&self, node: Node, scope: &Scope) -> Option<Parameter> {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.named_children(&mut cursor).collect();
        let declared = children
            .iter()
            .find(|child| !matches!(child.kind(), "modifiers" | "variable_declarator"))?;
        let name = children
            .iter()
            .find(|child| child.kind() == "variable_declarator")?
            .child_by_field_name("name")?;
        Some(Parameter {
            name: self.text.of(name).to_owned(),
            declared_type: DeclaredType::array(self.declared_type(*declared, scope)),
        })
    }
}

fn has_modifier(node: Node, modifier: &str) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| child.kind() == "modifiers")
        .any(|modifiers| {
            let mut cursor = modifiers.walk();
            modifiers
                .children(&mut cursor)
                .any(|child| child.kind() == modifier)
        })
}

struct Initializer {
    name: &'static str,
    is_static: bool,
    scope: Scope,
    line: Option<usize>,
    statements: Vec<Statement>,
}

impl Initializer {
    fn new(name: &'static str, is_static: bool, declaration: &Declaration) -> Self {
        Self {
            name,
            is_static,
            scope: Scope::for_class(declaration),
            line: None,
            statements: Vec::new(),
        }
    }

    fn add(&mut self, member: Node, statements: Vec<Statement>) {
        if statements.is_empty() {
            return;
        }
        self.line.get_or_insert(line_of(member));
        self.statements.extend(statements);
    }

    fn into_method(mut self, nested: &mut Vec<Class>) -> Option<Method> {
        nested.extend(self.scope.take_nested());
        Some(Method {
            name: self.name.to_owned(),
            line: self.line?,
            is_static: self.is_static,
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            returns: Some(DeclaredType::primitive("void")),
            body: Some(self.statements),
        })
    }
}
