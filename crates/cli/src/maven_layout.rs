use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_ROOT: &str = "src/main/java";
const BUILD_DIRECTORY: &str = "target";

#[derive(Debug, Default)]
pub struct MavenLayout {
    pub sources: Vec<PathBuf>,
    pub classpath: Vec<PathBuf>,
}

pub fn maven_layout(directory: &Path) -> MavenLayout {
    let mut layout = MavenLayout::default();
    layout.scan(directory);
    layout.classpath = without_duplicates(std::mem::take(&mut layout.classpath));
    layout
}

impl MavenLayout {
    fn scan(&mut self, directory: &Path) {
        let Ok(entries) = fs::read_dir(Path::new(".").join(directory)) else {
            return;
        };
        let mut directories: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .collect();
        directories.sort_by_key(fs::DirEntry::file_name);
        for entry in directories {
            let name = entry.file_name();
            let child = directory.join(&name);
            if child.ends_with(SOURCE_ROOT) {
                self.sources.push(child);
            } else if name == BUILD_DIRECTORY {
                self.read_build_output(&child);
            } else if !name.to_string_lossy().starts_with('.') {
                self.scan(&child);
            }
        }
    }

    fn read_build_output(&mut self, target: &Path) {
        let classes = target.join("classes");
        if classes.is_dir() {
            self.classpath.push(classes);
        }
        if let Ok(listed) = fs::read_to_string(target.join("classpath.txt")) {
            self.classpath.extend(
                env::split_paths(listed.trim()).filter(|entry| !entry.as_os_str().is_empty()),
            );
        }
    }
}

fn without_duplicates(entries: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    entries
        .into_iter()
        .filter(|entry| seen.insert(fs::canonicalize(entry).unwrap_or_else(|_| entry.clone())))
        .collect()
}
