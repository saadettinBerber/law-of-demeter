use std::path::PathBuf;

use law_of_demeter_class_file::{Error, Jdk, Reading, read_types_lenient};
use law_of_demeter_core::application::{ReadError, TypeReading, TypeSource};
use law_of_demeter_java_source::java_platform;

pub struct ClassFileTypes {
    classpath: Vec<PathBuf>,
    jdk: Option<(Jdk, u32)>,
}

impl ClassFileTypes {
    pub fn new(classpath: Vec<PathBuf>, jdk: Option<(Jdk, u32)>) -> Self {
        Self { classpath, jdk }
    }
}

impl TypeSource for ClassFileTypes {
    fn read(&self) -> TypeReading {
        let mut reading = TypeReading {
            platform: java_platform(),
            ..TypeReading::default()
        };
        if let Some((jdk, release)) = &self.jdk {
            match jdk.types(*release) {
                Ok(types) => absorb(types, &mut reading),
                Err(error) => reading.errors.push(read_error(&error)),
            }
        }
        absorb(read_types_lenient(&self.classpath), &mut reading);
        reading
    }
}

fn absorb(types: Reading, reading: &mut TypeReading) {
    reading.shapes.extend(types.shapes);
    reading.errors.extend(types.errors.iter().map(read_error));
}

fn read_error(error: &Error) -> ReadError {
    let text = error.to_string();
    let prefix = format!("{}: ", error.path().display());
    ReadError::new(error.path(), text.strip_prefix(&prefix).unwrap_or(&text))
}

#[cfg(test)]
mod tests;
