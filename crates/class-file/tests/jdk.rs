use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use law_of_demeter_class_file::{Error, Jdk, Reading};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

fn fixture_jdk() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jdk")
}

fn scratch_directory(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn jdk_with_release(name: &str, release_file: &str) -> Jdk {
    let home = scratch_directory(name);
    fs::write(home.join("release"), release_file).unwrap();
    Jdk::at(home)
}

fn reading_of(release: u32) -> Reading {
    Jdk::at(fixture_jdk()).types(release).unwrap()
}

fn names(reading: &Reading) -> Vec<&str> {
    reading.shapes.iter().map(TypeShape::name).collect()
}

fn shape<'a>(reading: &'a Reading, name: &str) -> &'a TypeShape {
    reading
        .shapes
        .iter()
        .find(|shape| shape.name() == name)
        .unwrap_or_else(|| panic!("no type named {name}"))
}

fn method_names(shape: &TypeShape) -> Vec<&str> {
    shape.methods().iter().map(MethodShape::name).collect()
}

fn error_of(result: Result<Reading, Error>) -> Error {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(error) => error,
    }
}

fn fake_jdk_with_javac(name: &str) -> (PathBuf, PathBuf) {
    let home = scratch_directory(name);
    fs::create_dir_all(home.join("bin")).unwrap();
    fs::write(home.join("release"), "JAVA_VERSION=\"17.0.1\"\n").unwrap();
    fs::write(home.join("bin/javac"), "").unwrap();
    (home.clone(), home.join("bin"))
}

fn located_home(java_home: Option<&Path>, path: Option<&Path>) -> Option<PathBuf> {
    let path: Option<OsString> = path.map(|directory| directory.as_os_str().to_owned());
    let java_home = java_home.map(|directory| directory.as_os_str().to_owned());
    Jdk::locate_in(java_home, path).map(|jdk| jdk.home().to_path_buf())
}

#[test]
fn the_feature_version_comes_from_the_release_file() {
    let jdk = Jdk::at(fixture_jdk());

    let version = jdk.feature_version().unwrap();

    assert_eq!(version, 12);
}

#[test]
fn a_legacy_version_number_names_its_feature_version_in_the_second_part() {
    let jdk = jdk_with_release("legacy-version", "JAVA_VERSION=\"1.8.0_292\"\n");

    let version = jdk.feature_version().unwrap();

    assert_eq!(version, 8);
}

#[test]
fn an_early_access_version_suffix_is_ignored() {
    let jdk = jdk_with_release("early-access", "JAVA_VERSION=\"25-ea\"\n");

    let version = jdk.feature_version().unwrap();

    assert_eq!(version, 25);
}

#[test]
fn a_missing_release_file_is_an_error_carrying_its_path() {
    let home = scratch_directory("no-release-file");

    let error = Jdk::at(&home).feature_version().unwrap_err();

    assert_eq!(error.path(), home.join("release"));
}

#[test]
fn a_release_file_without_a_version_is_an_error_carrying_its_path() {
    let jdk = jdk_with_release("no-version", "IMPLEMENTOR=\"Somebody\"\n");

    let error = jdk.feature_version().unwrap_err();

    assert_eq!(error.path(), jdk.home().join("release"));
}

#[test]
fn java_home_is_preferred_when_it_points_to_a_jdk() {
    let (other_home, other_bin) = fake_jdk_with_javac("other-jdk");

    let home = located_home(Some(&fixture_jdk()), Some(&other_bin));

    assert_eq!(home, Some(fixture_jdk()));
    assert_ne!(home, Some(other_home));
}

#[test]
fn javac_on_the_path_is_used_when_java_home_is_not_a_jdk() {
    let (jdk_home, bin) = fake_jdk_with_javac("javac-on-path");
    let stale = scratch_directory("stale-java-home");

    let home = located_home(Some(&stale), Some(&bin));

    assert_eq!(home, Some(fs::canonicalize(jdk_home).unwrap()));
}

#[cfg(unix)]
#[test]
fn a_javac_symlink_leads_to_the_jdk_it_points_into() {
    let (jdk_home, _) = fake_jdk_with_javac("symlinked-jdk");
    let links = scratch_directory("javac-links");
    std::os::unix::fs::symlink(jdk_home.join("bin/javac"), links.join("javac")).unwrap();

    let home = located_home(None, Some(&links));

    assert_eq!(home, Some(fs::canonicalize(jdk_home).unwrap()));
}

#[test]
fn a_javac_that_does_not_sit_in_a_jdk_is_not_used() {
    let home = scratch_directory("javac-without-jdk");
    fs::create_dir_all(home.join("bin")).unwrap();
    fs::write(home.join("bin/javac"), "").unwrap();

    let found = located_home(None, Some(&home.join("bin")));

    assert_eq!(found, None);
}

#[test]
fn no_jdk_is_found_without_java_home_or_javac() {
    let empty = scratch_directory("no-javac-here");

    let home = located_home(None, Some(&empty));

    assert_eq!(home, None);
}

#[test]
fn a_release_lists_the_types_whose_release_directory_names_it() {
    let reading = reading_of(8);

    assert_eq!(
        names(&reading),
        [
            "sample.awt.Canvas",
            "sample.util.Box",
            "sample.util.Pair",
            "sample.util.Tagged"
        ]
    );
}

#[test]
fn a_type_that_changed_between_releases_reads_as_the_version_of_the_asked_release() {
    let old = reading_of(8);
    let newer = reading_of(10);

    let old_box = shape(&old, "sample.util.Box");
    let new_box = shape(&newer, "sample.util.Box");

    assert_eq!(method_names(old_box), ["get"]);
    assert_eq!(method_names(new_box), ["get", "isEmpty"]);
}

#[test]
fn a_type_added_in_a_release_is_missing_from_earlier_ones() {
    let ten = reading_of(10);
    let eleven = reading_of(11);

    assert_eq!(
        names(&ten),
        ["sample.util.Box", "sample.util.Pair", "sample.util.Tagged"]
    );
    assert_eq!(
        names(&eleven),
        [
            "sample.util.Box",
            "sample.util.Fresh",
            "sample.util.Pair",
            "sample.util.Tagged"
        ]
    );
    assert_eq!(
        shape(&eleven, "sample.util.Fresh").kind(),
        TypeKind::Interface
    );
}

#[test]
fn jdk_types_keep_their_generic_signatures() {
    let reading = reading_of(11);

    let pair = shape(&reading, "sample.util.Pair");

    assert_eq!(
        pair.methods()
            .iter()
            .map(|method| method.returns().cloned())
            .collect::<Vec<_>>(),
        [Some(TypeRef::variable("A")), Some(TypeRef::variable("B"))]
    );
}

#[test]
fn jdk_types_list_their_type_parameters_and_their_generic_superclass() {
    let reading = reading_of(11);

    let pair = shape(&reading, "sample.util.Pair");
    let tagged = shape(&reading, "sample.util.Tagged");

    assert_eq!(pair.type_parameters(), ["A", "B"]);
    assert_eq!(pair.superclass(), None);
    assert_eq!(tagged.type_parameters(), ["K", "V"]);
    assert_eq!(
        tagged.superclass(),
        Some(&TypeRef::generic(
            "sample.util.Pair",
            vec![TypeRef::variable("K"), TypeRef::variable("V")]
        ))
    );
}

#[test]
fn an_unreadable_sig_file_is_reported_and_the_other_types_still_read() {
    let reading = reading_of(11);

    let paths: Vec<&Path> = reading.errors.iter().map(Error::path).collect();

    assert_eq!(paths, [fixture_jdk().join("lib/ct.sym")]);
    assert!(reading.errors[0].to_string().contains("Corrupt.sig"));
    assert_eq!(reading.shapes.len(), 4);
}

#[test]
fn releases_without_damaged_files_report_no_errors() {
    let reading = reading_of(8);

    assert!(reading.errors.is_empty());
}

#[test]
fn a_release_newer_than_the_jdk_is_an_error_carrying_the_ct_sym_path() {
    let jdk = Jdk::at(fixture_jdk());

    let error = error_of(jdk.types(13));

    assert_eq!(error.path(), fixture_jdk().join("lib/ct.sym"));
    assert!(error.to_string().contains("newer"));
}

#[test]
fn a_release_ct_sym_knows_nothing_about_is_an_error() {
    let jdk = Jdk::at(fixture_jdk());

    let error = error_of(jdk.types(7));

    assert_eq!(error.path(), fixture_jdk().join("lib/ct.sym"));
    assert!(error.to_string().contains("7"));
}

#[test]
fn a_jdk_without_ct_sym_is_an_error_carrying_its_path() {
    let jdk = jdk_with_release("no-ct-sym", "JAVA_VERSION=\"12.0.1\"\n");

    let error = error_of(jdk.types(8));

    assert_eq!(error.path(), jdk.home().join("lib/ct.sym"));
}

#[test]
fn the_release_of_the_jdk_itself_comes_from_the_runtime_image() {
    let jdk = Jdk::at(fixture_jdk());

    let error = error_of(jdk.types(12));

    assert_eq!(error.path(), fixture_jdk().join("lib/modules"));
}

fn real_jdk() -> Option<Jdk> {
    let jdk = Jdk::locate();
    if jdk.is_none() {
        eprintln!("skipped: no JDK found; set JAVA_HOME or put javac on PATH");
    }
    jdk
}

#[test]
fn the_real_jdk_lists_its_own_release_from_the_runtime_image() {
    let Some(jdk) = real_jdk() else { return };
    let release = jdk.feature_version().unwrap();

    let reading = jdk.types(release).unwrap();

    let list = shape(&reading, "java.util.List");
    assert!(reading.errors.is_empty(), "{:?}", reading.errors);
    assert_eq!(list.kind(), TypeKind::Interface);
    assert!(method_names(list).contains(&"size"));
    assert!(
        shape(&reading, "java.util.Optional")
            .methods()
            .iter()
            .any(|method| method.name() == "isPresent")
    );
}

#[test]
fn the_real_jdk_keeps_internal_and_non_public_classes_out() {
    let Some(jdk) = real_jdk() else { return };
    let release = jdk.feature_version().unwrap();

    let reading = jdk.types(release).unwrap();

    let names = names(&reading);
    assert!(names.contains(&"java.lang.String"));
    assert!(names.contains(&"java.util.Map$Entry"));
    assert!(!names.contains(&"jdk.internal.misc.Unsafe"));
    assert!(!names.contains(&"java.lang.StringLatin1"));
}

#[test]
fn the_real_jdk_lists_an_earlier_release_from_ct_sym() {
    let Some(jdk) = real_jdk() else { return };
    let release = jdk.feature_version().unwrap() - 1;

    let reading = jdk.types(release).unwrap();

    assert!(reading.errors.is_empty(), "{:?}", reading.errors);
    assert!(names(&reading).contains(&"java.lang.String"));
    assert!(reading.shapes.len() > 4_000);
}

fn assert_hash_map_is_generic(reading: &Reading) {
    let hash_map = shape(reading, "java.util.HashMap");
    assert_eq!(hash_map.type_parameters(), ["K", "V"]);
    assert_eq!(
        hash_map.superclass(),
        Some(&TypeRef::generic(
            "java.util.AbstractMap",
            vec![TypeRef::variable("K"), TypeRef::variable("V")]
        ))
    );
    assert_eq!(shape(reading, "java.util.List").type_parameters(), ["E"]);
}

#[test]
fn the_real_jdk_runtime_image_types_carry_type_parameters_and_superclass() {
    let Some(jdk) = real_jdk() else { return };
    let release = jdk.feature_version().unwrap();

    let reading = jdk.types(release).unwrap();

    assert_hash_map_is_generic(&reading);
}

#[test]
fn the_real_jdk_ct_sym_types_carry_type_parameters_and_superclass() {
    let Some(jdk) = real_jdk() else { return };
    let release = jdk.feature_version().unwrap() - 1;

    let reading = jdk.types(release).unwrap();

    assert_hash_map_is_generic(&reading);
}
