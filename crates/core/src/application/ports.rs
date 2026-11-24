use std::path::{Path, PathBuf};

use super::AnalysisIssue;
use crate::domain::classification::{Platform, Types};
use crate::domain::method::Method;
use crate::domain::type_shape::TypeShape;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadError {
    path: PathBuf,
    line: Option<usize>,
    message: String,
}

impl ReadError {
    pub fn new(path: &Path, message: &str) -> Self {
        Self {
            path: path.to_path_buf(),
            line: None,
            message: message.to_owned(),
        }
    }

    pub fn at_line(path: &Path, line: usize, message: &str) -> Self {
        Self {
            line: Some(line),
            ..Self::new(path, message)
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn line(&self) -> Option<usize> {
        self.line
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    path: PathBuf,
    classes: Vec<String>,
    methods: Vec<Method>,
}

impl SourceFile {
    pub fn new(path: &Path, classes: Vec<String>, methods: Vec<Method>) -> Self {
        Self {
            path: path.to_path_buf(),
            classes,
            methods,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    pub fn methods(&self) -> &[Method] {
        &self.methods
    }
}

pub trait SourceReader {
    fn read(&self, types: &Types) -> Vec<Result<SourceFile, ReadError>>;

    fn issues(&self) -> Vec<AnalysisIssue> {
        Vec::new()
    }
}

#[derive(Debug, Default)]
pub struct TypeReading {
    pub shapes: Vec<TypeShape>,
    pub errors: Vec<ReadError>,
    pub platform: Platform,
}

pub trait TypeSource {
    fn read(&self) -> TypeReading;

    fn project_types(&self) -> Vec<String> {
        Vec::new()
    }
}
