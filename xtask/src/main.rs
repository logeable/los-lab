use std::path::PathBuf;

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
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
        },
    }

    Ok(())
}
