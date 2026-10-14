use clap::Parser;
use mybible::cli::{Cli, CliRunner};

fn main() {
    let cli = Cli::parse();
    match CliRunner::new(cli) {
        Ok(runner) => {
            if let Err(e) = runner.run() {
                eprintln!("\nError: {}\n", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("\nInitialization Error: {}\n", e);
            std::process::exit(1);
        }
    }
}
