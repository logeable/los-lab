use std::{path::PathBuf, process::Command};

use anyhow::{anyhow, bail, Context};
use clap::{Parser, Subcommand};
use xtask::{app, get_bin_targets, get_targets, get_workspace_folder, objcopy_to_bin};

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

                let mut bins = Vec::new();
                for target in targets {
                    let bin_path = bin_dir.join(target);
                    if !bin_path.exists() {
                        bail!("{:?} not exists", bin_path);
                    }
                    let dest = bin_path.with_extension("bin");
                    let dest = dest.to_str().unwrap();
                    objcopy_to_bin(bin_path.to_str().unwrap(), dest)
                        .context("objcopy to bin failed")?;

                    bins.push(dest.to_string());
                }

                let dest = PathBuf::new().join(bin_dir).join("app.asm");
                let dest = dest.to_str().unwrap();
                app::gen_app_asm(bins, dest).context("gen app asm failed")?;
            }
        },
    }

    Ok(())
}
