use std::path::{Path, PathBuf};

use super::ports::ReadError;
use super::selection::Target;
use crate::domain::classification::HybridClass;
use crate::domain::method::{Reason, Verdict};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    path: PathBuf,
    class_name: String,
    method_name: String,
    verdict: Verdict,
}

impl Finding {
    pub fn new(path: &Path, class_name: &str, method_name: &str, verdict: Verdict) -> Self {
        Self {
            path: path.to_path_buf(),
            class_name: class_name.to_owned(),
            method_name: method_name.to_owned(),
            verdict,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    pub fn verdict(&self) -> &Verdict {
        &self.verdict
    }

    pub fn line(&self) -> usize {
        match &self.verdict {
            Verdict::Allowed { line, .. } | Verdict::Violation { line, .. } => *line,
        }
    }

    pub fn call(&self) -> &str {
        match &self.verdict {
            Verdict::Allowed { call, .. } | Verdict::Violation { call, .. } => call,
        }
    }

    pub fn is_violation(&self) -> bool {
        matches!(self.verdict, Verdict::Violation { .. })
    }

    pub fn is_unresolved(&self) -> bool {
        self.is_violation() && self.reason().is_unresolved()
    }

    pub fn reason(&self) -> &Reason {
        match &self.verdict {
            Verdict::Allowed { reason, .. } | Verdict::Violation { reason, .. } => reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridFinding {
    path: PathBuf,
    hybrid: HybridClass,
}

impl HybridFinding {
    pub fn new(path: &Path, hybrid: HybridClass) -> Self {
        Self {
            path: path.to_path_buf(),
            hybrid,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn hybrid(&self) -> &HybridClass {
        &self.hybrid
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Counts {
    pub files: usize,
    pub methods: usize,
    pub calls: usize,
    pub allowed: usize,
    pub violations: usize,
    pub unresolved: usize,
    pub hybrid_classes: usize,
    pub source_errors: usize,
    pub type_errors: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    pub(super) files: usize,
    pub(super) methods: usize,
    pub(super) findings: Vec<Finding>,
    pub(super) hybrid_classes: Vec<HybridFinding>,
    pub(super) source_errors: Vec<ReadError>,
    pub(super) type_errors: Vec<ReadError>,
    pub(super) unmatched_targets: Vec<Target>,
}

impl Report {
    pub fn new(
        files: usize,
        methods: usize,
        findings: Vec<Finding>,
        hybrid_classes: Vec<HybridFinding>,
        source_errors: Vec<ReadError>,
        type_errors: Vec<ReadError>,
    ) -> Self {
        Self {
            files,
            methods,
            findings,
            hybrid_classes,
            source_errors,
            type_errors,
            unmatched_targets: Vec::new(),
        }
    }

    pub fn counts(&self) -> Counts {
        let unresolved = self.findings.iter().filter(|f| f.is_unresolved()).count();
        let violations = self.findings.iter().filter(|f| f.is_violation()).count();
        Counts {
            files: self.files,
            methods: self.methods,
            calls: self.findings.len(),
            allowed: self.findings.len() - violations,
            violations: violations - unresolved,
            unresolved,
            hybrid_classes: self.hybrid_classes.len(),
            source_errors: self.source_errors.len(),
            type_errors: self.type_errors.len(),
        }
    }

    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    pub fn hybrid_classes(&self) -> &[HybridFinding] {
        &self.hybrid_classes
    }

    pub fn source_errors(&self) -> &[ReadError] {
        &self.source_errors
    }

    pub fn type_errors(&self) -> &[ReadError] {
        &self.type_errors
    }

    pub fn unmatched_targets(&self) -> &[Target] {
        &self.unmatched_targets
    }
}
