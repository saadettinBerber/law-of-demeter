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
    text.lines()
        .take_while(|line| line.trim() != "#[cfg(test)]")
        .collect::<Vec<_>>()
        .join("\n")
}

fn names_a_language(text: &str) -> bool {
    let lowered = text.to_lowercase();
    LANGUAGE_NAMES.iter().any(|name| lowered.contains(name))
}
