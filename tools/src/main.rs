mod user;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    User(UserCommands),
}

#[derive(Subcommand)]
enum UserCommands {
    Asm(AsmArgs),
    Build(BuildArgs),
}

#[derive(Args)]
struct AsmArgs {
    #[command(flatten)]
    pub user_args: UserArgs,
    pub app_asm_path: String,
}

#[derive(Args)]
struct BuildArgs {
    #[command(flatten)]
    user_args: UserArgs,
}

#[derive(Args)]
struct UserArgs {
    user_crate_dir: String,
    #[arg(long)]
    release: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::User(app_command) => match app_command {
            UserCommands::Asm(arg) => {
                user::asm(&arg).context("user asm failed")?;
            }
            UserCommands::Build(arg) => {
                user::build(&arg).context("user build failed")?;
            }
        },
    }

    Ok(())
}
