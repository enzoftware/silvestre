use clap::{Parser, Subcommand};
use thiserror::Error;

mod commands;

use commands::{apply::ApplyCommand, info::InfoCommand, list::ListCommand};

#[derive(Parser)]
#[command(name = "silvestre")]
#[command(about = "Cross-platform image processing CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply a filter to an image
    Apply(ApplyCommand),
    /// List available filters
    List,
    /// Show image information and metadata
    Info(InfoCommand),
}

#[derive(Error, Debug)]
enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Silvestre error: {0}")]
    Silvestre(#[from] silvestre_core::SilvestreError),

    #[error("{0}")]
    Custom(String),
}

type Result<T> = std::result::Result<T, CliError>;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Apply(cmd)) => cmd.execute(),
        Some(Commands::List) => ListCommand::execute(),
        Some(Commands::Info(cmd)) => cmd.execute(),
        None => {
            println!("silvestre v{}", env!("CARGO_PKG_VERSION"));
            println!("Cross-platform image processing CLI");
            println!();
            println!("Usage: silvestre <COMMAND>");
            println!();
            println!("Commands:");
            println!("  apply   Apply a filter to an image");
            println!("  list    List available filters");
            println!("  info    Show image information");
            println!("  help    Print this message or the help of the given subcommand(s)");
            println!();
            println!("Run 'silvestre --help' for more information.");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
