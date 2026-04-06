use clap::Parser;

#[derive(Parser)]
#[command(name = "silvestre", about = "Cross-platform image processing CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// List available filters
    List,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => {
            println!("Available filters: (none yet — coming soon)");
        }
        None => {
            println!("silvestre v{}", env!("CARGO_PKG_VERSION"));
            println!("Run with --help for usage information.");
        }
    }
}
