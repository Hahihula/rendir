use crate::types::ContentItem;

/// Render a ContentItem as HTML
pub fn render_html(item: &ContentItem) -> String {
    // The content is already rendered during parsing
    item.rendered_content
        .as_ref()
        .unwrap_or(&String::new())
        .clone()
}
