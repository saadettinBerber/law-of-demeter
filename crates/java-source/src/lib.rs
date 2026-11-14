mod error;
mod model;
mod syntax;

pub use error::ParseError;
pub use model::{Class, Import, SourceFile};

pub fn parse(path: &str, source: &str) -> Result<SourceFile, ParseError> {
    syntax::read_file(path, source)
}
