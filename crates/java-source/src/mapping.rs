use std::path::Path;

use law_of_demeter_core::domain::method::Method;

use crate::{Class, ParseError, SourceFile, parse_with};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedFile {
    pub classes: Vec<String>,
    pub methods: Vec<Method>,
}

pub fn map_source(
    path: &Path,
    source: &str,
    type_exists: &dyn Fn(&str) -> bool,
) -> Result<MappedFile, ParseError> {
    let file = parse_with(&path.to_string_lossy(), source, type_exists)?;
    Ok(map_file(&file))
}

fn map_file(file: &SourceFile) -> MappedFile {
    MappedFile {
        classes: file
            .classes
            .iter()
            .map(|class| class.name.clone())
            .collect(),
        methods: file
            .classes
            .iter()
            .flat_map(|class| map_class(file, class))
            .collect(),
    }
}

fn map_class(file: &SourceFile, class: &Class) -> Vec<Method> {
    let _ = file;
    class
        .methods
        .iter()
        .map(|method| {
            let name = if method.is_constructor() {
                "<init>"
            } else {
                &method.name
            };
            Method::new(&class.name, name, method.line)
        })
        .collect()
}
