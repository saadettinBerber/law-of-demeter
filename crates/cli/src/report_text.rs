use law_of_demeter_core::application::{Finding, HybridFinding, ReadError, Report};

use crate::reason_text::describe;

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
    text
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
