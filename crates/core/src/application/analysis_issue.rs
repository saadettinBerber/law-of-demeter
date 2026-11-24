use std::path::Path;

use super::{ReadError, Target};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisIssueKind {
    Read,
    Parse,
    TypeResolution,
    TargetResolution,
    Origin,
    Expansion,
    Unsupported,
    Compilation,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisIssue {
    kind: AnalysisIssueKind,
    location: ReadError,
    owner: Option<Target>,
}

impl AnalysisIssue {
    pub fn new(kind: AnalysisIssueKind, location: ReadError) -> Self {
        Self {
            kind,
            location,
            owner: None,
        }
    }

    pub fn in_body(mut self, owner: Target) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn kind(&self) -> AnalysisIssueKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        self.location.path()
    }

    pub fn line(&self) -> Option<usize> {
        self.location.line()
    }

    pub fn message(&self) -> &str {
        self.location.message()
    }

    pub fn owner(&self) -> Option<&Target> {
        self.owner.as_ref()
    }
}
