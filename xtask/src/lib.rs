pub mod app;

use std::{
    path::{self, PathBuf},
    process::Command,
};

use anyhow::bail;
use serde::Deserialize;

pub fn get_workspace_folder() -> String {
    let output = String::from_utf8(
        Command::new("cargo")
            .args(["locate-project", "--workspace", "--message-format=plain"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    path::Path::new(&output)
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[derive(Debug, PartialEq)]
enum TargetKind {
    Bin,
    Lib,
}

#[derive(Debug)]
pub struct Target {
    kind: TargetKind,
    name: String,
}

pub fn get_targets(crate_name: &str) -> anyhow::Result<Vec<Target>> {
    let workspace = get_workspace_folder();
    let manifest_path = PathBuf::new()
        .join(workspace)
        .join(crate_name)
        .join("Cargo.toml");

    let output = Command::new("cargo")
        .args(["read-manifest", "--manifest-path"])
        .arg(manifest_path.to_str().unwrap())
        .output()
        .unwrap()
        .stdout;

    let manifest: Manifest = serde_json::from_slice(output.as_slice()).unwrap();

    let mut targets = Vec::new();

    for t in manifest.targets {
        let kind = match t.kind[0].as_str() {
            "bin" => TargetKind::Bin,
            "lib" => TargetKind::Lib,
            _ => bail!("unknown kind: {}", t.kind[0]),
        };
        targets.push(Target { kind, name: t.name })
    }
    Ok(targets)
}

pub fn get_bin_targets(crate_name: &str) -> anyhow::Result<Vec<String>> {
    let mut targets: Vec<_> = get_targets(crate_name)?
        .iter()
        .filter(|v| v.kind == TargetKind::Bin)
        .map(|v| v.name.clone())
        .collect();
    targets.sort();
    Ok(targets)
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
struct Manifest {
    targets: Vec<TargetInner>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
struct TargetInner {
    kind: Vec<String>,
    name: String,
}

pub fn objcopy_to_bin(elf_path: &str, dest_path: &str) -> anyhow::Result<()> {
    let output = Command::new("riscv64-unknown-elf-objcopy")
        .arg("--strip-all")
        .arg(elf_path)
        .arg("-O")
        .arg("binary")
        .arg(dest_path)
        .output()
        .unwrap();
    if !output.status.success() {
        bail!(
            "objcopy failed: {}",
            String::from_utf8(output.stderr).unwrap()
        );
    }

    Ok(())
}
