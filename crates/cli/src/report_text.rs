use law_of_demeter_core::application::{Finding, HybridFinding, ReadError, Report};

use std::collections::BTreeMap;

use crate::reason_text::{category, describe};

pub fn render(report: &Report, show_allowed: bool) -> String {
    let findings = report.findings();
    let finding_lines = |keep: fn(&Finding) -> bool, verdict: &str| -> Vec<String> {
        findings
            .iter()
            .filter(|finding| keep(finding))
            .map(|finding| finding_line(finding, verdict))
            .collect()
    };
    let mut text = String::new();
    let has_violation = findings
        .iter()
        .any(|finding| finding.is_violation() && !finding.is_unresolved());
    if !has_violation && report.hybrid_classes().is_empty() {
        text.push_str("No violations found.\n\n");
    }
    section(
        &mut text,
        "Violations",
        finding_lines(
            |finding| finding.is_violation() && !finding.is_unresolved(),
            "Violation",
        ),
    );
    section(
        &mut text,
        "Unresolved types",
        finding_lines(Finding::is_unresolved, "Unresolved"),
    );
    if show_allowed {
        section(
            &mut text,
            "Allowed calls",
            finding_lines(|finding| !finding.is_violation(), "Allowed"),
        );
    }
    section(
        &mut text,
        "Hybrid classes",
        report.hybrid_classes().iter().map(hybrid_line).collect(),
    );
    section(
        &mut text,
        "Source errors",
        report.source_errors().iter().map(error_line).collect(),
    );
    section(
        &mut text,
        "Type reading errors",
        report.type_errors().iter().map(error_line).collect(),
    );
    text.push_str(&summary(report, show_allowed));
    text
}

fn summary(report: &Report, show_allowed: bool) -> String {
    let counts = report.counts();
    let mut text = String::from("Summary:\n");
    for line in [
        format!("Files: {}", counts.files),
        format!("Methods: {}", counts.methods),
        format!(
            "Calls: {} (allowed {}, violations {}, unresolved {})",
            counts.calls, counts.allowed, counts.violations, counts.unresolved
        ),
        format!("Hybrid classes: {}", counts.hybrid_classes),
        format!("Source errors: {}", counts.source_errors),
        format!("Type reading errors: {}", counts.type_errors),
    ] {
        text.push_str(&format!("  {line}\n"));
    }
    let reasons = reason_counts(report.findings(), |finding| {
        finding.is_violation() && !finding.is_unresolved()
    });
    if !reasons.is_empty() {
        text.push_str("\nReasons for violations:\n");
        for (reason, count) in reasons {
            text.push_str(&format!("  {count}  {reason}\n"));
        }
    }
    let allowed = reason_counts(report.findings(), |finding| !finding.is_violation());
    if show_allowed && !allowed.is_empty() {
        text.push_str("\nReasons for allowed calls:\n");
        for (reason, count) in allowed {
            text.push_str(&format!("  {count}  {reason}\n"));
        }
    }
    text
}

fn reason_counts(
    findings: &[Finding],
    keep: impl Fn(&Finding) -> bool,
) -> Vec<(&'static str, usize)> {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for finding in findings.iter().filter(|finding| keep(finding)) {
        *counts.entry(category(finding.reason())).or_default() += 1;
    }
    let mut counts: Vec<_> = counts.into_iter().collect();
    counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    counts
}

fn section(text: &mut String, title: &str, lines: Vec<String>) {
    if lines.is_empty() {
        return;
    }
    text.push_str(&format!("{title} ({}):\n", lines.len()));
    for line in lines {
        text.push_str(&line);
        text.push('\n');
    }
    text.push('\n');
}

fn finding_line(finding: &Finding, verdict: &str) -> String {
    format!(
        "{}:{}  {}.{}  {}()  {}: {}",
        finding.path().display(),
        finding.line(),
        finding.class_name(),
        finding.method_name(),
        finding.call(),
        verdict,
        describe(finding.reason())
    )
}

fn hybrid_line(finding: &HybridFinding) -> String {
    let hybrid = finding.hybrid();
    format!(
        "{}  {}  Hybrid class: behavior: {}; setters: {}",
        finding.path().display(),
        hybrid.class(),
        hybrid.behavior_methods().join(", "),
        hybrid.setters().join(", ")
    )
}

fn error_line(error: &ReadError) -> String {
    let location = match error.line() {
        Some(line) => format!("{}:{line}", error.path().display()),
        None => error.path().display().to_string(),
    };
    format!("{location}  Error: {}", error.message())
}

#[cfg(test)]
mod tests;
