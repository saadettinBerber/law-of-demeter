mod accessor;
mod error;
mod shape;
mod signature;
mod source;

use std::path::Path;

use law_of_demeter_core::domain::type_shape::TypeShape;

pub use error::Error;

pub struct Reading {
    pub shapes: Vec<TypeShape>,
    pub errors: Vec<Error>,
}

impl Reading {
    pub(crate) fn record(&mut self, result: Result<Option<TypeShape>, Error>) {
        match result {
            Ok(shape) => self.shapes.extend(shape),
            Err(error) => self.errors.push(error),
        }
    }
}

pub fn read_types_lenient<P: AsRef<Path>>(paths: &[P]) -> Reading {
    let mut reading = Reading {
        shapes: Vec::new(),
        errors: Vec::new(),
    };
    for path in paths {
        source::read_into(path.as_ref(), &mut reading);
    }
    reading
        .shapes
        .sort_by(|left, right| left.name().cmp(right.name()));
    reading
}

pub fn read_types<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<TypeShape>, Error> {
    let Reading { shapes, errors } = read_types_lenient(paths);
    match errors.into_iter().next() {
        Some(error) => Err(error),
        None => Ok(shapes),
    }
}
