use ra_ap_syntax::{AstNode, ast, ast::HasName};

pub(crate) fn declared_function(source: &ast::SourceFile, name: &str) -> ast::Fn {
    source
        .syntax()
        .children()
        .filter_map(ast::Fn::cast)
        .find(|function| {
            function
                .name()
                .is_some_and(|function_name| function_name.text() == name)
        })
        .expect("fixture function declaration is missing")
}

pub(crate) fn declared_struct(source: &ast::SourceFile, name: &str) -> ast::Struct {
    source
        .syntax()
        .children()
        .filter_map(ast::Struct::cast)
        .find(|item| {
            item.name()
                .is_some_and(|item_name| item_name.text() == name)
        })
        .expect("fixture struct declaration is missing")
}

pub(crate) fn declared_method(source: &ast::SourceFile, owner: &str, name: &str) -> ast::Fn {
    source
        .syntax()
        .children()
        .filter_map(ast::Impl::cast)
        .find(|implementation| {
            implementation
                .self_ty()
                .is_some_and(|ty| ty.syntax().text() == owner)
        })
        .expect("fixture impl declaration is missing")
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|method| {
            method
                .name()
                .is_some_and(|method_name| method_name.text() == name)
        })
        .expect("fixture method declaration is missing")
}

pub(crate) fn method_call(caller: &ast::Fn, name: &str) -> ast::MethodCallExpr {
    caller
        .body()
        .expect("fixture caller has no body")
        .syntax()
        .descendants()
        .filter_map(ast::MethodCallExpr::cast)
        .find(|call| {
            call.name_ref()
                .is_some_and(|method_name| method_name.text() == name)
        })
        .expect("fixture method call is missing")
}
