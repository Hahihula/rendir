use crate::components::{Component, ComponentRegistry};
use crate::types::ContentItem;
use pulldown_cmark::{Options, Parser, html};
use regex::Regex;
use std::collections::HashMap;

/// Process markdown to recognize custom components
fn process_components(content: &str, registry: &ComponentRegistry) -> String {
    // Process HTML-like component syntax - using a simpler approach without backreferences
    let opening_tag_regex = Regex::new(r"<([A-Z][A-Za-z0-9]+)(\s+[^>]*)?(?:/>|>)").unwrap();

    let mut result = content.to_string();
    let mut replacements = Vec::new();

    // Find all opening tags
    for cap in opening_tag_regex.captures_iter(content) {
        let component_name = &cap[1];
        let attrs_str = cap.get(2).map_or("", |m| m.as_str());
        let full_match = cap.get(0).unwrap();

        // Check if this is a self-closing tag
        if full_match.as_str().ends_with("/>") {
            // Self-closing component
            // Parse attributes
            let attr_regex = Regex::new(r#"([a-zA-Z0-9_-]+)="([^"]*)""#).unwrap();
            let mut attributes = HashMap::new();
            for attr_match in attr_regex.captures_iter(attrs_str) {
                attributes.insert(attr_match[1].to_string(), attr_match[2].to_string());
            }

            let component = Component {
                name: component_name.to_string(),
                attributes,
                content: None,
            };

            if registry.has_component(component_name) {
                replacements.push((
                    full_match.start(),
                    full_match.end(),
                    registry.render(&component),
                ));
            }
        } else {
            // Opening tag, now look for the matching closing tag
            let closing_tag = format!("</{}>", component_name);
            if let Some(end_pos) = result[full_match.end()..].find(&closing_tag) {
                let content_end = full_match.end() + end_pos;
                let component_content = &result[full_match.end()..content_end];

                // Parse attributes
                let attr_regex = Regex::new(r#"([a-zA-Z0-9_-]+)="([^"]*)""#).unwrap();
                let mut attributes = HashMap::new();
                for attr_match in attr_regex.captures_iter(attrs_str) {
                    attributes.insert(attr_match[1].to_string(), attr_match[2].to_string());
                }

                let component = Component {
                    name: component_name.to_string(),
                    attributes,
                    content: Some(component_content.to_string()),
                };

                if registry.has_component(component_name) {
                    replacements.push((
                        full_match.start(),
                        content_end + closing_tag.len(),
                        registry.render(&component),
                    ));
                }
            }
        }
    }

    // Apply replacements in reverse order to not invalidate positions
    replacements.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));
    for (start, end, replacement) in replacements {
        result.replace_range(start..end, &replacement);
    }

    // Process markdown special syntax
    let md_component_regex = Regex::new(r":::([\w-]+)(?:\{([^}]*)\})?([\s\S]*?):::").unwrap();
    let result = md_component_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let component_name = &caps[1];
            let attrs_str = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map(|m| m.as_str().trim().to_string());

            // Parse attributes
            let attr_regex =
                Regex::new(r#"([a-zA-Z0-9_-]+)="([^"]*)"|([a-zA-Z0-9_-]+)=([^"\s]+)"#).unwrap();
            let mut attributes = HashMap::new();
            for attr_match in attr_regex.captures_iter(attrs_str) {
                let key = attr_match
                    .get(1)
                    .or_else(|| attr_match.get(3))
                    .unwrap()
                    .as_str();
                let value = attr_match
                    .get(2)
                    .or_else(|| attr_match.get(4))
                    .unwrap()
                    .as_str();
                attributes.insert(key.to_string(), value.to_string());
            }

            // Create and render the component
            let component = Component {
                name: component_name.to_string(),
                attributes,
                content: if content.is_none() {
                    None
                } else {
                    Some(content.unwrap())
                },
            };

            if registry.has_component(component_name) {
                registry.render(&component)
            } else {
                // If not registered, convert to HTML comment
                format!("<!-- Unknown component: {} -->", component_name)
            }
        })
        .to_string();

    result
}

/// Extract frontmatter and content from markdown
/// Extract frontmatter and content from markdown
fn extract_frontmatter(content: &str) -> (HashMap<String, String>, &str) {
    let mut metadata = HashMap::new();

    // Check if the content starts with "---" (frontmatter delimiter)
    if content.starts_with("---") {
        if let Some(end_index) = content[3..].find("\n---\n").map(|i| i + 3) {
            // Handle Unix line endings (\n)
            let frontmatter = &content[3..end_index];
            let content_start = end_index + 5; // Skip past the ending "---" and newline

            // Parse YAML frontmatter
            if let Ok(yaml_map) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                if let Some(map) = yaml_map.as_mapping() {
                    for (key, value) in map {
                        if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                            metadata.insert(key_str.to_string(), value_str.to_string());
                        }
                    }
                }
            }
            return (metadata, &content[content_start..]);
        } else if let Some(end_index) = content[3..].find("\r\n---\r\n").map(|i| i + 3) {
            // Handle Windows line endings (\r\n)
            let frontmatter = &content[3..end_index];
            let content_start = end_index + 7; // Skip past the ending "---\r\n"

            // Parse YAML frontmatter
            if let Ok(yaml_map) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                if let Some(map) = yaml_map.as_mapping() {
                    for (key, value) in map {
                        if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                            metadata.insert(key_str.to_string(), value_str.to_string());
                        }
                    }
                }
            }
            return (metadata, &content[content_start..]);
        }
    }

    // No frontmatter found, return empty metadata and full content
    (metadata, content)
}

/// Parse markdown content into a ContentItem with component support
pub fn parse_markdown(content: &str, registry: Option<&ComponentRegistry>) -> ContentItem {
    // Extract frontmatter
    let (metadata, content_without_frontmatter) = extract_frontmatter(content);

    // Process components if registry is provided
    let processed_content = if let Some(reg) = registry {
        process_components(&content_without_frontmatter, reg)
    } else {
        content_without_frontmatter.to_string()
    };

    // Convert markdown to HTML
    let html_output = parse_markdown_fragment(&processed_content);

    ContentItem {
        path: None,
        content: content.to_string(),
        metadata,
        rendered_content: Some(html_output),
        related_items: Vec::new(),
    }
}

// Helper function to parse markdown fragments
pub fn parse_markdown_fragment(content: &str) -> String {
    // Set up parser options
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    // Parse the content
    let parser = Parser::new_ext(content, options);

    // Convert markdown to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}
