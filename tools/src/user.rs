use anyhow::{bail, Context};
use minijinja::{context, Environment, UndefinedBehavior};
use regex::Regex;
use serde::Serialize;
use std::{fs::File, path::PathBuf, process::Command};
use uuid::Uuid;

pub fn build(user_path: &str, release: bool) -> anyhow::Result<()> {
    let targets = get_bin_targets(user_path).context("get bin targets failed")?;

    let user_path = PathBuf::from(user_path);
    for target in targets.iter() {
        let mut cmd = Command::new("cargo");
        let cmd = cmd.arg("build").arg("--bin").arg(target);
        if release {
            cmd.arg("--release");
        }
        let output = cmd
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

pub fn asm(user_path: &str, app_asm_path: &str, release: bool) -> anyhow::Result<()> {
    let profile = if release { "release" } else { "debug" };

    let targets = get_bin_targets(user_path).context("get bin targets failed")?;
    let bin_dir = PathBuf::new()
        .join(user_path)
        .join("target")
        .join("riscv64gc-unknown-none-elf")
        .join(profile);

    let bin_name_regex = Regex::new(r"^(?<ID>\d+)_.*").unwrap();
    let mut bins = Vec::new();
    for target in targets {
        let bin_path = bin_dir.join(target);
        if !bin_path.exists() {
            bail!("{:?} not exists", bin_path);
        }

        bins.push(bin_path);
    }

    let mut bins: Vec<_> = bins
        .iter()
        .filter(|p| bin_name_regex.is_match(p.file_name().unwrap().to_str().unwrap()))
        .collect();

    bins.sort_by(|a, b| {
        let a = a.file_name().unwrap().to_str().unwrap();
        let b = b.file_name().unwrap().to_str().unwrap();
        let id_a: i32 = bin_name_regex.captures(a).unwrap()["ID"].parse().unwrap();
        let id_b: i32 = bin_name_regex.captures(b).unwrap()["ID"].parse().unwrap();

        id_a.cmp(&id_b)
    });

    let bins: Vec<String> = bins
        .into_iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();

    for name in bins.iter() {
        println!("{:?}", name);
    }

    gen_app_asm(bins, app_asm_path).context("gen app asm failed")?;

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
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.add_template("app.asm", include_str!("./app.asm.tmpl"))
        .context("add template failed")?;
    let tmpl = env.get_template("app.asm").unwrap();

    let apps: Vec<_> = bins
        .iter()
        .map(|bin| {
            let name = PathBuf::from(&bin)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap()
                .to_string();
            let bin_path = bin.clone();
            AppInfo { name, bin_path }
        })
        .collect();

    let ctx = context! {
        apps,
        number_of_apps => apps.len(),
        uuid=> Uuid::new_v4().to_string()
    };

    let file = File::create(dest).context("create file failed")?;
    tmpl.render_to_write(ctx, file)
        .context("render to file failed")?;

    Ok(())
}

#[derive(Serialize)]
struct AppInfo {
    name: String,
    bin_path: String,
}
