use std::fs;
use std::path::Path;

use law_of_demeter_core::domain::type_shape::TypeShape;

use crate::{Error, shape};

pub(crate) fn read_into(path: &Path, shapes: &mut Vec<TypeShape>) -> Result<(), Error> {
    if path.is_dir() {
        read_directory(path, shapes)
    } else {
        read_class_file(path, shapes)
    }
}

fn read_directory(directory: &Path, shapes: &mut Vec<TypeShape>) -> Result<(), Error> {
    let entries = fs::read_dir(directory).map_err(|error| Error::new(directory, error))?;
    for entry in entries {
        let path = entry.map_err(|error| Error::new(directory, error))?.path();
        if path.is_dir() {
            read_directory(&path, shapes)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "class")
        {
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
