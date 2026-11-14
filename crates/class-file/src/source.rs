use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use law_of_demeter_core::domain::type_shape::TypeShape;
use zip::ZipArchive;

use crate::{Error, shape};

pub(crate) fn read_into(path: &Path, shapes: &mut Vec<TypeShape>) -> Result<(), Error> {
    if path.is_dir() {
        read_directory(path, shapes)
    } else if has_extension(path, "jar") {
        read_jar(path, shapes)
    } else {
        read_class_file(path, shapes)
    }
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension().is_some_and(|actual| actual == extension)
}

fn read_directory(directory: &Path, shapes: &mut Vec<TypeShape>) -> Result<(), Error> {
    let entries = fs::read_dir(directory).map_err(|error| Error::new(directory, error))?;
    for entry in entries {
        let path = entry.map_err(|error| Error::new(directory, error))?.path();
        if path.is_dir() {
            read_directory(&path, shapes)?;
        } else if has_extension(&path, "class") {
            read_class_file(&path, shapes)?;
        }
    }
    Ok(())
}

fn read_class_file(path: &Path, shapes: &mut Vec<TypeShape>) -> Result<(), Error> {
    let bytes = fs::read(path).map_err(|error| Error::new(path, error))?;
    shapes.extend(shape::from_bytes(&bytes).map_err(|message| Error::new(path, message))?);
    Ok(())
}

fn read_jar(path: &Path, shapes: &mut Vec<TypeShape>) -> Result<(), Error> {
    let file = File::open(path).map_err(|error| Error::new(path, error))?;
    let mut archive = ZipArchive::new(file).map_err(|error| Error::new(path, error))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| Error::new(path, error))?;
        let name = entry
            .name()
            .map_err(|error| Error::new(path, error))?
            .into_owned();
        if !name.ends_with(".class") || name.starts_with("META-INF/") {
            continue;
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| Error::new(path, format!("{name}: {error}")))?;
        shapes.extend(
            shape::from_bytes(&bytes)
                .map_err(|message| Error::new(path, format!("{name}: {message}")))?,
        );
    }
    Ok(())
}
