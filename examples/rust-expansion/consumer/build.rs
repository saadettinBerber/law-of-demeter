use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build-input.txt");
    println!("cargo::rerun-if-changed=invalid-generated.rs");
    println!("cargo::rerun-if-env-changed=LOD_FIXTURE_MODE");
    println!("cargo::rerun-if-env-changed=LOD_FIXTURE_BROKEN");
    println!("cargo::rustc-check-cfg=cfg(build_bonus)");

    let mode = env::var("LOD_FIXTURE_MODE")?;
    if mode != "base" && mode != "bonus" {
        return Err("LOD_FIXTURE_MODE must be base or bonus".into());
    }
    let input: i32 = fs::read_to_string("build-input.txt")?.trim().parse()?;
    let amount = if mode == "bonus" {
        println!("cargo::rustc-cfg=build_bonus");
        input + 4
    } else {
        input
    };
    let source = if env::var("LOD_FIXTURE_BROKEN").as_deref() == Ok("1") {
        fs::read_to_string("invalid-generated.rs")?
    } else {
        format!("fn generated_charge(wallet: &Wallet) {{ wallet.charge({amount}); }}\n")
    };
    let output = PathBuf::from(env::var("OUT_DIR")?);
    fs::write(output.join("generated.rs"), source)?;
    let mut runs = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output.join("build-runs.log"))?;
    writeln!(runs, "{mode}")?;
    Ok(())
}
