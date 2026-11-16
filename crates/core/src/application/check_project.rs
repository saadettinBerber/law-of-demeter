use super::ports::{SourceFile, SourceReader, TypeReading, TypeSource};
use super::report::{Finding, HybridFinding, Report};
use crate::domain::classification::Types;

pub struct CheckProject<'a> {
    sources: &'a dyn SourceReader,
    types: &'a dyn TypeSource,
}

impl<'a> CheckProject<'a> {
    pub fn new(sources: &'a dyn SourceReader, types: &'a dyn TypeSource) -> Self {
        Self { sources, types }
    }

    pub fn execute(&self) -> Report {
        let TypeReading {
            shapes,
            errors,
            known,
        } = self.types.read();
        let shapes = shapes
            .into_iter()
            .fold(Types::new().with_known(known), Types::with_shape);
        let files = self.sources.read(&|name| shapes.shape(name).is_some());
        let declared = files
            .iter()
            .flatten()
            .flat_map(|file| file.classes().iter().cloned());
        let catalog = shapes.with_project_types(declared);
        let mut report = Report {
            type_errors: errors,
            ..Report::default()
        };
        for file in files {
            match file {
                Ok(file) => judge(&file, &catalog, &mut report),
                Err(error) => report.source_errors.push(error),
            }
        }
        report
    }
}

fn judge(file: &SourceFile, catalog: &Types, report: &mut Report) {
    report.files += 1;
    report.methods += file.methods().len();
    for class in file.classes() {
        if let Some(hybrid) = catalog.hybrid_class(class) {
            report
                .hybrid_classes
                .push(HybridFinding::new(file.path(), hybrid));
        }
    }
    for method in file.methods() {
        for verdict in method.verdicts(catalog) {
            report.findings.push(Finding::new(
                file.path(),
                method.class_name(),
                method.name(),
                verdict,
            ));
        }
    }
}
