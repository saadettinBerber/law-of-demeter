use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use law_of_demeter_class_file::{Jdk, read_types};
use law_of_demeter_core::application::{
    CheckProject, Finding, ReadError, Report, Selection, SourceFile, SourceReader, Target,
    TypeReading, TypeSource,
};
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::TypeRef;
use law_of_demeter_java_source::{java_platform, map_source};

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_created_owner_is_a_friend_but_its_private_collection_is_not() {
    let fixture = CompiledFixture::new();

    let report = fixture.check("shop.Order", "create");

    assert_eq!(
        report.findings(),
        [
            Finding::new(
                &fixture.source,
                "shop.Order",
                "create",
                Verdict::allowed(
                    "addMember",
                    16,
                    Reason::local("result", Reason::created(TypeRef::class("shop.Order"))),
                )
            ),
            Finding::new(
                &fixture.source,
                "shop.Order",
                "create",
                Verdict::violation(
                    "add",
                    17,
                    Reason::container_mutation(
                        TypeRef::generic("java.util.List", vec![TypeRef::class("shop.Member")]),
                        Reason::field_of("items"),
                    ),
                )
            ),
        ]
    );
    assert_complete(&report);
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn private_access_to_another_instance_does_not_make_its_collection_a_friend() {
    let fixture = CompiledFixture::new();

    let report = fixture.check("shop.Order", "copy");

    assert_eq!(
        report.findings(),
        [
            Finding::new(
                &fixture.source,
                "shop.Order",
                "copy",
                Verdict::allowed("addMember", 22, Reason::parameter("other"))
            ),
            Finding::new(
                &fixture.source,
                "shop.Order",
                "copy",
                Verdict::violation(
                    "add",
                    23,
                    Reason::container_mutation(
                        TypeRef::generic("java.util.List", vec![TypeRef::class("shop.Member")]),
                        Reason::field_of("items"),
                    ),
                )
            ),
        ]
    );
    assert_complete(&report);
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_new_holder_does_not_make_the_wallet_it_carries_a_friend() {
    let fixture = CompiledFixture::new();

    let report = fixture.check("shop.Billing", "charge");

    assert_eq!(
        report.findings(),
        [
            Finding::new(
                &fixture.source,
                "shop.Billing",
                "charge",
                Verdict::allowed("wallet", 36, Reason::parameter("order"))
            ),
            Finding::new(
                &fixture.source,
                "shop.Billing",
                "charge",
                Verdict::allowed(
                    "charge",
                    37,
                    Reason::local("holder", Reason::created(TypeRef::class("shop.Holder"))),
                )
            ),
            Finding::new(
                &fixture.source,
                "shop.Billing",
                "charge",
                Verdict::violation("withdraw", 38, Reason::field_of("wallet"))
            ),
        ]
    );
    assert_complete(&report);
}

fn assert_complete(report: &Report) {
    assert!(
        report.source_errors().is_empty(),
        "{:?}",
        report.source_errors()
    );
    assert!(
        report.type_errors().is_empty(),
        "{:?}",
        report.type_errors()
    );
    assert!(report.issues().is_empty(), "{:?}", report.issues());
    assert!(
        report.unmatched_targets().is_empty(),
        "{:?}",
        report.unmatched_targets()
    );
}

struct CompiledFixture {
    _temporary: tempfile::TempDir,
    source: PathBuf,
    classes: PathBuf,
    jdk: Jdk,
}

impl CompiledFixture {
    fn new() -> Self {
        let temporary = tempfile::tempdir().expect("temporary class directory");
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/field-friendship/Order.java");
        let classes = temporary.path().join("classes");
        let jdk = Jdk::locate().expect("these tests require a JDK with javac");
        let compiled = Command::new(jdk.home().join("bin/javac"))
            .args(["--release", "17", "-d"])
            .arg(&classes)
            .arg(&source)
            .output()
            .expect("javac runs");
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        Self {
            _temporary: temporary,
            source,
            classes,
            jdk,
        }
    }

    fn check(&self, class: &str, method: &str) -> Report {
        CheckProject::new(self, self)
            .with_selection(Selection::of(vec![Target::method(class, method)]))
            .execute()
    }
}

impl SourceReader for CompiledFixture {
    fn read(&self, types: &Types) -> Vec<Result<SourceFile, ReadError>> {
        let result = fs::read_to_string(&self.source)
            .map_err(|error| ReadError::new(&self.source, &error.to_string()))
            .and_then(|source| {
                map_source(&self.source, &source, types)
                    .map(|mapped| SourceFile::new(&self.source, mapped.classes, mapped.methods))
                    .map_err(|error| ReadError::new(&self.source, &error.to_string()))
            });
        vec![result]
    }
}

impl TypeSource for CompiledFixture {
    fn read(&self) -> TypeReading {
        let mut reading = self
            .jdk
            .types(17)
            .expect("JDK release 17 types are readable");
        assert!(reading.errors.is_empty(), "{:?}", reading.errors);
        reading
            .shapes
            .extend(read_types(&[&self.classes]).expect("compiled fixture types"));
        TypeReading {
            shapes: reading.shapes,
            platform: java_platform(),
            ..TypeReading::default()
        }
    }
}
