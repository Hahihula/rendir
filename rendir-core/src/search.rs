use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub tags: Vec<String>,
    pub snippet: String,
    pub score: f32,
    /// Character index of the earliest matched trigram within the document's
    /// lowercased plain text (title + " " + content). `0` is a valid value and
    /// is also used as a sentinel for "no match" if the field is ever set
    /// without a `matched > 0` guard. Callers should always check the
    /// accompanying `score > 0` before treating this as meaningful.
    pub match_position: u32,
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
        let mut raw_hits: Vec<(String, u32, u32)> = Vec::new();

        for (idx, doc) in self.documents.iter().enumerate() {
            let doc_id = idx as u32;
            let text = format!(
                "{} {}",
                doc.title.to_lowercase(),
                doc.content.to_lowercase()
            );
            let chars: Vec<char> = text.chars().collect();

            if chars.len() >= 3 {
                for i in 0..=chars.len() - 3 {
                    let tri: String = chars[i..i + 3].iter().collect();
                    raw_hits.push((tri, doc_id, i as u32));
                }
            }
        }

        raw_hits.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.cmp(&b.2))
        });

        let mut postings: HashMap<String, Vec<TrigramPosting>> = HashMap::new();
        let mut i = 0;
        while i < raw_hits.len() {
            let tri = raw_hits[i].0.clone();
            let mut doc_groups: Vec<(u32, Vec<u32>)> = Vec::new();
            let mut j = i;
            while j < raw_hits.len() && raw_hits[j].0 == tri {
                let doc_id = raw_hits[j].1;
                if doc_groups.last().map(|(id, _)| *id) != Some(doc_id) {
                    doc_groups.push((doc_id, Vec::new()));
                }
                doc_groups.last_mut().unwrap().1.push(raw_hits[j].2);
                j += 1;
            }
            postings.insert(
                tri,
                doc_groups
                    .into_iter()
                    .map(|(doc_id, positions)| TrigramPosting { doc_id, positions })
                    .collect(),
            );
            i = j;
        }

        let documents: Vec<DocMeta> = self
            .documents
            .iter()
            .map(|doc| DocMeta {
                id: doc.id.clone(),
                title: doc.title.clone(),
                url: doc.url.clone(),
                tags: doc.tags.clone(),
            })
            .collect();

        let docs_text: Vec<String> = self
            .documents
            .iter()
            .map(|doc| {
                format!(
                    "{} {}",
                    doc.title.to_lowercase(),
                    doc.content.to_lowercase()
                )
            })
            .collect();

        BuiltSearchIndex {
            postings,
            documents,
            docs_text,
        }
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrigramPosting {
    doc_id: u32,
    positions: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMeta {
    pub id: String,
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltSearchIndex {
    /// trigram (3-char, lowercased) -> list of (doc_id, positions in plain text)
    postings: HashMap<String, Vec<TrigramPosting>>,
    /// Per-doc metadata. No content field.
    documents: Vec<DocMeta>,
    /// Doc plain text (title + " " + content, lowercased) for Rust-side
    /// snippet extraction and content boost. Not used by the JS client.
    #[serde(skip)]
    docs_text: Vec<String>,
}

impl BuiltSearchIndex {
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let query_tris = extract_query_trigrams(&query_lower);
        if query_lower.is_empty() {
            return vec![];
        }

        let is_short = query_lower.len() < 3;

        let mut results: Vec<SearchResult> = Vec::new();

        for (idx, doc) in self.documents.iter().enumerate() {
            let doc_id = idx as u32;
            let mut matched = 0u32;
            // `min_position` is a char index into the doc's lowercased text.
            // `u32::MAX` is the uninitialised sentinel; the `matched == 0`
            // guard below ensures we never push a result with that value.
            // `SearchResult::match_position` is typed `u32` (not `Option<u32>`)
            // for stable JSON shape; see the field's doc comment.
            let mut min_position = u32::MAX;

            if is_short {
                // Short-query fallback: match any trigram whose key starts with
                // the query prefix. Note this scans the entire postings map,
                // i.e. O(unique_trigrams) per document. For indexes with tens
                // of thousands of unique trigrams (large corpora) this is the
                // hot bottleneck for short queries such as 1–2 char prefixes.
                // A sorted key list with binary search, or a trie, would
                // reduce this to O(log N) / O(|query|). Out of scope for the
                // current fix; flagging here for future work.
                let mut matched_tris: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for (tri_key, tri_postings) in &self.postings {
                    if tri_key.starts_with(&query_lower) {
                        for posting in tri_postings {
                            if posting.doc_id == doc_id && !matched_tris.contains(tri_key) {
                                matched += 1;
                                matched_tris.insert(tri_key.clone());
                                if let Some(&first_pos) = posting.positions.first()
                                    && first_pos < min_position
                                {
                                    min_position = first_pos;
                                }
                                break;
                            }
                        }
                    }
                }
            } else {
                for tri in &query_tris {
                    if let Some(postings) = self.postings.get(tri) {
                        for posting in postings {
                            if posting.doc_id == doc_id {
                                matched += 1;
                                if let Some(&first_pos) = posting.positions.first()
                                    && first_pos < min_position
                                {
                                    min_position = first_pos;
                                }
                                break;
                            }
                        }
                    }
                }
            }

            if matched == 0 {
                continue;
            }

            let recall = matched as f32 / query_tris.len() as f32;
            let mut score = 2.0 * recall;

            if doc.title.to_lowercase().contains(&query_lower) {
                score += 1.0;
            }

            if let Some(text) = self.docs_text.get(idx)
                && text.contains(&query_lower)
            {
                score += 0.5;
            }

            if matched > 0 {
                score += 0.3 * (1.0 / (1.0 + min_position as f32 / 1000.0));
            }

            let snippet = if let Some(text) = self.docs_text.get(idx) {
                Self::extract_snippet(text, min_position as usize)
            } else {
                String::new()
            };

            results.push(SearchResult {
                id: doc.id.clone(),
                title: doc.title.clone(),
                url: doc.url.clone(),
                tags: doc.tags.clone(),
                snippet,
                score,
                match_position: min_position,
            });
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    /// Extract a snippet of `content` centred on the matched position.
    ///
    /// `position` is a *character* index into the lowercased plain text (as
    /// produced by the trigram indexer), not a byte offset. We translate it
    /// to a byte offset using `char_indices` so that multi-byte UTF-8
    /// characters (e.g. "café", "🦀", "日本語") do not panic on
    /// `content[start..end]`. `start` and `end` are clamped to the nearest
    /// valid char boundary in case the surrounding 50/150-char window lands
    /// inside a multi-byte code point.
    fn extract_snippet(content: &str, position: usize) -> String {
        let char_count = content.chars().count();
        if char_count == 0 {
            return String::new();
        }
        let pos = position.min(char_count.saturating_sub(1));
        let target_start_char = pos.saturating_sub(50);
        let target_end_char = (pos + 150).min(char_count);

        // Convert char positions back to byte offsets, clamped to boundaries.
        let start_byte = floor_char_boundary(content, target_start_char);
        let end_byte = ceil_char_boundary(content, target_end_char);

        let mut snippet = content[start_byte..end_byte].to_string();
        if target_start_char > 0 {
            snippet = format!("...{snippet}");
        }
        if target_end_char < char_count {
            snippet = format!("{snippet}...");
        }
        snippet
    }

    pub fn into_serialized(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap_or_default()
    }

    pub fn from_serialized(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

fn extract_query_trigrams(query: &str) -> Vec<String> {
    let chars: Vec<char> = query.chars().collect();
    if chars.len() < 3 {
        if chars.is_empty() {
            return vec![];
        }
        return vec![chars.iter().collect::<String>()];
    }
    let mut trigrams = Vec::with_capacity(chars.len() - 2);
    for i in 0..=chars.len() - 3 {
        trigrams.push(chars[i..i + 3].iter().collect());
    }
    trigrams
}

/// Return the byte offset of the start of the `n`-th char in `s`.
/// If `n >= char count`, returns `s.len()`. Always lands on a char boundary.
fn floor_char_boundary(s: &str, n: usize) -> usize {
    s.char_indices()
        .nth(n)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(s.len())
}

/// Return the byte offset *one past* the `n`-th char (i.e. the byte after the
/// last char in the `0..=n` range). Clamped to a char boundary.
fn ceil_char_boundary(s: &str, n: usize) -> usize {
    s.char_indices()
        .nth(n)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(s.len())
}

pub trait SearchEngine: Send + Sync {
    fn add_document(&mut self, doc: SearchDocument);
    /// Run a search. Implementations should build the index lazily and cache
    /// it so repeated calls (e.g. on every keystroke in an interactive UI)
    /// are cheap. The `&mut self` receiver is what allows that caching
    /// without `RefCell`/`RwLock` on the consumer side.
    fn search(&mut self, query: &str, limit: usize) -> Vec<SearchResult>;
    fn clear(&mut self);
}

pub struct DefaultSearchEngine {
    documents: Vec<SearchDocument>,
    index: Option<BuiltSearchIndex>,
}

impl DefaultSearchEngine {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            index: None,
        }
    }

    /// Build the index (or return the already-built one) and consume self.
    pub fn build(mut self) -> BuiltSearchIndex {
        self.build_internal()
    }

    fn build_internal(&mut self) -> BuiltSearchIndex {
        if let Some(index) = self.index.take() {
            return index;
        }
        let mut search_index = SearchIndex::new();
        for doc in self.documents.drain(..) {
            search_index.add_document(doc);
        }
        search_index.build()
    }
}

impl Default for DefaultSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine for DefaultSearchEngine {
    fn add_document(&mut self, doc: SearchDocument) {
        self.documents.push(doc);
        // Invalidate the cached index; the next search() will rebuild.
        self.index = None;
    }

    fn search(&mut self, query: &str, limit: usize) -> Vec<SearchResult> {
        if self.index.is_none() {
            // Lazy build on first search; subsequent searches reuse the cache.
            // Cloning every document on every call (the old behaviour) is what
            // made interactive search O(N·docs) per keystroke.
            self.index = Some(self.build_internal());
        }
        self.index.as_ref().unwrap().search(query, limit)
    }

    fn clear(&mut self) {
        self.documents.clear();
        self.index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_no_content_duplication() {
        let body = "word ".repeat(5000 / 5);
        let mut index = SearchIndex::new();
        for i in 0..3 {
            index.add_document(SearchDocument {
                id: format!("doc{i}"),
                title: format!("Title {i}"),
                content: format!("Post {i} body {body}"),
                url: format!("/{i}"),
                tags: vec![],
            });
        }
        let built = index.build();
        let serialized = serde_json::to_vec(&built).unwrap();
        let json_str = String::from_utf8_lossy(&serialized);

        assert!(
            !json_str.contains("\"content\""),
            "Serialized JSON should not contain any content key"
        );

        let raw_size: usize =
            3 * (format!("Title {}", 0).len() + 1 + format!("Post {} body {}", 0, body).len());
        assert!(
            serialized.len() <= raw_size * 5,
            "Serialized index size {} should be <= {} (5x raw size)",
            serialized.len(),
            raw_size * 5
        );
    }

    #[test]
    fn test_trigram_substring_match() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Crates".to_string(),
            content: "rusty crates are great".to_string(),
            url: "/crates".to_string(),
            tags: vec![],
        });
        let built = index.build();
        let results = built.search("rust", 10);
        assert!(
            !results.is_empty(),
            "Query 'rust' should match 'rusty crates' via trigram overlap"
        );
        assert_eq!(results[0].id, "1");
    }

    #[test]
    fn test_trigram_typo_tolerance() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Guide".to_string(),
            content: "rust programming language".to_string(),
            url: "/rust".to_string(),
            tags: vec![],
        });
        let built = index.build();
        // "rusts" is an insertion typo; trigrams "rus" and "ust" overlap with "rust"
        let results = built.search("rusts", 10);
        assert!(
            !results.is_empty(),
            "Query 'rusts' (insertion typo) should match 'rust' via trigram overlap"
        );
    }

    #[test]
    fn test_exact_word_outranks_partial() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Rust".to_string(),
            content: "rust is great".to_string(),
            url: "/rust".to_string(),
            tags: vec![],
        });
        index.add_document(SearchDocument {
            id: "2".to_string(),
            title: "Rusty Tools".to_string(),
            content: "rusty tools are useful".to_string(),
            url: "/rusty".to_string(),
            tags: vec![],
        });
        let built = index.build();
        let results = built.search("rust", 10);
        assert!(!results.is_empty());
        assert_eq!(
            results[0].id, "1",
            "Exact match 'Rust' should outrank 'Rusty'"
        );
    }

    #[test]
    fn test_title_boost() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Introduction to WebAssembly".to_string(),
            content: "WebAssembly is a binary instruction format for a stack-based virtual machine"
                .to_string(),
            url: "/wasm".to_string(),
            tags: vec![],
        });
        index.add_document(SearchDocument {
            id: "2".to_string(),
            title: "Systems Programming Concepts".to_string(),
            content: "WebAssembly can be used for systems programming on the web".to_string(),
            url: "/systems".to_string(),
            tags: vec![],
        });
        let built = index.build();
        let results = built.search("webassembly", 10);
        assert!(!results.is_empty());
        assert_eq!(
            results[0].id, "1",
            "Doc with 'WebAssembly' in title should rank above doc with it only in body"
        );
    }

    #[test]
    fn test_snippet_anchored_to_match_position() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Doc".to_string(),
            content: "This is a long text about Rust programming language and tools".to_string(),
            url: "/doc".to_string(),
            tags: vec![],
        });
        let built = index.build();
        let results = built.search("rust", 10);
        assert!(!results.is_empty());
        assert!(
            results[0].match_position > 0,
            "match_position should be > 0"
        );
        assert!(
            results[0].snippet.to_lowercase().contains("rust"),
            "Snippet '{}' should contain 'rust'",
            results[0].snippet
        );
    }

    #[test]
    fn test_short_query_prefix_match() {
        // Queries shorter than 3 chars fall back to prefix matching against
        // the trigram key set (e.g. "ru" matches "rus", "rub", "rux", ...).
        // This is not a separate bigram index — it's the same trigram index
        // queried with a `starts_with` filter. The test name reflects that.
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Rust".to_string(),
            content: "rusty crates".to_string(),
            url: "/rust".to_string(),
            tags: vec![],
        });
        let built = index.build();
        let results = built.search("ru", 10);
        assert!(
            !results.is_empty(),
            "Query 'ru' (< 3 chars) should still return matches"
        );
    }

    #[test]
    fn test_search_handles_multibyte_utf8() {
        // Regression: `extract_snippet` used to slice on byte offsets derived
        // from char positions, panicking on multi-byte characters. This test
        // reproduces that case using "café" and a CJK string.
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Café".to_string(),
            content: "Le café est très agréable. 日本語のテキストも含む文書です。".to_string(),
            url: "/cafe".to_string(),
            tags: vec![],
        });
        let built = index.build();
        // Trigram search over lowercased text
        let results = built.search("café", 10);
        assert!(
            !results.is_empty(),
            "Multi-byte query 'café' should match 'Café' via trigram overlap"
        );
        // Snippet extraction must not panic on multi-byte content
        let snippet = &results[0].snippet;
        assert!(!snippet.is_empty(), "Snippet should be non-empty");
    }

    #[test]
    fn test_default_search_engine_accumulates() {
        let mut engine = DefaultSearchEngine::new();
        engine.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Hello".to_string(),
            content: "hello world".to_string(),
            url: "/hello".to_string(),
            tags: vec![],
        });
        engine.add_document(SearchDocument {
            id: "2".to_string(),
            title: "Rust".to_string(),
            content: "rust programming".to_string(),
            url: "/rust".to_string(),
            tags: vec![],
        });
        // First call: lazy build of the index. Second call: cached.
        let results = engine.search("hello", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "1");
        let results = engine.search("rust", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "2");
        // Third call should be cheap (cache hit).
        let results = engine.search("rust", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "2");
    }

    #[test]
    fn test_default_search_engine_invalidates_on_add() {
        // Adding a document after the index has been built must invalidate
        // the cache so the new doc is searchable.
        let mut engine = DefaultSearchEngine::new();
        engine.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Hello".to_string(),
            content: "hello world".to_string(),
            url: "/hello".to_string(),
            tags: vec![],
        });
        // Build via first search.
        let _ = engine.search("hello", 10);
        // Add a new doc — must invalidate cache.
        engine.add_document(SearchDocument {
            id: "2".to_string(),
            title: "Rust".to_string(),
            content: "rust programming".to_string(),
            url: "/rust".to_string(),
            tags: vec![],
        });
        let results = engine.search("rust", 10);
        assert!(
            !results.is_empty(),
            "Doc added after first search should be searchable"
        );
        assert_eq!(results[0].id, "2");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut index = SearchIndex::new();
        index.add_document(SearchDocument {
            id: "1".to_string(),
            title: "Test Document".to_string(),
            content: "Some test content here".to_string(),
            url: "/test".to_string(),
            tags: vec!["test".to_string()],
        });
        let built = index.build();
        let serialized = built.into_serialized();
        let deserialized = BuiltSearchIndex::from_serialized(&serialized);
        assert!(deserialized.is_some());
        let results = deserialized.unwrap().search("test", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "1");
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
        // Use trigrams that don't overlap with common English text
        let results = built.search("zzz qqq xxx", 10);
        assert!(results.is_empty());
    }
}
