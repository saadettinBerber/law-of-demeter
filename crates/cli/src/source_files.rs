use std::fs;
use std::path::{Path, PathBuf};

use law_of_demeter_core::application::{ReadError, SourceFile, SourceReader};
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::Method;

pub struct Declarations {
    pub classes: Vec<String>,
    pub methods: Vec<Method>,
}

pub type Mapper = dyn Fn(&Path, &str, &Types) -> Result<Declarations, ReadError>;

pub struct SourceFiles<'a> {
    files: Vec<PathBuf>,
    errors: Vec<ReadError>,
    mapper: &'a Mapper,
}

impl<'a> SourceFiles<'a> {
    pub fn new(files: Vec<PathBuf>, mapper: &'a Mapper) -> Self {
        Self {
            files,
            errors: Vec::new(),
            mapper,
        }
    }

    pub fn with_errors(mut self, errors: Vec<ReadError>) -> Self {
        self.errors = errors;
        self
    }
}

impl SourceReader for SourceFiles<'_> {
    fn read(&self, types: &Types) -> Vec<Result<SourceFile, ReadError>> {
        let errors = self.errors.iter().cloned().map(Err);
        let files = self.files.iter().map(|path| self.read_file(path, types));
        errors.chain(files).collect()
    }
}

impl SourceFiles<'_> {
    fn read_file(&self, path: &Path, types: &Types) -> Result<SourceFile, ReadError> {
        let text =
            fs::read_to_string(path).map_err(|error| ReadError::new(path, &error.to_string()))?;
        let declarations = (self.mapper)(path, &text, types)?;
        Ok(SourceFile::new(
            path,
            declarations.classes,
            declarations.methods,
        ))
    }
}

#[cfg(test)]
mod tests;
