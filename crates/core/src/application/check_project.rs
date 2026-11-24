use super::ports::{SourceFile, SourceReader, TypeReading, TypeSource};
use super::report::{Finding, HybridFinding, Report};
use super::selection::Selection;
use crate::domain::classification::Types;
use crate::domain::method::Method;

pub struct CheckProject<'a> {
    sources: &'a dyn SourceReader,
    types: &'a dyn TypeSource,
    selection: Selection,
}

impl<'a> CheckProject<'a> {
    pub fn new(sources: &'a dyn SourceReader, types: &'a dyn TypeSource) -> Self {
        Self {
            sources,
            types,
            selection: Selection::everything(),
        }
    }

    pub fn with_selection(mut self, selection: Selection) -> Self {
        self.selection = selection;
        self
    }

    pub fn execute(&self) -> Report {
        let TypeReading {
            shapes,
            errors,
            platform,
        } = self.types.read();
        let types = shapes
            .into_iter()
            .fold(Types::new().with_platform(platform), Types::with_shape)
            .with_project_types(self.types.project_types());
        let files = self.sources.read(&types);
        let declared = files
            .iter()
            .flatten()
            .flat_map(|file| file.classes().iter().cloned());
        let catalog = types.with_project_types(declared);
        let mut report = Report {
            type_errors: errors,
            issues: self
                .sources
                .issues()
                .into_iter()
                .filter(|issue| {
                    self.selection
                        .selects_issue(issue, files.iter().flatten(), &catalog)
                })
                .collect(),
            unmatched_targets: self.selection.unmatched(files.iter().flatten(), &catalog),
            ..Report::default()
        };
        for file in files {
            match file {
                Ok(file) => judge(&file, &catalog, &self.selection, &mut report),
                Err(error) => report.source_errors.push(error),
            }
        }
        report
    }
}

fn judge(file: &SourceFile, catalog: &Types, selection: &Selection, report: &mut Report) {
    if selection.covers(file.classes(), catalog) {
        report.files += 1;
    }
    let methods: Vec<&Method> = file
        .methods()
        .iter()
        .filter(|method| selection.selects(method, catalog))
        .collect();
    report.methods += methods.len();
    for class in file
        .classes()
        .iter()
        .filter(|class| selection.selects_class(class, catalog))
    {
        if let Some(hybrid) = catalog.hybrid_class(class) {
            report
                .hybrid_classes
                .push(HybridFinding::new(file.path(), hybrid));
        }
    }
    for method in methods {
        for verdict in method.verdicts_among(catalog, file.methods()) {
            report.findings.push(Finding::new(
                file.path(),
                method.class_name(),
                method.name(),
                verdict,
            ));
        }
    }
}
