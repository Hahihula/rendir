use anyhow::Result;
use clap::{Parser, Subcommand};
use rustpress_core::components::{ComponentRegistry, builtins::register_builtin_components};
use rustpress_core::render::render_with_template;
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

        /// Custom HTML template file
        #[arg(short, long)]
        template: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create component registry with built-in components
    let mut registry = ComponentRegistry::new();
    register_builtin_components(&mut registry);

    match &cli.command {
        Commands::Convert {
            input,
            output,
            template,
        } => {
            let content = fs::read_to_string(input)?;
            let item = parse_markdown(&content, Some(&registry));

            let html = if let Some(template_path) = template {
                let template_name = template_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let template_content = fs::read_to_string(template_path)?;
                render_with_template(&item, &template_name, &template_content)
            } else {
                render_html(&item)
            };

            match output {
                Some(path) => fs::write(path, html)?,
                None => println!("{}", html),
            }
        }
    }

    Ok(())
}
