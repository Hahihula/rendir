use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a content item (e.g., a markdown file)
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
}

/// Represents a relationship between content items
pub struct RelatedContent {
    /// Type of relationship
    pub relation_type: RelationType,

    /// Path to the related content
    pub path: PathBuf,
}

/// Types of relationships between content items
pub enum RelationType {
    Parent,
    Child,
    Sibling,
    Tag,
    Category,
    Custom(String),
}
