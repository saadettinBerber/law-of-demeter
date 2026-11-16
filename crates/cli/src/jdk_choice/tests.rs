use std::ffi::OsString;
use std::path::PathBuf;

use super::choose_jdk;
use crate::arguments::Arguments;

fn fixture_jdk() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../class-file/tests/fixtures/jdk")
}

#[test]
fn an_explicit_jdk_home_is_used_with_its_own_release_by_default() {
    let arguments = Arguments {
        jdk_home: Some(fixture_jdk()),
        ..Arguments::default()
    };

    let (jdk, release) = choose_jdk(&arguments, None, None).expect("jdk is chosen");

    assert_eq!((jdk.home().to_path_buf(), release), (fixture_jdk(), 12));
}

#[test]
fn an_explicit_release_wins_over_the_jdks_own() {
    let arguments = Arguments {
        jdk_home: Some(fixture_jdk()),
        release: Some(8),
        ..Arguments::default()
    };

    let (_, release) = choose_jdk(&arguments, None, None).expect("jdk is chosen");

    assert_eq!(release, 8);
}

#[test]
fn without_an_explicit_home_java_home_is_used() {
    let java_home = OsString::from(fixture_jdk());

    let (jdk, _) = choose_jdk(&Arguments::default(), Some(java_home), None).expect("jdk is chosen");

    assert_eq!(jdk.home(), fixture_jdk());
}

#[test]
fn a_missing_jdk_is_a_usage_error() {
    let error = choose_jdk(&Arguments::default(), None, None).err();

    assert_eq!(
        error.map(|error| error.0),
        Some("no JDK found: pass --jdk-home or set JAVA_HOME".to_owned())
    );
}

#[test]
fn a_jdk_home_without_a_release_file_is_a_usage_error() {
    let arguments = Arguments {
        jdk_home: Some(PathBuf::from("/nonexistent/jdk")),
        ..Arguments::default()
    };

    let error = choose_jdk(&arguments, None, None).err();

    assert!(error.map(|error| error.0).is_some_and(|message| {
        message.starts_with("cannot read the JDK version: /nonexistent/jdk/release")
    }));
}
