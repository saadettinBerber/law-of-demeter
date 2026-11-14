use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use cafebabe::attributes::AttributeData;
use cafebabe::parse_class;
use ristretto_jimage::Image;
use zip::ZipArchive;

fn real_jdk() -> Option<PathBuf> {
    let home = java_home_variable().or_else(javac_on_path);
    if home.is_none() {
        eprintln!("skipped: no JDK found; set JAVA_HOME or put javac on PATH");
    }
    home
}

fn java_home_variable() -> Option<PathBuf> {
    let home = PathBuf::from(env::var_os("JAVA_HOME")?);
    home.join("lib").is_dir().then_some(home)
}

fn javac_on_path() -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|directory| directory.join("javac"))
        .find_map(|javac| fs::canonicalize(javac).ok())
        .and_then(|javac| Some(javac.parent()?.parent()?.to_path_buf()))
}

fn feature_version(home: &Path) -> u32 {
    let release = fs::read_to_string(home.join("release")).unwrap();
    let version = release
        .lines()
        .find_map(|line| line.strip_prefix("JAVA_VERSION="))
        .unwrap()
        .trim_matches('"');
    version.split('.').next().unwrap().parse().unwrap()
}

fn release_char(release: u32) -> char {
    char::from_digit(release, 36).unwrap().to_ascii_uppercase()
}

fn ct_sym(home: &Path) -> ZipArchive<File> {
    ZipArchive::new(File::open(home.join("lib/ct.sym")).unwrap()).unwrap()
}

fn ct_sym_entry_names(home: &Path) -> Vec<String> {
    ct_sym(home)
        .file_names()
        .map(|name| name.unwrap().into_owned())
        .collect()
}

fn read_entry(home: &Path, name: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    ct_sym(home)
        .by_name(name)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

#[test]
fn the_release_file_names_the_feature_version_of_the_jdk() {
    let Some(home) = real_jdk() else { return };

    let version = feature_version(&home);

    assert!(version >= 9, "found feature version {version}");
}

#[test]
fn ct_sym_keeps_sig_files_under_a_release_directory_and_a_module_directory() {
    let Some(home) = real_jdk() else { return };

    let names = ct_sym_entry_names(&home);

    let object_paths: Vec<&String> = names
        .iter()
        .filter(|name| name.ends_with("/java.base/java/lang/Object.sig"))
        .collect();
    assert!(!object_paths.is_empty());
    for path in object_paths {
        let release_directory = path.split('/').next().unwrap();
        assert!(
            release_directory
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
        );
    }
}

#[test]
fn a_release_directory_name_lists_the_releases_that_share_the_same_type() {
    let Some(home) = real_jdk() else { return };
    let feature = feature_version(&home);

    let mut owner_of: BTreeMap<char, Vec<String>> = BTreeMap::new();
    for name in ct_sym_entry_names(&home) {
        if let Some(directory) = name.strip_suffix("/java.base/java/lang/Object.sig") {
            for release in directory.chars() {
                owner_of.entry(release).or_default().push(name.clone());
            }
        }
    }

    let covered: BTreeSet<char> = owner_of.keys().copied().collect();
    let expected: BTreeSet<char> = (8..feature).map(release_char).collect();
    assert_eq!(covered, expected);
    assert!(owner_of.values().all(|paths| paths.len() == 1));
}

#[test]
fn ct_sym_has_no_sig_files_for_the_release_of_the_jdk_itself() {
    let Some(home) = real_jdk() else { return };
    let own = release_char(feature_version(&home));

    let names = ct_sym_entry_names(&home);

    let own_sigs = names
        .iter()
        .filter(|name| name.ends_with(".sig"))
        .filter(|name| name.split('/').next().unwrap().contains(own))
        .count();
    assert_eq!(own_sigs, 0);
    assert!(names.contains(&format!("{own}/system-modules")));
}

#[test]
fn a_sig_file_is_a_class_file_with_generic_signatures() {
    let Some(home) = real_jdk() else { return };
    let previous = release_char(feature_version(&home) - 1);
    let name = ct_sym_entry_names(&home)
        .into_iter()
        .find(|name| {
            name.ends_with("/java.base/java/util/Optional.sig")
                && name.split('/').next().unwrap().contains(previous)
        })
        .unwrap();

    let bytes = read_entry(&home, &name);
    let class = parse_class(&bytes).unwrap();

    assert_eq!(&*class.this_class, "java/util/Optional");
    assert!(class.methods.iter().any(|method| method.name == "isEmpty"));
    assert!(
        class
            .attributes
            .iter()
            .any(|attribute| matches!(attribute.data, AttributeData::Signature(_)))
    );
}

#[test]
fn the_runtime_image_holds_the_classes_of_the_jdk_itself() {
    let Some(home) = real_jdk() else { return };
    let image = Image::from_file(&home.join("lib/modules")).unwrap();

    let resource = image
        .get_resource("/java.base/java/lang/String.class")
        .unwrap();
    let class = parse_class(resource.data()).unwrap();

    assert_eq!(&*class.this_class, "java/lang/String");
    assert_eq!(resource.module(), "java.base");
    assert_eq!(resource.parent(), "java/lang");
    assert_eq!(resource.base(), "String");
    assert_eq!(resource.extension(), "class");
}

#[test]
fn the_runtime_image_lists_every_resource_and_module_info_names_the_exports() {
    let Some(home) = real_jdk() else { return };
    let image = Image::from_file(&home.join("lib/modules")).unwrap();

    let class_count = image
        .iter()
        .map(|resource| resource.unwrap())
        .filter(|resource| resource.extension() == "class" && !resource.module().is_empty())
        .count();
    let module_info = image.get_resource("/java.base/module-info.class").unwrap();
    let class = parse_class(module_info.data()).unwrap();

    assert!(class_count > 10_000, "found {class_count} classes");
    let exported: Vec<String> = class
        .attributes
        .iter()
        .filter_map(|attribute| match &attribute.data {
            AttributeData::Module(module) => Some(module),
            _ => None,
        })
        .flat_map(|module| module.exports.iter())
        .filter(|export| export.exports_to.is_empty())
        .map(|export| export.package_name.to_string())
        .collect();
    assert!(exported.contains(&"java/lang".to_owned()));
    assert!(!exported.contains(&"jdk/internal/misc".to_owned()));
}
