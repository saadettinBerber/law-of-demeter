mod catalog;
mod diagnostics;
mod snapshot;
mod source;

use std::path::Path;

use law_of_demeter_core::application::{
    AnalysisIssue, ReadError, SourceFile, SourceReader, TypeReading, TypeSource,
};
use law_of_demeter_core::domain::classification::{Platform, Types};
use law_of_demeter_core::domain::type_shape::TypeShape;

pub use snapshot::LoadOptions;
use snapshot::Snapshot;

pub struct RustProject {
    _snapshot: Snapshot,
    shapes: Vec<TypeShape>,
    files: Vec<Result<SourceFile, ReadError>>,
    issues: Vec<AnalysisIssue>,
    type_errors: Vec<ReadError>,
    project_types: Vec<String>,
}

impl RustProject {
    pub fn load(manifest: &Path, options: LoadOptions) -> Result<Self, ReadError> {
        let snapshot = Snapshot::load(manifest, options)
            .map_err(|error| ReadError::new(manifest, &error.to_string()))?;
        let (shapes, type_errors, project_types) = catalog::read(&snapshot);
        let (files, issues) = source::read(&snapshot);
        Ok(Self {
            _snapshot: snapshot,
            shapes,
            files,
            issues,
            type_errors,
            project_types,
        })
    }
}

impl SourceReader for RustProject {
    fn read(&self, _types: &Types) -> Vec<Result<SourceFile, ReadError>> {
        self.files.clone()
    }

    fn issues(&self) -> Vec<AnalysisIssue> {
        self.issues.clone()
    }
}

impl TypeSource for RustProject {
    fn read(&self) -> TypeReading {
        TypeReading {
            shapes: self.shapes.clone(),
            errors: self.type_errors.clone(),
            platform: Platform::new(),
        }
    }

    fn project_types(&self) -> Vec<String> {
        self.project_types.clone()
    }
}

#[cfg(test)]
mod tests;
