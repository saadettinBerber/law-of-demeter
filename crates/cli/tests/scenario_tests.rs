use std::fs;
use std::path::Path;

const SCENARIOS: &str = "SCENARIOS.md";
const END_TO_END: &str = "crates/java-source/tests/end_to_end.rs";
const SUMMARY_START: &str = "<summary><strong>";
const SECTIONS: [(&str, &str); 3] = [
    ("## Uygun değil", "violation"),
    ("## Uygun", "allowed"),
    ("## Kapsam dışı", "out_of_scope"),
];

#[test]
fn every_scenario_has_a_numbered_end_to_end_test() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scenarios = fs::read_to_string(root.join(SCENARIOS)).expect("SCENARIOS.md is readable");
    let tests = fs::read_to_string(root.join(END_TO_END)).expect("end_to_end.rs is readable");
    let tests = tests.as_str();

    let missing: Vec<String> = SECTIONS
        .iter()
        .flat_map(|(heading, prefix)| {
            numbers_under(&scenarios, heading)
                .into_iter()
                .filter(move |number| !tests.contains(&format!("fn {prefix}_{number}_")))
                .map(move |number| format!("{heading} {number} needs a `fn {prefix}_{number}_…`"))
        })
        .collect();

    assert!(
        missing.is_empty(),
        "{END_TO_END} lacks tests for scenarios:\n{}",
        missing.join("\n")
    );
}

fn numbers_under(markdown: &str, heading: &str) -> Vec<u32> {
    markdown
        .lines()
        .skip_while(|line| *line != heading)
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .filter_map(|line| line.strip_prefix(SUMMARY_START))
        .filter_map(|title| title.split_once('.')?.0.parse().ok())
        .collect()
}
