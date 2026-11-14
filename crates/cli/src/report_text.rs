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
    section(
        &mut text,
        "İhlaller",
        finding_lines(
            |finding| finding.is_violation() && !finding.is_unresolved(),
            "İhlal",
        ),
    );
    section(
        &mut text,
        "Çözülemeyen tipler",
        finding_lines(Finding::is_unresolved, "Belirsiz"),
    );
    if show_allowed {
        section(
            &mut text,
            "Uygun çağrılar",
            finding_lines(|finding| !finding.is_violation(), "Uygun"),
        );
    }
    section(
        &mut text,
        "Hibrit sınıflar",
        report.hybrid_classes().iter().map(hybrid_line).collect(),
    );
    section(
        &mut text,
        "Kaynak hataları",
        report.source_errors().iter().map(error_line).collect(),
    );
    section(
        &mut text,
        "Tip okuma hataları",
        report.type_errors().iter().map(error_line).collect(),
    );
    text.push_str(&summary(report));
    text
}

fn summary(report: &Report) -> String {
    let counts = report.counts();
    let mut text = String::from("Özet:\n");
    for line in [
        format!("Dosya: {}", counts.files),
        format!("Metot: {}", counts.methods),
        format!(
            "Çağrı: {} (uygun {}, ihlal {}, belirsiz {})",
            counts.calls, counts.allowed, counts.violations, counts.unresolved
        ),
        format!("Hibrit sınıf: {}", counts.hybrid_classes),
        format!("Kaynak hatası: {}", counts.source_errors),
        format!("Tip okuma hatası: {}", counts.type_errors),
    ] {
        text.push_str(&format!("  {line}\n"));
    }
    let reasons = reason_counts(report.findings(), |finding| {
        finding.is_violation() && !finding.is_unresolved()
    });
    if !reasons.is_empty() {
        text.push_str("\nİhlal nedenleri:\n");
        for (reason, count) in reasons {
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
        "{}  {}  Hibrit sınıf: davranış: {}; setter: {}",
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
    format!("{location}  Hata: {}", error.message())
}

#[cfg(test)]
mod tests;
