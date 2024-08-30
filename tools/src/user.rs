use anyhow::{bail, Context};
use minijinja::{context, Environment};
use regex::Regex;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
};

use crate::{AsmArgs, BuildArgs};

pub fn build(asm_args: &BuildArgs) -> anyhow::Result<()> {
    let user_path = &asm_args.user_args.user_crate_dir;

    let targets = get_bin_targets(user_path).context("get bin targets failed")?;

    let mut env = Environment::new();
    env.add_template("linker.ld", include_str!("linker.ld.tmpl"))
        .context("add template failed")?;
    let tmpl = env.get_template("linker.ld").unwrap();

    let base_address = 0x80400000;
    let step = 0x20000;

    let user_path = PathBuf::from(user_path);
    let linker_ld_script_path = user_path.join("src").join("linker.ld");
    for (i, target) in targets.iter().enumerate() {
        let address = format!("{:#x}", base_address + step * i);
        let content = tmpl.render(context!(address)).context("render failed")?;

        File::create(&linker_ld_script_path)
            .unwrap()
            .write_all(content.as_bytes())
            .context("write linker.ld failed")?;
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--bin")
            .arg(target)
            .current_dir(&user_path)
            .output()
            .context("build failed")?;
        if !output.status.success() {
            bail!(
                "build {:?} failed: {}",
                target,
                String::from_utf8(output.stderr).unwrap()
            );
        }
        println!("build {} done", target);
    }

    Ok(())
}

pub fn asm(asm_args: &AsmArgs) -> anyhow::Result<()> {
    let user_path = &asm_args.user_args.user_crate_dir;

    let targets = get_bin_targets(user_path).context("get bin targets failed")?;
    let bin_dir = PathBuf::new()
        .join(user_path)
        .join("target")
        .join("riscv64gc-unknown-none-elf")
        .join("release");

    let bin_name_regex = Regex::new(r"^\d+_.*").unwrap();
    let mut bins = Vec::new();
    for target in targets {
        let bin_path = bin_dir.join(target);
        if !bin_path.exists() {
            bail!("{:?} not exists", bin_path);
        }
        let dest_path = bin_path.with_extension("bin");
        let dest = dest_path.to_str().unwrap();
        objcopy_to_bin(bin_path.to_str().unwrap(), dest).context("objcopy to bin failed")?;

        bins.push(dest_path);
    }

    let mut bins: Vec<_> = bins
        .iter()
        .filter(|p| bin_name_regex.is_match(p.file_name().unwrap().to_str().unwrap()))
        .collect();
    bins.sort();
    let bins = bins
        .into_iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();

    gen_app_asm(bins, &asm_args.app_asm_path).context("gen app asm failed")?;

    Ok(())
}

fn objcopy_to_bin(src: &str, dest: &str) -> anyhow::Result<()> {
    let output = Command::new("riscv64-unknown-elf-objcopy")
        .arg("--strip-all")
        .arg(src)
        .arg("-O")
        .arg("binary")
        .arg(dest)
        .output()
        .context("execute command failed")?;

    if !output.status.success() {
        bail!("status is not ok");
    }

    Ok(())
}

fn get_bin_targets(user_path: &str) -> anyhow::Result<Vec<String>> {
    let cargo_toml_path = PathBuf::from(user_path).join("Cargo.toml");
    let cmd = format!(
        r#"cargo read-manifest --manifest-path {} | jq -r '.targets[] | select(.kind[] == "bin") | .name'"#,
        cargo_toml_path.to_str().unwrap()
    );
    let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
    if !output.status.success() {
        bail!(
            "get bin targets failed: {}",
            String::from_utf8(output.stderr).unwrap()
        );
    }
    let targets = String::from_utf8(output.stdout).unwrap();
    let targets = targets
        .split("\n")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(targets)
}

fn gen_app_asm(bins: Vec<String>, dest: &str) -> anyhow::Result<()> {
    let file = File::create(dest).context("create file failed")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "\t.section .data")?;
    writeln!(writer, "\t.globl app_data")?;
    writeln!(writer, "app_data:")?;
    writeln!(writer, "\t.quad {}", bins.len())?;

    for i in 0..bins.len() {
        writeln!(writer, "\t.quad app_{}_start", i)?;
        writeln!(writer, "\t.quad app_{}_end", i)?;
        writeln!(writer, "\t.quad app_{}_name", i)?;
        writeln!(writer, "\t.quad app_{}_entry", i)?;
    }

    let base_address = 0x80400000;
    let step = 0x20000;

    for (i, bin) in bins.iter().enumerate() {
        let name = Path::new(bin).file_name().unwrap().to_str().unwrap();
        let address = format!("{:#x}", base_address + step * i);
        writeln!(writer, "app_{}_start:", i)?;
        writeln!(writer, "\t.incbin \"{}\"", bin)?;
        writeln!(writer, "app_{}_end:", i)?;
        writeln!(writer, "app_{}_name:", i)?;
        writeln!(writer, "\t.string \"{}\"", name)?;
        writeln!(writer, "app_{}_entry:", i)?;
        writeln!(writer, "\t.quad {}", address)?;
    }

    writer.flush().context("flush failed")?;

    Ok(())
}
