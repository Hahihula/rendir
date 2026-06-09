use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub url: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    documents: Vec<SearchDocument>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    pub fn add_document(&mut self, doc: SearchDocument) {
        self.documents.push(doc);
    }

    pub fn build(self) -> BuiltSearchIndex {
        let mut index: std::collections::HashMap<String, Vec<SearchDocument>> =
            std::collections::HashMap::new();
        for doc in self.documents.clone() {
            let title_lower = doc.title.to_lowercase();
            for word in title_lower.split_whitespace() {
                index.entry(word.to_string()).or_default().push(doc.clone());
            }
            let content_words: Vec<String> = doc
                .content
                .to_lowercase()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            for word in content_words {
                if word.len() > 2 {
                    index.entry(word).or_default().push(doc.clone());
                }
            }
        }
        BuiltSearchIndex {
            index,
            documents: self.documents,
        }
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltSearchIndex {
    index: std::collections::HashMap<String, Vec<SearchDocument>>,
    documents: Vec<SearchDocument>,
}

impl BuiltSearchIndex {
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scores: std::collections::HashMap<String, (f32, &SearchDocument)> =
            std::collections::HashMap::new();

        for word in &query_words {
            if let Some(docs) = self.index.get(*word) {
                for doc in docs {
                    let entry = scores.entry(doc.id.clone()).or_insert((0.0, doc));
                    entry.0 += 1.0;
                }
            }
        }

        for doc in &self.documents {
            let title_lower = doc.title.to_lowercase();
            let content_lower = doc.content.to_lowercase();
            for word in &query_words {
                if title_lower.contains(word) {
                    let entry = scores.entry(doc.id.clone()).or_insert((0.0, doc));
                    entry.0 += 2.0;
                }
                if content_lower.contains(word) {
                    let entry = scores.entry(doc.id.clone()).or_insert((0.0, doc));
                    entry.0 += 1.0;
                }
            }
        }

        let mut results: Vec<SearchResult> = scores
            .into_iter()
            .filter(|(_, (score, _))| *score > 0.0)
            .map(|(_, (score, doc))| {
                let snippet = Self::extract_snippet(&doc.content, &query_lower);
                SearchResult {
                    id: doc.id.clone(),
                    title: doc.title.clone(),
                    url: doc.url.clone(),
                    snippet,
                    score,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    fn extract_snippet(content: &str, query: &str) -> String {
        let content_lower = content.to_lowercase();
        let query_words: Vec<&str> = query.split_whitespace().collect();

        if let Some(first_word) = query_words.first()
            && let Some(pos) = content_lower.find(first_word) {
                let start = pos.saturating_sub(50);
                let end = (pos + 150).min(content.len());
                let mut snippet = content[start..end].to_string();
                if start > 0 {
                    snippet = format!("...{}", snippet);
                }
                if end < content.len() {
                    snippet = format!("{}...", snippet);
                }
                return snippet;
            }

        content[..150.min(content.len())].to_string()
    }

    pub fn into_serialized(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap_or_default()
    }

    pub fn from_serialized(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

pub trait SearchEngine: Send + Sync {
    fn add_document(&mut self, doc: SearchDocument);
    fn search(&self, query: &str, limit: usize) -> Vec<SearchResult>;
    fn clear(&mut self);
}

pub struct DefaultSearchEngine {
    index: BuiltSearchIndex,
}

impl DefaultSearchEngine {
    pub fn new() -> Self {
        Self {
            index: SearchIndex::new().build(),
        }
    }

    pub fn build(self) -> BuiltSearchIndex {
        self.index
    }
}

impl Default for DefaultSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine for DefaultSearchEngine {
    fn add_document(&mut self, doc: SearchDocument) {
        let mut search_index = SearchIndex::new();
        search_index.add_document(doc);
        self.index = search_index.build();
    }

    fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        self.index.search(query, limit)
    }

    fn clear(&mut self) {
        self.index = SearchIndex::new().build();
    }
}
