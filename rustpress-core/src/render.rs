use crate::types::ContentItem;
use std::collections::HashMap;
use std::sync::Once;
use std::sync::OnceLock;

use tera::{Context, Tera};

// Initialize Tera once
static TERA: OnceLock<Tera> = OnceLock::new();
/// Get the Tera instance (singleton)/// Get or initialize the Tera template engine
fn get_tera() -> &'static Tera {
    TERA.get_or_init(|| {
        let mut tera = Tera::default();
        // Register default templates
        tera.add_raw_template("default", include_str!("templates/default.html"))
            .expect("Failed to parse default template");
        tera
    })
}

/// Default HTML template
const DEFAULT_HTML_TEMPLATE: &str = include_str!("templates/default.html");

/// Render a ContentItem as HTML
pub fn render_html(item: &ContentItem) -> String {
    let default_content = String::new();

    let content = item.rendered_content.as_ref().unwrap_or(&default_content);
    // Create Tera context with metadata
    let mut context = Context::new();

    // Add content
    context.insert("content", content);

    // Add all metadata
    for (key, value) in &item.metadata {
        context.insert(key, value);
    }

    // Use default title if not provided
    if !item.metadata.contains_key("title") {
        context.insert("title", "Rustpress Page");
    }

    // Render with Tera
    match get_tera().render("default", &context) {
        Ok(rendered) => rendered,
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            // Fallback to basic template with simple replacement
            include_str!("templates/default.html")
                .replace(
                    "{{title}}",
                    item.metadata
                        .get("title")
                        .unwrap_or(&String::from("Rustpress Page")),
                )
                .replace("{{content}}", content)
        }
    }
}

/// Render a ContentItem as HTML with a custom template
pub fn render_with_template(
    item: &ContentItem,
    template_name: &str,
    template_content: &str,
) -> String {
    let default_content = String::new();
    let content = item.rendered_content.as_ref().unwrap_or(&default_content);

    // Register the template
    let mut tera = Tera::default();
    if let Err(e) = tera.add_raw_template(template_name, template_content) {
        eprintln!("Template parsing error: {}", e);
        return format!("<!-- Template error: {} -->", e);
    }

    // Create Tera context
    let mut context = Context::new();

    // Add content
    context.insert("content", content);

    // Replace template placeholders - this is very basic for now
    // Add all metadata
    for (key, value) in &item.metadata {
        context.insert(key, value);
    }

    // Render with Tera
    match tera.render(template_name, &context) {
        Ok(rendered) => rendered,
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            format!("<!-- Template rendering error: {} -->", e)
        }
    }
}
