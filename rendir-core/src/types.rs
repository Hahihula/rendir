use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a content item (e.g., a markdown file)
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ContentItem {
    /// File path (if from a file)
    pub path: Option<PathBuf>,

    /// Raw content
    pub content: String,

    /// Metadata extracted from frontmatter or inferred
    pub metadata: HashMap<String, String>,

    /// Rendered HTML content
    pub rendered_content: Option<String>,

    /// Related content items (by reference)
    pub related_items: Vec<RelatedContent>,

    /// Image file paths referenced in this content (local)
    pub image_references: Vec<PathBuf>,

    /// Remote image URLs referenced in this content
    #[serde(default)]
    pub remote_references: Vec<String>,

    /// Language code (e.g., "en", "de", "cs", "zh")
    #[serde(default)]
    pub language: Option<String>,

    /// All available translations of this page
    #[serde(default)]
    pub translations: Vec<Translation>,

    /// Whether this content was generated from fallback language
    #[serde(default)]
    pub is_fallback: bool,
}

/// Represents a translation of a page
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Translation {
    /// Language code
    pub language: String,
    /// URL to the translated page
    pub url: String,
    /// Title of the translated page
    pub title: String,
    /// Whether this translation actually exists or is a fallback
    #[serde(default)]
    pub exists: bool,
}

/// Represents a supported language
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Language {
    /// Language code (e.g., "en", "de", "cs", "zh")
    pub code: String,
    /// Native name (e.g., "English", "Deutsch", "Čeština", "中文")
    pub name: String,
    /// Whether this is the default/fallback language
    #[serde(default)]
    pub is_default: bool,
}

/// Represents a relationship between content items
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RelatedContent {
    /// Type of relationship
    pub relation_type: RelationType,

    /// Path to the related content
    pub path: PathBuf,
}

/// Types of relationships between content items
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum RelationType {
    Parent,
    Child,
    Sibling,
    Tag,
    Category,
    Custom(String),
}

// =============================================================================
// Vue Store Types - Data structures for Vue SPA templates
// =============================================================================

/// Slide layout types for slideshow Vue components
#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub enum SlideLayout {
    #[serde(rename = "full")]
    #[default]
    Full,
    #[serde(rename = "split-left")]
    SplitLeft,
    #[serde(rename = "split-right")]
    SplitRight,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "list")]
    List,
    #[serde(rename = "image-bg")]
    ImageBg,
}

/// A single slide in a slideshow
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Slide {
    pub id: usize,
    #[serde(rename = "type")]
    pub layout: SlideLayout,
    pub title: String,
    pub content: String,
    pub notes: String,
    #[serde(default)]
    pub images: Vec<String>,
}

/// Vue store data for a slideshow
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SlideshowStore {
    pub title: String,
    pub slides: Vec<Slide>,
    #[serde(default)]
    pub current_slide_index: usize,
    #[serde(default)]
    pub images: std::collections::HashMap<String, String>,
}

/// Vue store data for a blog post
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BlogPostStore {
    pub title: String,
    pub author: String,
    pub date: String,
    pub category: String,
    pub tags: Vec<String>,
    pub content: String,
    pub images: Vec<String>,
    /// Available translations for this post
    #[serde(default)]
    pub translations: Vec<Translation>,
}

/// Vue store data for the blog index
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BlogIndexStore {
    pub title: String,
    pub description: String,
    pub content: String,
    pub posts: Vec<BlogPostSummary>,
    pub tags: Vec<TagCount>,
    pub recent_posts: Vec<BlogPostSummary>,
    /// JSON-serialized BuiltSearchIndex for client-side full-text search
    #[serde(default)]
    pub search_index: String,
    /// Available languages for i18n
    #[serde(default)]
    pub languages: Vec<Language>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BlogPostSummary {
    pub id: String,
    pub title: String,
    pub date: String,
    pub author: String,
    pub excerpt: String,
    pub tags: Vec<String>,
    pub url: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TagCount {
    pub name: String,
    pub count: usize,
}

/// Vue store data for mdbook
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct MdBookStore {
    pub title: String,
    pub chapters: Vec<ChapterStore>,
    pub current_chapter: ChapterStore,
    /// JSON-serialized BuiltSearchIndex for client-side full-text search
    #[serde(default)]
    pub search_index: String,
    /// Available languages for i18n
    #[serde(default)]
    pub languages: Vec<Language>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChapterStore {
    pub title: String,
    pub url: String,
    pub content: String,
    pub level: usize,
    pub children: Vec<ChapterStore>,
    pub prev_chapter: Option<ChapterNav>,
    pub next_chapter: Option<ChapterNav>,
    /// Available translations for this chapter
    #[serde(default)]
    pub translations: Vec<Translation>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChapterNav {
    pub title: String,
    pub url: String,
}
