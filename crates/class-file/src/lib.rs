mod error;
mod shape;
mod source;

use std::path::Path;

use law_of_demeter_core::domain::type_shape::TypeShape;

pub use error::Error;

pub fn read_types<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<TypeShape>, Error> {
    let mut shapes = Vec::new();
    for path in paths {
        source::read_into(path.as_ref(), &mut shapes)?;
    }
    shapes.sort_by(|left, right| left.name().cmp(right.name()));
    Ok(shapes)
}
