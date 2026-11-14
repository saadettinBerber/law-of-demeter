use law_of_demeter_core::application::{Finding, Report};

use crate::reason_text::describe;

pub fn render(report: &Report, show_allowed: bool) -> String {
    let _ = show_allowed;
    let violations: Vec<&Finding> = report
        .findings()
        .iter()
        .filter(|finding| finding.is_violation() && !finding.is_unresolved())
        .collect();
    let mut text = String::new();
    if !violations.is_empty() {
        text.push_str(&format!("İhlaller ({}):\n", violations.len()));
        for finding in violations {
            text.push_str(&line(finding, "İhlal"));
        }
        text.push('\n');
    }
    text
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
