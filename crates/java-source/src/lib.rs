mod declared_types;
mod error;
mod members;
mod model;
mod reader;
mod resolver;
mod scope;
mod tree;

pub use error::ParseError;
pub use model::{
    Call, Catch, Class, ClassKind, DeclaredType, Expression, Field, Import, Lambda,
    LambdaParameter, LocalVariable, Method, New, Parameter, SourceFile, Statement, Switch,
    SwitchBranch, TypeName,
};

pub fn parse(path: &str, source: &str) -> Result<SourceFile, ParseError> {
    parse_with(path, source, &|_| false)
}

pub fn parse_with(
    path: &str,
    source: &str,
    type_exists: &dyn Fn(&str) -> bool,
) -> Result<SourceFile, ParseError> {
    reader::read_file(path, source, type_exists)
}
