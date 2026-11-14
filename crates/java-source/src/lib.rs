mod error;
mod model;
mod reader;
mod resolver;
mod tree;

pub use error::ParseError;
pub use model::{Class, ClassKind, DeclaredType, Field, Import, SourceFile, TypeName};

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
