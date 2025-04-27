use crate::components::{Component, ComponentRegistry};
use crate::types::ContentItem;
use pulldown_cmark::{Options, Parser, html};
use regex::Regex;
use std::collections::HashMap;

/// Process markdown to recognize custom components
fn process_components(content: &str, registry: &ComponentRegistry) -> String {
    // Process HTML-like component syntax
    let html_component_regex =
        Regex::new(r"<([A-Z][A-Za-z0-9]+)(\s+[^>]*)?\s*(?:/>|>(.*?)</\1>)").unwrap();
    let content = html_component_regex.replace_all(content, |caps: &regex::Captures| {
        let component_name = &caps[1];
        let attrs_str = caps.get(2).map_or("", |m| m.as_str());
        let content = caps.get(3).map(|m| m.as_str().to_string());

        // Parse attributes
        let attr_regex = Regex::new(r#"([a-zA-Z0-9_-]+)="([^"]*)""#).unwrap();
        let mut attributes = HashMap::new();
        for attr_match in attr_regex.captures_iter(attrs_str) {
            attributes.insert(attr_match[1].to_string(), attr_match[2].to_string());
        }

        // Create and render the component
        let component = Component {
            name: component_name.to_string(),
            attributes,
            content,
        };

        if registry.has_component(component_name) {
            registry.render(&component)
        } else {
            // If not registered, keep original text
            caps[0].to_string()
        }
    });

    // Process markdown special syntax
    let md_component_regex = Regex::new(r":::([\w-]+)(?:\{([^}]*)\})?([\s\S]*?):::").unwrap();
    md_component_regex
        .replace_all(&content, |caps: &regex::Captures| {
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
                content,
            };

            if registry.has_component(component_name) {
                registry.render(&component)
            } else {
                // If not registered, convert to HTML comment
                format!("<!-- Unknown component: {} -->", component_name)
            }
        })
        .to_string()
}

/// Extract frontmatter and content from markdown
fn extract_frontmatter(content: &str) -> (HashMap<String, String>, &str) {
    let mut metadata = HashMap::new();

    // Check if the content starts with "---" (frontmatter delimiter)
    if content.starts_with("---") {
        if let Some(end_index) = content
            .find("\n---\n")
            .or_else(|| content.find("\r\n---\r\n"))
        {
            let frontmatter = &content[3..end_index];
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
            // Return metadata and content after frontmatter
            let content_start = end_index + 5; // Skip past the ending "---" and newline
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
        process_components(content_without_frontmatter, reg)
    } else {
        content_without_frontmatter.to_string()
    };
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&processed_content, options);

    // Convert markdown to HTML right away
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    ContentItem {
        path: None,
        content: content.to_string(),
        metadata,
        rendered_content: Some(html_output),
        related_items: Vec::new(),
    }
}
