use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Arguments {
    pub sources: Vec<PathBuf>,
    pub classpath: Vec<PathBuf>,
    pub jdk_home: Option<PathBuf>,
    pub release: Option<u32>,
    pub show_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Check(Arguments),
    Help,
}

pub fn usage() -> &'static str {
    "Kullanım: law-of-demeter --source <DİZİN>... [SEÇENEKLER]

Java kaynağında Law of Demeter ihlallerini bulur.

Seçenekler:
  -s, --source <DİZİN>      Kaynak kökü; birden çok kez verilebilir (zorunlu)
  -c, --classpath <YOL>     Dizin ya da jar; birden çok kez ya da ':' ile ayrılmış liste verilebilir
      --jdk-home <DİZİN>    JDK dizini (varsayılan: JAVA_HOME, sonra PATH'teki javac)
      --release <SÜRÜM>     Okunacak Java sürümü (varsayılan: JDK'nın kendi sürümü)
      --show-allowed        Uygun çağrıları da uydukları kuralla birlikte listeler
  -h, --help                Bu yardımı gösterir

Çıkış kodu: 0 ihlal yok, 1 ihlal var, 2 kullanım ya da ölümcül hata.
"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError(pub String);

pub fn parse<I: IntoIterator<Item = OsString>>(arguments: I) -> Result<Command, UsageError> {
    let mut parsed = Arguments::default();
    let mut arguments = arguments.into_iter().flat_map(split_at_equals);
    while let Some(argument) = arguments.next() {
        let option = argument.to_string_lossy();
        match option.as_ref() {
            "--help" | "-h" => return Ok(Command::Help),
            "--source" | "-s" => parsed.sources.push(value(&option, &mut arguments)?.into()),
            "--classpath" | "-c" => parsed.classpath.extend(
                env::split_paths(&value(&option, &mut arguments)?)
                    .filter(|entry| !entry.as_os_str().is_empty()),
            ),
            "--jdk-home" => parsed.jdk_home = Some(value(&option, &mut arguments)?.into()),
            "--release" => parsed.release = Some(release(&value(&option, &mut arguments)?)?),
            "--show-allowed" => parsed.show_allowed = true,
            _ if option.starts_with('-') => {
                return Err(UsageError(format!("bilinmeyen seçenek: {option}")));
            }
            _ => return Err(UsageError(format!("beklenmeyen argüman: {option}"))),
        }
    }
    if parsed.sources.is_empty() {
        return Err(UsageError("en az bir --source gerekli".to_owned()));
    }
    Ok(Command::Check(parsed))
}

fn split_at_equals(argument: OsString) -> Vec<OsString> {
    let split = argument
        .to_str()
        .filter(|text| text.starts_with("--"))
        .and_then(|text| text.split_once('='));
    match split {
        Some((option, value)) => vec![option.into(), value.into()],
        None => vec![argument],
    }
}

fn value(
    option: &str,
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<OsString, UsageError> {
    arguments
        .next()
        .ok_or_else(|| UsageError(format!("{option} için değer gerekli")))
}

fn release(value: &OsString) -> Result<u32, UsageError> {
    value
        .to_str()
        .and_then(|text| text.parse().ok())
        .ok_or_else(|| {
            UsageError(format!(
                "--release bir sürüm numarası olmalı: {}",
                value.to_string_lossy()
            ))
        })
}

#[cfg(test)]
mod tests;
