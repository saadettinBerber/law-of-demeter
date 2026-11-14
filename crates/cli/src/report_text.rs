use law_of_demeter_core::application::{Finding, Report};

use crate::reason_text::describe;

pub fn render(report: &Report, show_allowed: bool) -> String {
    let findings = report.findings();
    let mut text = String::new();
    section(
        &mut text,
        "İhlaller",
        "İhlal",
        findings
            .iter()
            .filter(|finding| finding.is_violation() && !finding.is_unresolved()),
    );
    section(
        &mut text,
        "Çözülemeyen tipler",
        "Belirsiz",
        findings.iter().filter(|finding| finding.is_unresolved()),
    );
    if show_allowed {
        section(
            &mut text,
            "Uygun çağrılar",
            "Uygun",
            findings.iter().filter(|finding| !finding.is_violation()),
        );
    }
    text
}

fn section<'a>(
    text: &mut String,
    title: &str,
    verdict: &str,
    findings: impl Iterator<Item = &'a Finding>,
) {
    let findings: Vec<&Finding> = findings.collect();
    if findings.is_empty() {
        return;
    }
    text.push_str(&format!("{title} ({}):\n", findings.len()));
    for finding in findings {
        text.push_str(&line(finding, verdict));
    }
    text.push('\n');
}

fn line(finding: &Finding, verdict: &str) -> String {
    format!(
        "{}:{}  {}.{}  {}()  {}: {}\n",
        finding.path().display(),
        finding.line(),
        finding.class_name(),
        finding.method_name(),
        finding.call(),
        verdict,
        describe(finding.reason())
    )
}

#[cfg(test)]
mod tests;
