use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rustpress_core::components::{ComponentRegistry, builtins::register_builtin_components};
use rustpress_core::{parse_markdown, render_html, render_with_template};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    /// Build a site from a directory of markdown files
    Build {
        /// Input directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Custom HTML template file
        #[arg(short, long)]
        template: Option<PathBuf>,
    },
}

/// Create a simple index.html that lists all HTML files
fn create_index_page(dir: &Path) -> Result<()> {
    let mut index_html =
        String::from("<!DOCTYPE html>\n<html>\n<head>\n<title>Site Index</title>\n");
    index_html.push_str("<style>body { font-family: system-ui, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; }</style>\n");
    index_html.push_str("</head>\n<body>\n");
    index_html.push_str("<h1>Site Index</h1>\n<ul>\n");

    // Add links to all HTML files
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "html") {
            let rel_path = path.strip_prefix(dir)?;
            let path_str = rel_path.to_string_lossy();

            // Skip the index file itself
            if rel_path != Path::new("index.html") {
                index_html.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>\n",
                    path_str, path_str
                ));
            }
        }
    }

    index_html.push_str("</ul>\n</body>\n</html>");

    // Write the index file
    let index_path = dir.join("index.html");
    let mut file = File::create(index_path)?;
    file.write_all(index_html.as_bytes())?;

    Ok(())
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
            let content = fs::read_to_string(input)
                .with_context(|| format!("Failed to read file {}", input.display()))?;

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
                Some(path) => {
                    let mut file = File::create(path)?;
                    file.write_all(html.as_bytes())?;
                }
                None => println!("{}", html),
            }
        }

        Commands::Build {
            input,
            output,
            template,
        } => {
            // Ensure output directory exists
            fs::create_dir_all(output)?;

            // Load template if specified
            let template_content = if let Some(template_path) = template {
                Some(fs::read_to_string(template_path)?)
            } else {
                None
            };

            // Process all markdown files
            for entry in WalkDir::new(input) {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                    // Read markdown file
                    let content = fs::read_to_string(path)?;
                    let item = parse_markdown(&content, Some(&registry));

                    // Determine output path
                    let rel_path = path.strip_prefix(input)?;
                    let mut output_path = output.join(rel_path);
                    output_path.set_extension("html");

                    // Create parent directories if needed
                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    // Render HTML
                    let html = if let Some(ref template) = template_content {
                        let template_name = "custom";
                        render_with_template(&item, template_name, template)
                    } else {
                        render_html(&item)
                    };

                    // Write output file
                    fs::write(&output_path, html)?;
                    println!("Generated: {}", &output_path.display());
                } else if path.is_file()
                    && path
                        != template
                            .as_ref()
                            .map(|p| p.as_path())
                            .unwrap_or(&Path::new(""))
                {
                    // Copy non-markdown files (assets)
                    let rel_path = path.strip_prefix(input)?;
                    let output_path = output.join(rel_path);

                    // Create parent directories if needed
                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::copy(path, &output_path)?;
                    println!("Copied: {}", output_path.display());
                }
            }

            println!("Build completed successfully.");
        }
    }

    Ok(())
}
