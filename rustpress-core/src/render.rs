use crate::types::{
    BlogIndexStore, ChapterStore, ContentItem, MdBookStore, Slide, SlideLayout, SlideshowStore,
};
use std::collections::HashMap;

const DEFAULT_HTML_TEMPLATE: &str = include_str!("templates/default.html");
const SLIDESHOW_TEMPLATE: &str = include_str!("templates/slideshow/index.html");
const PRESENTER_TEMPLATE: &str = include_str!("templates/slideshow/presenter.html");
const MDBOOK_TEMPLATE: &str = include_str!("templates/mdbook/index.html");
const BLOG_INDEX_TEMPLATE: &str = include_str!("templates/blog/index.html");
const BLOG_POST_TEMPLATE: &str = include_str!("templates/blog/post.html");
const SHARED_SEARCH_JS: &str = include_str!("templates/shared/search.js");

/// Get a builtin template by name
pub fn get_builtin_template(name: &str) -> Option<&'static str> {
    match name {
        "slideshow" | "slideshow/index" => Some(SLIDESHOW_TEMPLATE),
        "slideshow/presenter" => Some(PRESENTER_TEMPLATE),
        "mdbook" => Some(MDBOOK_TEMPLATE),
        "blog" | "blog/index" => Some(BLOG_INDEX_TEMPLATE),
        "blog/post" => Some(BLOG_POST_TEMPLATE),
        _ => None,
    }
}

/// Simple HTML renderer (backward compatible, no Tera)
pub fn render_html(item: &ContentItem) -> String {
    let default_content = String::new();
    let content = item.rendered_content.as_ref().unwrap_or(&default_content);

    let title = item
        .metadata
        .get("title")
        .cloned()
        .unwrap_or_else(|| "Rustpress Page".to_string());

    DEFAULT_HTML_TEMPLATE
        .replace("{{ title }}", &title)
        .replace("{{ content }}", content)
}

/// Render HTML with a custom template string (backward compatible, no Tera)
pub fn render_with_template(
    item: &ContentItem,
    _template_name: &str,
    template_content: &str,
) -> String {
    let default_content = String::new();
    let content = item.rendered_content.as_ref().unwrap_or(&default_content);

    let title = item
        .metadata
        .get("title")
        .cloned()
        .unwrap_or_else(|| "Rustpress Page".to_string());

    let author = item.metadata.get("author").cloned().unwrap_or_default();
    let date = item.metadata.get("date").cloned().unwrap_or_default();
    let category = item.metadata.get("category").cloned().unwrap_or_default();
    let description = item
        .metadata
        .get("description")
        .cloned()
        .unwrap_or_default();
    let tags: Vec<String> = item
        .metadata
        .get("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let mut html = template_content.to_string();

    // Standard field replacements - Vue handles rendering, Rust just provides data
    html = html.replace("{{ title }}", &title);
    html = html.replace("{{ content }}", content);
    html = html.replace("{{ author }}", &author);
    html = html.replace("{{ date }}", &date);
    html = html.replace("{{ category }}", &category);
    html = html.replace("{{ description }}", &description);

    // Handle posts_html, tag_cloud, recent_posts from metadata
    if let Some(posts_html) = item.metadata.get("posts_html") {
        html = html.replace("{{ posts_html }}", posts_html);
        html = html.replace("{{ posts_html | safe }}", posts_html);
    }
    if let Some(tag_cloud) = item.metadata.get("tag_cloud") {
        html = html.replace("{{ tag_cloud }}", tag_cloud);
        html = html.replace("{{ tag_cloud | safe }}", tag_cloud);
    }
    if let Some(recent_posts) = item.metadata.get("recent_posts") {
        html = html.replace("{{ recent_posts }}", recent_posts);
        html = html.replace("{{ recent_posts | safe }}", recent_posts);
    }
    if let Some(search_index) = item.metadata.get("search_index") {
        html = html.replace("{{ search_index }}", search_index);
        html = html.replace("{{ search_index | safe }}", search_index);
    }

    if !tags.is_empty() {
        let tags_html = tags
            .iter()
            .map(|t| format!("<span class=\"tag\">{}</span>", t))
            .collect::<Vec<_>>()
            .join(" ");
        html = html.replace("{{ tags }}", &tags_html);
    }

    html
}

/// Convert a ContentItem to a SlideshowStore for Vue SPA
pub fn content_to_slideshow_store(item: &ContentItem) -> SlideshowStore {
    let title = item
        .metadata
        .get("title")
        .cloned()
        .unwrap_or_else(|| "Slideshow".to_string());

    let default_content = String::new();
    let content = item.rendered_content.as_ref().unwrap_or(&default_content);

    // Parse slides from the HTML content
    let slides = parse_slides_from_html(content);

    // Collect images
    let images: HashMap<String, String> = item
        .image_references
        .iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|name| (name.to_string(), p.to_string_lossy().to_string()))
        })
        .collect();

    SlideshowStore {
        title,
        slides,
        current_slide_index: 0,
        images,
    }
}

/// Parse slides from HTML content looking for <div class="slide"> elements
fn parse_slides_from_html(html: &str) -> Vec<Slide> {
    let mut slides = Vec::new();
    let mut slide_id = 0;

    // Match slide elements: <div class="slide slide-..." data-title="...">...</div>
    let slide_regex = regex::Regex::new(
        r#"<div\s+class="slide\s+slide-([^"]+)"\s+data-title="([^"]*)"[^>]*>([\s\S]*?)</div>"#,
    )
    .unwrap();

    // Also match slides without title
    let slide_no_title_regex =
        regex::Regex::new(r#"<div\s+class="slide\s+slide-([^"]+)"[^>]*>([\s\S]*?)</div>"#).unwrap();

    for cap in slide_regex.captures_iter(html) {
        let type_str = cap.get(1).map(|m| m.as_str()).unwrap_or("full");
        let title = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let content = cap
            .get(3)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        let layout = parse_layout_from_tag(type_str);

        slides.push(Slide {
            id: slide_id,
            layout,
            title,
            content: content.trim().to_string(),
            notes: String::new(),
            images: Vec::new(),
        });
        slide_id += 1;
    }

    // Handle slides without title (only if we haven't found any with title)
    if slides.is_empty() {
        for cap in slide_no_title_regex.captures_iter(html) {
            let type_str = cap.get(1).map(|m| m.as_str()).unwrap_or("full");
            let content = cap
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let layout = parse_layout_from_tag(type_str);

            slides.push(Slide {
                id: slide_id,
                layout,
                title: String::new(),
                content: content.trim().to_string(),
                notes: String::new(),
                images: Vec::new(),
            });
            slide_id += 1;
        }
    }

    // If still no slides found, create a single slide from the entire content
    if slides.is_empty() && !html.is_empty() {
        slides.push(Slide {
            id: 0,
            layout: SlideLayout::Full,
            title: String::new(),
            content: html.to_string(),
            notes: String::new(),
            images: Vec::new(),
        });
    }

    slides
}

/// Parse slide layout from tag name
fn parse_layout_from_tag(tag: &str) -> SlideLayout {
    match tag.to_lowercase().as_str() {
        "full" => SlideLayout::Full,
        "split-left" => SlideLayout::SplitLeft,
        "split-right" => SlideLayout::SplitRight,
        "center" => SlideLayout::Center,
        "list" => SlideLayout::List,
        "image-bg" | "imagebg" => SlideLayout::ImageBg,
        _ => SlideLayout::Full,
    }
}

/// Serialize SlideshowStore to JSON for embedding in Vue SPA
pub fn serialize_slideshow_store(store: &SlideshowStore) -> String {
    serde_json::to_string(store).unwrap_or_else(|_| r#"{"title":"Error","slides":[]}"#.to_string())
}

/// Render a slideshow using Vue SPA template with embedded store data
pub fn render_slideshow_vue(item: &ContentItem) -> String {
    let store = content_to_slideshow_store(item);
    let store_json = serialize_slideshow_store(&store);

    let template = SLIDESHOW_TEMPLATE;
    template.replace("{{STORE_DATA}}", &store_json)
}

/// Render presenter view using Vue SPA template with embedded store data
pub fn render_presenter_vue(item: &ContentItem) -> String {
    let store = content_to_slideshow_store(item);
    let store_json = serialize_slideshow_store(&store);

    let template = PRESENTER_TEMPLATE;
    template.replace("{{STORE_DATA}}", &store_json)
}

/// Render blog index using Vue SPA template with embedded store data
pub fn render_blog_index_vue(store: &BlogIndexStore) -> String {
    let store_json =
        serde_json::to_string(store).unwrap_or_else(|_| r#"{"title":"Error"}"#.to_string());
    let html = BLOG_INDEX_TEMPLATE.replace("{{STORE_DATA}}", &store_json);
    html.replace(
        "{{SEARCH_COMPONENT_SCRIPT}}",
        &format!("<script>\n{}</script>", SHARED_SEARCH_JS),
    )
}

/// Render mdbook chapter using Vue SPA template with embedded store data
pub fn render_mdbook_vue(
    book_title: &str,
    chapters: &[ChapterStore],
    current_chapter: &ChapterStore,
    search_index_json: &str,
) -> String {
    let store = MdBookStore {
        title: book_title.to_string(),
        chapters: chapters.to_vec(),
        current_chapter: current_chapter.clone(),
        search_index: search_index_json.to_string(),
    };
    let store_json =
        serde_json::to_string(&store).unwrap_or_else(|_| r#"{"title":"Error"}"#.to_string());
    let html = MDBOOK_TEMPLATE.replace("{{STORE_DATA}}", &store_json);
    html.replace(
        "{{SEARCH_COMPONENT_SCRIPT}}",
        &format!("<script>\n{}</script>", SHARED_SEARCH_JS),
    )
}
