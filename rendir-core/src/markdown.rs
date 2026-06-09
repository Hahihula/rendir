use crate::components::{Component, ComponentRegistry};
use crate::types::ContentItem;
use pulldown_cmark::{Options, Parser, html};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Process markdown to recognize custom components
fn process_components(content: &str, registry: &ComponentRegistry) -> String {
    let opening_tag_regex = Regex::new(r"<([A-Z][A-Za-z0-9]+)(\s+[^>]*)?(?:/>|>)").unwrap();
    let attr_regex = Regex::new(r#"([a-zA-Z0-9_-]+)="([^"]*)""#).unwrap();
    let md_attr_regex = Regex::new(r#"([a-zA-Z0-9_-]+)="([^"]*)"|([a-zA-Z0-9_-]+)=([^"\s]+)"#).unwrap();

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

    md_component_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let component_name = &caps[1];
            let attrs_str = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map(|m| m.as_str().trim().to_string());

            // Parse attributes
            let mut attributes = HashMap::new();
            for attr_match in md_attr_regex.captures_iter(attrs_str) {
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
                format!("<!-- Unknown component: {} -->", component_name)
            }
        })
        .to_string()
}

/// Extract frontmatter and content from markdown
/// Extract frontmatter and content from markdown
fn extract_frontmatter(content: &str) -> (HashMap<String, String>, &str) {
    let mut metadata = HashMap::new();

    // Check if the content starts with "---" (frontmatter delimiter)
    if let Some(stripped) = content.strip_prefix("---") {
        if let Some(end_index) = stripped.find("\n---\n") {
            // Handle Unix line endings (\n)
            let frontmatter = &stripped[..end_index];
            let content_start = 3 + end_index + 5; // 3 for "---", 5 for "\n---\n"

            // Parse YAML frontmatter
            if let Ok(yaml_map) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter)
                && let Some(map) = yaml_map.as_mapping() {
                    for (key, value) in map {
                        if let Some(key_str) = key.as_str() {
                            let value_str = if let Some(seq) = value.as_sequence() {
                                seq.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            } else {
                                value.as_str().unwrap_or_default().to_string()
                            };
                            if !value_str.is_empty() {
                                metadata.insert(key_str.to_string(), value_str);
                            }
                        }
                    }
                }
            return (metadata, &content[content_start..]);
        } else if let Some(end_index) = stripped.find("\r\n---\r\n") {
            // Handle Windows line endings (\r\n)
            let frontmatter = &stripped[..end_index];
            let content_start = 3 + end_index + 7; // 3 for "---", 7 for "\r\n---\r\n"

            // Parse YAML frontmatter
            if let Ok(yaml_map) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter)
                && let Some(map) = yaml_map.as_mapping() {
                    for (key, value) in map {
                        if let Some(key_str) = key.as_str() {
                            let value_str = if let Some(seq) = value.as_sequence() {
                                seq.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            } else {
                                value.as_str().unwrap_or_default().to_string()
                            };
                            if !value_str.is_empty() {
                                metadata.insert(key_str.to_string(), value_str);
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
    parse_markdown_with_path(content, registry, None)
}

/// Parse markdown content into a ContentItem with component support and source path
pub fn parse_markdown_with_path(
    content: &str,
    registry: Option<&ComponentRegistry>,
    path: Option<PathBuf>,
) -> ContentItem {
    // Extract frontmatter
    let (metadata, content_without_frontmatter) = extract_frontmatter(content);

    // Process components if registry is provided
    let processed_content = if let Some(reg) = registry {
        process_components(content_without_frontmatter, reg)
    } else {
        content_without_frontmatter.to_string()
    };

    // Convert markdown to HTML
    let html_output = parse_markdown_fragment(&processed_content);

    // Extract local references (images and links) if path is provided
    let local_references = if let Some(ref p) = path {
        let source_dir = get_source_dir(p);
        extract_local_references(content)
            .into_iter()
            .map(|rel_path| {
                if let Some(ref dir) = source_dir {
                    resolve_image_path(&rel_path, dir)
                } else {
                    PathBuf::from(rel_path)
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    // Extract remote image references
    let remote_references = extract_remote_image_references(content);

    ContentItem {
        path,
        content: content.to_string(),
        metadata,
        rendered_content: Some(html_output),
        related_items: Vec::new(),
        image_references: local_references,
        remote_references,
        language: None,
        translations: Vec::new(),
        is_fallback: false,
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

/// Extract all image references from markdown content
/// Returns a list of image source paths (the part inside `![alt](path)`)
pub fn extract_image_references(content: &str) -> Vec<String> {
    let image_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
    image_regex
        .captures_iter(content)
        .map(|cap| cap[2].to_string())
        .collect()
}

/// Extract all local file references from markdown content (images, links, etc.)
/// Returns a list of paths that appear to be local references (not URLs)
pub fn extract_local_references(content: &str) -> Vec<String> {
    let mut references = Vec::new();

    let image_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
    let image_matches: Vec<_> = image_regex.captures_iter(content).collect();
    for cap in &image_matches {
        let path = &cap[2];
        if is_local_path(path) {
            references.push(path.to_string());
        }
    }

    let link_regex = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    for cap in link_regex.captures_iter(content) {
        let full_match_start = cap.get(0).map(|m| m.start()).unwrap_or(0);
        let check_pos = full_match_start.saturating_sub(2);
        let snippet = &content[check_pos..];
        if snippet.starts_with("![") {
            continue;
        }
        let path = &cap[2];
        if is_local_path(path) {
            references.push(path.to_string());
        }
    }

    references
}

/// Check if a path is a local path (not a URL)
fn is_local_path(path: &str) -> bool {
    !path.starts_with("http://")
        && !path.starts_with("https://")
        && !path.starts_with("//")
        && !path.starts_with("mailto:")
        && !path.starts_with("tel:")
}

/// Check if a path is a remote URL (http/https)
fn is_remote_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

/// Extract all remote image references from markdown content
/// Returns a list of remote image URLs
pub fn extract_remote_image_references(content: &str) -> Vec<String> {
    let image_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
    image_regex
        .captures_iter(content)
        .map(|cap| cap[2].to_string())
        .filter(|path| is_remote_url(path))
        .collect()
}

/// Get the directory containing a file path (for relative path resolution)
pub fn get_source_dir(file_path: &Path) -> Option<PathBuf> {
    file_path.parent().map(|p| p.to_path_buf())
}

/// Resolve a relative image path against a source directory
pub fn resolve_image_path(relative_path: &str, source_dir: &Path) -> PathBuf {
    if Path::new(relative_path).is_absolute() {
        PathBuf::from(relative_path)
    } else {
        source_dir.join(relative_path)
    }
}
