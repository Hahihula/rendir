use anyhow::Result;
use clap::{Parser, Subcommand};
use rustpress_core::{parse_markdown, render_html};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a markdown file to HTML
    Convert {
        /// Input markdown file
        #[arg(short, long)]
        input: PathBuf,

        /// Output HTML file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Convert { input, output } => {
            let content = fs::read_to_string(input)?;
            let item = parse_markdown(&content);
            let html = render_html(&item);

            match output {
                Some(path) => fs::write(path, html)?,
                None => println!("{}", html),
            }
        }
    }

    Ok(())
}
