use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use cafebabe::attributes::AttributeData;
use cafebabe::parse_class;
use law_of_demeter_core::domain::type_shape::TypeShape;
use ristretto_jimage::Image;
use zip::ZipArchive;

use crate::shape::{self, Scope};
use crate::source::is_module_descriptor;
use crate::{Error, Reading};

pub struct Jdk {
    home: PathBuf,
}

impl Jdk {
    pub fn at(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }

    pub fn locate() -> Option<Self> {
        Self::locate_in(env::var_os("JAVA_HOME"), env::var_os("PATH"))
    }

    pub fn locate_in(java_home: Option<OsString>, path: Option<OsString>) -> Option<Self> {
        java_home
            .map(PathBuf::from)
            .filter(|home| is_jdk(home))
            .or_else(|| path.and_then(|path| javac_home(&path)))
            .map(Self::at)
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn feature_version(&self) -> Result<u32, Error> {
        let path = self.home.join("release");
        let text = fs::read_to_string(&path).map_err(|error| Error::new(&path, error))?;
        let version = text
            .lines()
            .find_map(|line| line.strip_prefix("JAVA_VERSION="))
            .map(|version| version.trim().trim_matches('"'))
            .ok_or_else(|| Error::new(&path, "no JAVA_VERSION"))?;
        parse_feature_version(version)
            .ok_or_else(|| Error::new(&path, format!("unreadable JAVA_VERSION '{version}'")))
    }

    pub fn types(&self, release: u32) -> Result<Reading, Error> {
        let own = self.feature_version()?;
        if release > own {
            return Err(Error::new(
                &self.ct_sym(),
                format!("release {release} is newer than the JDK's own release {own}"),
            ));
        }
        let mut reading = Reading::default();
        if release == own {
            self.read_image(&mut reading)?;
        } else {
            self.read_ct_sym(release, &mut reading)?;
        }
        reading
            .shapes
            .sort_by(|left, right| left.name().cmp(right.name()));
        Ok(reading)
    }

    fn ct_sym(&self) -> PathBuf {
        self.home.join("lib/ct.sym")
    }

    fn read_ct_sym(&self, release: u32, reading: &mut Reading) -> Result<(), Error> {
        let path = self.ct_sym();
        let marker = release_marker(release)
            .ok_or_else(|| Error::new(&path, format!("release {release} has no name in ct.sym")))?;
        let file = File::open(&path).map_err(|error| Error::new(&path, error))?;
        let mut archive = ZipArchive::new(file).map_err(|error| Error::new(&path, error))?;
        for index in 0..archive.len() {
            reading.record(read_sig_entry(&mut archive, index, marker, &path));
        }
        if reading.shapes.is_empty() && reading.errors.is_empty() {
            return Err(Error::new(&path, format!("no types for release {release}")));
        }
        Ok(())
    }

    fn read_image(&self, reading: &mut Reading) -> Result<(), Error> {
        let path = self.home.join("lib/modules");
        let image = Image::from_file(&path).map_err(|error| Error::new(&path, error))?;
        let mut exports: HashMap<String, HashSet<String>> = HashMap::new();
        for resource in image.iter() {
            let resource = match resource {
                Ok(resource) => resource,
                Err(error) => {
                    reading.errors.push(Error::new(&path, error));
                    continue;
                }
            };
            if resource.extension() == "class"
                && resource.base() == "module-info"
                && resource.parent().is_empty()
            {
                match exported_packages(resource.data()) {
                    Ok(packages) => {
                        exports.insert(resource.module().to_owned(), packages);
                    }
                    Err(message) => reading.errors.push(Error::new(
                        &path,
                        format!("{}: {message}", resource.full_name()),
                    )),
                }
            }
        }
        for resource in image.iter().flatten() {
            let exported = exports
                .get(resource.module())
                .is_some_and(|packages| packages.contains(resource.parent()));
            if resource.extension() == "class" && exported {
                reading.record(
                    shape::from_bytes(resource.data(), Scope::PublicApi).map_err(|message| {
                        Error::new(&path, format!("{}: {message}", resource.full_name()))
                    }),
                );
            }
        }
        Ok(())
    }
}

fn is_jdk(home: &Path) -> bool {
    home.join("release").is_file()
}

fn javac_home(path: &OsStr) -> Option<PathBuf> {
    env::split_paths(path).find_map(|directory| {
        let javac = fs::canonicalize(directory.join("javac")).ok()?;
        let home = javac.parent()?.parent()?.to_path_buf();
        is_jdk(&home).then_some(home)
    })
}

fn parse_feature_version(version: &str) -> Option<u32> {
    let mut parts = version.split('.');
    let first = parts.next()?;
    let feature = if first == "1" { parts.next()? } else { first };
    let digits: String = feature.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn release_marker(release: u32) -> Option<char> {
    char::from_digit(release, 36).map(|digit| digit.to_ascii_uppercase())
}

fn read_sig_entry(
    archive: &mut ZipArchive<File>,
    index: usize,
    marker: char,
    path: &Path,
) -> Result<Option<TypeShape>, Error> {
    let mut entry = archive
        .by_index(index)
        .map_err(|error| Error::new(path, error))?;
    let name = entry
        .name()
        .map_err(|error| Error::new(path, error))?
        .into_owned();
    let in_release = name.ends_with(".sig")
        && !is_module_descriptor(&name)
        && name
            .split('/')
            .next()
            .is_some_and(|directory| directory.contains(marker));
    if !in_release {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|error| Error::new(path, format!("{name}: {error}")))?;
    shape::from_bytes(&bytes, Scope::PublicApi)
        .map_err(|message| Error::new(path, format!("{name}: {message}")))
}

fn exported_packages(bytes: &[u8]) -> Result<HashSet<String>, String> {
    let class = parse_class(bytes).map_err(|error| error.to_string())?;
    Ok(class
        .attributes
        .iter()
        .filter_map(|attribute| match &attribute.data {
            AttributeData::Module(module) => Some(module),
            _ => None,
        })
        .flat_map(|module| module.exports.iter())
        .filter(|export| export.exports_to.is_empty())
        .map(|export| export.package_name.to_string())
        .collect())
}
