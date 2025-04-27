use crate::types::ContentItem;
use pulldown_cmark::{Options, Parser, html};

/// Parse markdown content into a ContentItem
pub fn parse_markdown(content: &str) -> ContentItem {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);

    // Convert markdown to HTML right away
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    ContentItem {
        path: None,
        content: content.to_string(),
        metadata: Default::default(),
        rendered_content: Some(html_output),
        related_items: Vec::new(),
    }
}
