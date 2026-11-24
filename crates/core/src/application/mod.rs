mod analysis_issue;
mod check_project;
mod ports;
mod report;
mod selection;

pub use analysis_issue::{AnalysisIssue, AnalysisIssueKind};
pub use check_project::CheckProject;
pub use ports::{ReadError, SourceFile, SourceReader, TypeReading, TypeSource};
pub use report::{Counts, Finding, HybridFinding, Report};
pub use selection::{Selection, Target};

#[cfg(test)]
mod tests;
