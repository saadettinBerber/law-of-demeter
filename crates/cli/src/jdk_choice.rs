use std::ffi::OsString;

use law_of_demeter_class_file::Jdk;

use crate::arguments::{Arguments, UsageError};

pub fn choose_jdk(
    arguments: &Arguments,
    java_home: Option<OsString>,
    path: Option<OsString>,
) -> Result<(Jdk, u32), UsageError> {
    let jdk = match &arguments.jdk_home {
        Some(home) => Jdk::at(home),
        None => Jdk::locate_in(java_home, path).ok_or_else(|| {
            UsageError("JDK bulunamadı: --jdk-home verin ya da JAVA_HOME ayarlayın".to_owned())
        })?,
    };
    let release = match arguments.release {
        Some(release) => release,
        None => jdk
            .feature_version()
            .map_err(|error| UsageError(format!("JDK sürümü okunamadı: {error}")))?,
    };
    Ok((jdk, release))
}

#[cfg(test)]
mod tests;
