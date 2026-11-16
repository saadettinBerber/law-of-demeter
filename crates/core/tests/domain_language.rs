use std::fs;
use std::path::{Path, PathBuf};

const LANGUAGE_NAMES: [&str; 1] = ["java"];

#[test]
fn domain_production_sources_name_no_language() {
    let domain = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/domain");

    let sources = rust_files(&domain);
    let test_only_modules = test_only_module_names(&sources);
    let offenders: Vec<String> = sources
        .iter()
        .filter(|path| !is_test_only(path, &test_only_modules))
        .filter(|path| names_a_language(&production_part(&read(path))))
        .map(|path| path.display().to_string())
        .collect();

    assert!(
        offenders.is_empty(),
        "the domain must not name a language: {offenders:?}"
    );
}

fn rust_files(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).expect("domain directory is readable") {
        let path = entry.expect("directory entry is readable").path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("domain source is readable")
}

fn test_only_module_names(sources: &[PathBuf]) -> Vec<String> {
    sources
        .iter()
        .flat_map(|path| {
            let text = read(path);
            let lines: Vec<&str> = text.lines().collect();
            lines
                .windows(2)
                .filter(|pair| pair[0].trim() == "#[cfg(test)]")
                .filter_map(|pair| pair[1].trim().strip_prefix("mod "))
                .filter_map(|rest| rest.strip_suffix(';'))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn is_test_only(path: &Path, test_only_modules: &[String]) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| test_only_modules.iter().any(|module| module == stem))
}

fn production_part(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let end = lines
        .iter()
        .enumerate()
        .position(|(index, line)| {
            line.trim() == "#[cfg(test)]"
                && lines
                    .get(index + 1)
                    .is_some_and(|next| is_inline_module(next))
        })
        .unwrap_or(lines.len());
    lines[..end].join("\n")
}

fn is_inline_module(line: &str) -> bool {
    let line = line.trim();
    line.starts_with("mod ") && line.ends_with('{')
}

fn names_a_language(text: &str) -> bool {
    let lowered = text.to_lowercase();
    LANGUAGE_NAMES.iter().any(|name| lowered.contains(name))
}

#[test]
fn a_test_only_item_before_production_code_does_not_hide_it() {
    let source = "#[cfg(test)]\nconst MARKER: u8 = 0;\nconst LANGUAGE: &str = \"java\";";

    let production = production_part(source);

    assert!(names_a_language(&production));
}

#[test]
fn an_inline_test_module_is_not_production_code() {
    let source = "fn rule() {}\n#[cfg(test)]\nmod tests {\n    const LANGUAGE: &str = \"java\";\n}";

    let production = production_part(source);

    assert!(!names_a_language(&production));
}
