use std::fs;
use std::path::{Path, PathBuf};

use law_of_demeter_core::application::ReadError;

#[derive(Debug, Default)]
pub struct JavaFiles {
    pub files: Vec<PathBuf>,
    pub errors: Vec<ReadError>,
}

pub fn java_files(roots: &[PathBuf]) -> JavaFiles {
    let mut found = JavaFiles::default();
    for root in roots {
        walk(root, &mut found);
    }
    found
}

fn walk(directory: &Path, found: &mut JavaFiles) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            found
                .errors
                .push(ReadError::new(directory, &error.to_string()));
            return;
        }
    };
    let mut paths: Vec<_> = entries.filter_map(Result::ok).collect();
    paths.sort_by_key(fs::DirEntry::path);
    for entry in paths {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            walk(&path, found);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "java")
        {
            found.files.push(path);
        }
    }
}

#[cfg(test)]
mod tests;
