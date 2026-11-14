use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use law_of_demeter_core::domain::type_shape::TypeShape;
use zip::ZipArchive;

use crate::{Error, Reading, shape};

pub(crate) fn read_into(path: &Path, reading: &mut Reading) {
    if path.is_dir() {
        read_directory(path, reading);
    } else if has_extension(path, "jar") {
        read_jar(path, reading);
    } else {
        reading.record(read_class_file(path));
    }
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension().is_some_and(|actual| actual == extension)
}

fn read_directory(directory: &Path, reading: &mut Reading) {
    let paths = match sorted_entries(directory) {
        Ok(paths) => paths,
        Err(error) => return reading.errors.push(error),
    };
    for path in paths {
        if path.is_dir() {
            read_directory(&path, reading);
        } else if has_extension(&path, "class") {
            reading.record(read_class_file(&path));
        }
    }
}

fn sorted_entries(directory: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory).map_err(|error| Error::new(directory, error))? {
        paths.push(entry.map_err(|error| Error::new(directory, error))?.path());
    }
    paths.sort();
    Ok(paths)
}

fn read_class_file(path: &Path) -> Result<Option<TypeShape>, Error> {
    let bytes = fs::read(path).map_err(|error| Error::new(path, error))?;
    shape::from_bytes(&bytes).map_err(|message| Error::new(path, message))
}

fn read_jar(path: &Path, reading: &mut Reading) {
    let mut archive = match open_jar(path) {
        Ok(archive) => archive,
        Err(error) => return reading.errors.push(error),
    };
    for index in 0..archive.len() {
        reading.record(read_jar_entry(&mut archive, index, path));
    }
}

fn open_jar(path: &Path) -> Result<ZipArchive<File>, Error> {
    let file = File::open(path).map_err(|error| Error::new(path, error))?;
    ZipArchive::new(file).map_err(|error| Error::new(path, error))
}

fn read_jar_entry(
    archive: &mut ZipArchive<File>,
    index: usize,
    path: &Path,
) -> Result<Option<TypeShape>, Error> {
    let mut entry = archive
        .by_index(index)
        .map_err(|error| Error::new(path, error))?;
    let name = entry
        .name()
        .map_err(|error| Error::new(path, error))?
        .into_owned();
    if !name.ends_with(".class") || name.starts_with("META-INF/") {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|error| Error::new(path, format!("{name}: {error}")))?;
    shape::from_bytes(&bytes).map_err(|message| Error::new(path, format!("{name}: {message}")))
}
