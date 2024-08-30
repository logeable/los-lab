use std::{env, error::Error, fs::File, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out_dir.display());

    File::create(out_dir.join("linker.ld"))?.write_all(include_bytes!("linker.ld"))?;

    Ok(())
}
