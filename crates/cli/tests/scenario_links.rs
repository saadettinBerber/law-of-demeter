use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SCENARIOS: &str = "SCENARIOS.md";
const EXPECTED_REPORT: &str = "expected-report.txt";
const LINK_START: &str = "](examples/";
const COLUMN_SEPARATOR: &str = "  ";

#[test]
fn every_scenario_link_points_at_a_finding_in_the_expected_report() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scenarios = fs::read_to_string(root.join(SCENARIOS)).expect("SCENARIOS.md is readable");
    let mut reports: HashMap<String, Result<Vec<Finding>, String>> = HashMap::new();

    let problems: Vec<String> = link_targets(&scenarios)
        .into_iter()
        .filter_map(|target| {
            let link = Link::parse(target).map_err(|error| format!("{target}: {error}"));
            link.and_then(|link| {
                let report = reports
                    .entry(link.project.clone())
                    .or_insert_with(|| read_findings(&root, &link.project));
                match report {
                    Ok(findings) if findings.iter().any(|finding| link.covers(finding)) => Ok(()),
                    Ok(_) => Err(format!(
                        "{target}: no finding of {}/{EXPECTED_REPORT} is on these lines",
                        link.project
                    )),
                    Err(error) => Err(format!("{target}: {error}")),
                }
            })
            .err()
        })
        .collect();

    assert!(
        problems.is_empty(),
        "{SCENARIOS} has stale links:\n{}",
        problems.join("\n")
    );
}

fn link_targets(markdown: &str) -> Vec<&str> {
    markdown
        .split(LINK_START)
        .skip(1)
        .filter_map(|rest| rest.split_once(')').map(|(target, _)| target))
        .filter(|target| target.contains(".java"))
        .collect()
}

struct Link {
    project: String,
    file: String,
    first: u32,
    last: u32,
}

impl Link {
    fn parse(target: &str) -> Result<Self, String> {
        let (project, rest) = target.split_once('/').ok_or("the link has no file path")?;
        let (file, lines) = rest.split_once("#L").ok_or("the link has no line anchor")?;
        let (first, last) = lines.split_once("-L").unwrap_or((lines, lines));
        let number = |text: &str| {
            text.parse::<u32>()
                .map_err(|_| format!("`{text}` is not a line number"))
        };
        Ok(Self {
            project: project.to_owned(),
            file: file.to_owned(),
            first: number(first)?,
            last: number(last)?,
        })
    }

    fn covers(&self, finding: &Finding) -> bool {
        finding.file == self.file
            && finding
                .line
                .is_none_or(|line| (self.first..=self.last).contains(&line))
    }
}

struct Finding {
    file: String,
    line: Option<u32>,
}

impl Finding {
    fn parse(report_line: &str) -> Option<Self> {
        let location = report_line.split(COLUMN_SEPARATOR).next()?;
        if !location.ends_with(".java") && !location.contains(".java:") {
            return None;
        }
        match location.split_once(':') {
            Some((file, line)) => Some(Self {
                file: file.to_owned(),
                line: Some(line.parse().ok()?),
            }),
            None => Some(Self {
                file: location.to_owned(),
                line: None,
            }),
        }
    }
}

fn read_findings(root: &Path, project: &str) -> Result<Vec<Finding>, String> {
    let report: PathBuf = root.join("examples").join(project).join(EXPECTED_REPORT);
    let text = fs::read_to_string(&report)
        .map_err(|error| format!("cannot read {project}/{EXPECTED_REPORT}: {error}"))?;
    Ok(text.lines().filter_map(Finding::parse).collect())
}
