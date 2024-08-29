use std::{fs::File, io::Write, path::PathBuf, process::Command};

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use minijinja::{context, Environment};
use regex::Regex;
use xtask::{app, get_bin_targets, get_workspace_folder, objcopy_to_bin};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    App(AppCommands),
}

#[derive(Subcommand)]
enum AppCommands {
    Asm,
    Build,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::App(app_command) => match app_command {
            AppCommands::Asm => {
                let targets = get_bin_targets("los-user").context("get bin targets failed")?;
                let workspace = get_workspace_folder();
                let bin_dir = PathBuf::new()
                    .join(workspace)
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
                    objcopy_to_bin(bin_path.to_str().unwrap(), dest)
                        .context("objcopy to bin failed")?;

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

                let dest = PathBuf::new().join(bin_dir).join("app.asm");
                let dest = dest.to_str().unwrap();

                println!("bins: {:?}", bins);
                app::gen_app_asm(bins, dest).context("gen app asm failed")?;
            }
            AppCommands::Build => {
                let targets = get_bin_targets("los-user").context("get bin targets failed")?;

                let mut env = Environment::new();
                env.add_template("linker.ld", include_str!("linker.ld.tmpl"))
                    .context("add template failed")?;
                let tmpl = env.get_template("linker.ld").unwrap();

                let base_address = 0x80400000;
                let step = 0x20000;

                let los_user_dir = PathBuf::from(get_workspace_folder()).join("los-user");
                let linker_ld_script_path = los_user_dir.join("src").join("linker.ld");
                for (i, target) in targets.iter().enumerate() {
                    let address = format!("{:#x}", base_address + step * i);
                    let content = tmpl.render(context!(address)).unwrap();

                    File::create(&linker_ld_script_path)
                        .unwrap()
                        .write_all(content.as_bytes())
                        .unwrap();
                    let output = Command::new("cargo")
                        .arg("build")
                        .arg("--release")
                        .arg("--bin")
                        .arg(target)
                        .current_dir(&los_user_dir)
                        .output()
                        .unwrap();
                    if !output.status.success() {
                        panic!(
                            "build {} failed: {}",
                            target,
                            String::from_utf8(output.stderr).unwrap()
                        );
                    }
                    println!("build {} done", target);
                }
            }
        },
    }

    Ok(())
}
