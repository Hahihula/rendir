pub mod components;
pub mod markdown;
pub mod mdbook;
pub mod render;
pub mod rss;
pub mod search;
pub mod types;
pub mod vue;

pub use components::{Component, ComponentRegistry};
pub use markdown::{parse_markdown, parse_markdown_with_path};
pub use render::{
    content_to_slideshow_store, get_builtin_template, render_blog_index_vue, render_html,
    render_mdbook_vue, render_presenter_vue, render_slideshow_vue, render_with_template,
    serialize_slideshow_store,
};
pub use search::{BuiltSearchIndex, SearchDocument, SearchIndex, SearchResult};
pub use types::{ChapterNav, ChapterStore, MdBookStore, Slide, SlideLayout, SlideshowStore};
pub use vue::{parse_vue_component, VueComponent, VueRegistry};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_markdown() {
        let content = "# Hello\n\nThis is a test.";
        let item = parse_markdown(content, None);
        let html = item.rendered_content.unwrap();
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("This is a test"));
    }

    #[test]
    fn test_parse_markdown_with_frontmatter() {
        let content = "---
title: My Page
author: Test Author
---
# Content";
        let item = parse_markdown(content, None);
        assert_eq!(item.metadata.get("title"), Some(&"My Page".to_string()));
        assert_eq!(
            item.metadata.get("author"),
            Some(&"Test Author".to_string())
        );
        assert!(item.rendered_content.unwrap().contains("<h1>Content</h1>"));
    }

    #[test]
    fn test_render_alert_component() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#"<Alert type="warning" title="Watch out">
This is a warning.
</Alert>"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("alert-warning"));
        assert!(html.contains("Watch out"));
        assert!(html.contains("This is a warning"));
    }

    #[test]
    fn test_render_youtube_component() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#"<YouTube id="dQw4w9WgXcQ" />"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("youtube.com/embed/dQw4w9WgXcQ"));
    }

    #[test]
    fn test_render_html_without_registry() {
        let content = "# Title\n\nParagraph";
        let item = parse_markdown(content, None);
        let html = render_html(&item);
        assert!(html.contains("<h1>Title</h1>"));
    }

    #[test]
    fn test_tabs_component_markdown_syntax() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#":::tabs
## Tab One
Content of tab one

## Tab Two
Content of tab two
:::"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("tab-btn"));
        assert!(html.contains("Tab One"));
        assert!(html.contains("Tab Two"));
    }

    #[test]
    fn test_tabs_component_html_syntax() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#"<Tabs>
## First Tab
Some content here

## Second Tab
More content
</Tabs>"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("tab-btn"));
        assert!(html.contains("First Tab"));
    }

    #[test]
    fn test_alert_all_types() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        for alert_type in ["info", "warning", "error"] {
            let content = format!(
                r#"<Alert type="{}" title="Test">Alert content</Alert>"#,
                alert_type
            );
            let item = parse_markdown(&content, Some(&registry));
            let html = item.rendered_content.unwrap();
            assert!(
                html.contains(&format!("alert-{}", alert_type).as_str()),
                "Alert type {} should be present",
                alert_type
            );
            assert!(html.contains("Test"));
            assert!(html.contains("Alert content"));
        }
    }

    #[test]
    fn test_alert_with_markdown_syntax() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#":::alert{type="warning" title="Warning"}
This is **bold** content
:::"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("alert-warning"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_render_with_custom_template() {
        use crate::render::render_with_template;

        let content = "# Hello\n\nWorld";
        let item = parse_markdown(content, None);
        let template = "<html><body>{{ content }}</body></html>";
        let html = render_with_template(&item, "custom", template);
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_frontmatter_with_special_chars() {
        let content = "---
title: \"Test: With Colon\"
author: Jane Doe
tags: [rust, web]
---
# Content";
        let item = parse_markdown(content, None);
        assert_eq!(
            item.metadata.get("title"),
            Some(&"Test: With Colon".to_string())
        );
        assert_eq!(item.metadata.get("author"), Some(&"Jane Doe".to_string()));
    }

    #[test]
    fn test_youtube_component_no_id() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#"<YouTube />"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("YouTube component requires 'id' attribute"));
    }

    fn assert_html_balanced(html: &str, _component_name: &str) {
        use std::collections::HashSet;
        let void_elements: HashSet<&str> = [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
            "source", "track", "wbr",
        ]
        .into_iter()
        .collect();
        let raw_text_elements = [
            "script", "style", "textarea", "noscript", "xmp", "pre", "code",
        ];

        let mut stack: Vec<&str> = Vec::new();
        let bytes = html.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            if bytes[i] == b'<' {
                if i + 3 < len && bytes[i + 1] == b'!' {
                    if let Some(end) = html[i..].find("-->") {
                        i += end + 3;
                        continue;
                    }
                }

                let is_closing = i + 1 < len && bytes[i + 1] == b'/';

                if is_closing {
                    let mut j = i + 2;
                    while j < len && bytes[j] != b'>' {
                        j += 1;
                    }
                    let mut tag_end = j;
                    while tag_end > i + 2
                        && (bytes[tag_end - 1] == b' '
                            || bytes[tag_end - 1] == b'\t'
                            || bytes[tag_end - 1] == b'\n'
                            || bytes[tag_end - 1] == b'\r')
                    {
                        tag_end -= 1;
                    }
                    let tag = &html[i + 2..tag_end];
                    if let Some(last) = stack.last() {
                        if *last == tag {
                            stack.pop();
                        }
                    }
                    i = if j < len && bytes[j] == b'>' {
                        j + 1
                    } else {
                        j
                    };
                    continue;
                }

                let mut j = i + 1;
                while j < len
                    && (bytes[j] == b' '
                        || bytes[j] == b'\t'
                        || bytes[j] == b'\n'
                        || bytes[j] == b'\r')
                {
                    j += 1;
                }
                let tag_start = j;
                j = tag_start;
                while j < len && bytes[j] != b'>' {
                    j += 1;
                }
                let tag = &html[tag_start..j];

                if tag.is_empty() || tag.starts_with('!') {
                    i = if j < len && bytes[j] == b'>' {
                        j + 1
                    } else {
                        j
                    };
                    continue;
                }

                let base = tag.split_whitespace().next().unwrap_or(tag);

                if raw_text_elements.contains(&base) {
                    if let Some(end) = html[i..].find(&format!("</{}>", base)) {
                        i = i + end + base.len() + 3;
                        continue;
                    }
                }

                if void_elements.contains(&base) || base.ends_with('/') {
                    i = if j < len && bytes[j] == b'>' {
                        j + 1
                    } else {
                        j
                    };
                    continue;
                }

                stack.push(base);
                i = if j < len && bytes[j] == b'>' {
                    j + 1
                } else {
                    j
                };
                continue;
            }
            i += 1;
        }

        eprintln!("Final stack: {:?}", stack);
        assert!(
            stack.is_empty(),
            "HTML not balanced, missing: </{}>",
            stack.join(", </")
        );
    }

    #[test]
    fn test_all_components_html_structure() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let components = [
            ("Search", r#"<Search placeholder="Find something" />"#),
            ("Slide", r#"<Slide title="My Slide">Hello World</Slide>"#),
        ];

        for (name, content) in components {
            let item = parse_markdown(content, Some(&registry));
            let html = item.rendered_content.unwrap();
            assert_html_balanced(&html, name);
        }
    }

    #[test]
    fn test_all_registered_components_html_structure() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        for name in registry.component_names() {
            let component = Component {
                name: name.to_string(),
                attributes: std::collections::HashMap::new(),
                content: Some("Test content".to_string()),
            };
            let html = registry.render(&component);
            assert_html_balanced(&html, name);
        }
    }

    #[test]
    fn test_search_index_add_and_search() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Hello World".to_string(),
            content: "This is a test document".to_string(),
            url: "/hello".to_string(),
            tags: vec!["test".to_string()],
        });
        index.add_document(SearchDocument {
            id: "2".to_string(),
            title: "Rust Programming".to_string(),
            content: "Rust is a systems programming language".to_string(),
            url: "/rust".to_string(),
            tags: vec!["rust".to_string(), "programming".to_string()],
        });
        let built = index.build();
        let results = built.search("rust", 10);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.title.contains("Rust")));
    }

    #[test]
    fn test_search_index_fuzzy_match() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Getting Started with Rust".to_string(),
            content: "Learn Rust programming step by step".to_string(),
            url: "/rust-getting-started".to_string(),
            tags: vec!["rust".to_string()],
        });
        let built = index.build();
        let results = built.search("program", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_index_no_results() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Hello World".to_string(),
            content: "Just a simple document".to_string(),
            url: "/hello".to_string(),
            tags: vec![],
        });
        let built = index.build();
        let results = built.search("nonexistent query xyz", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_index_title_boost() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Python Tutorial".to_string(),
            content: "Learn Python programming".to_string(),
            url: "/python".to_string(),
            tags: vec![],
        });
        index.add_document(SearchDocument {
            id: "2".to_string(),
            title: "JavaScript Guide".to_string(),
            content: "JavaScript is a language for the web".to_string(),
            url: "/javascript".to_string(),
            tags: vec![],
        });
        let built = index.build();
        let results = built.search("JavaScript", 10);
        assert!(!results.is_empty());
        let first = &results[0];
        assert!(first.title.contains("JavaScript"));
    }

    #[test]
    fn test_search_serialization_roundtrip() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Test Document".to_string(),
            content: "Content here".to_string(),
            url: "/test".to_string(),
            tags: vec!["test".to_string()],
        });
        let built = index.build();
        let serialized = built.into_serialized();
        let deserialized = BuiltSearchIndex::from_serialized(&serialized);
        assert!(deserialized.is_some());
        let results = deserialized.unwrap().search("test", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_slide_component_renders() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#"<Slide title="Welcome"># Hello World

This is a test slide.
</Slide>"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("slide-full"));
        assert!(html.contains("data-title=\"Welcome\""));
        assert!(html.contains("Hello World"));
    }

    #[test]
    fn test_slide_component_without_title() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#"<Slide>Simple slide content</Slide>"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("slide-full"));
        assert!(html.contains("Simple slide content"));
    }

    #[test]
    fn test_multiple_slides_in_markdown() {
        let mut registry = ComponentRegistry::new();
        crate::components::builtins::register_builtin_components(&mut registry);

        let content = r#"<Slide title="Slide 1">Content 1</Slide>
<Slide title="Slide 2">Content 2</Slide>"#;
        let item = parse_markdown(content, Some(&registry));
        let html = item.rendered_content.unwrap();
        assert!(html.contains("data-title=\"Slide 1\""));
        assert!(html.contains("data-title=\"Slide 2\""));
        assert!(html.contains("Content 1"));
        assert!(html.contains("Content 2"));
    }

    #[test]
    fn test_extract_image_references() {
        use crate::markdown::extract_image_references;

        let content = r#"Here is an image:
![Logo](images/logo.png)

Another image:
![Screenshot](screenshots/home.png)

No image here.
"#;
        let refs = extract_image_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], "images/logo.png");
        assert_eq!(refs[1], "screenshots/home.png");
    }

    #[test]
    fn test_extract_image_references_none() {
        use crate::markdown::extract_image_references;

        let content = "This is just plain text with no images.";
        let refs = extract_image_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_parse_markdown_with_path_includes_images() {
        use crate::markdown::parse_markdown_with_path;
        use std::path::PathBuf;

        let content = r#"---
title: Test
---

# Hello

![Logo](logo.png)

Some content with ![Icon](icons/icon.png) inline.
"#;
        let item =
            parse_markdown_with_path(content, None, Some(PathBuf::from("/project/src/page.md")));
        assert_eq!(item.image_references.len(), 2);
        assert!(item.image_references[0]
            .to_string_lossy()
            .ends_with("logo.png"));
        assert!(item.image_references[1]
            .to_string_lossy()
            .ends_with("icons/icon.png"));
    }
}
