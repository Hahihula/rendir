# Search Example

This example demonstrates how rustpress's built-in search works.

## Overview

rustpress includes a **pure Rust search engine** that works in both WASM and native environments. The search index is built at compile/build time and can be serialized to JSON for embedding in WASM binaries.

**Key features:**
- Pure Rust implementation (no external JS dependencies in WASM)
- Pre-built index at build time
- Title boosting (matches in titles score higher)
- Snippet extraction with query highlighting
- Pluggable engine abstraction (can swap engines)
- WASM-compatible serialization

## How It Works

### 1. Index Building

Documents are added to a `SearchIndex`:

```rust
let mut index = SearchIndex::new();
index.add_document(SearchDocument {
    id: "1".to_string(),
    title: "My Post".to_string(),
    content: "Post content here...".to_string(),
    url: "/blog/my-post".to_string(),
    tags: vec!["rust".to_string(), "blog".to_string()],
});
let built = index.build();
```

### 2. Serialization

The built index serializes to JSON for embedding in WASM:

```rust
let serialized = built.into_serialized(); // Vec<u8>
let deserialized = BuiltSearchIndex::from_serialized(&serialized);
```

### 3. Searching

```rust
let results = index.search("rust tutorial", 10);
// Returns Vec<SearchResult> sorted by relevance score
```

## SearchDocument Structure

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique document identifier |
| `title` | String | Document title (boosted in search) |
| `content` | String | Full text content |
| `url` | String | URL/path to the document |
| `tags` | Vec<String> | Associated tags |

## SearchResult Structure

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Matching document ID |
| `title` | String | Document title |
| `url` | String | Document URL |
| `snippet` | String | Content snippet around match |
| `score` | f32 | Relevance score |

## Search Engine Abstraction

The `SearchEngine` trait allows swapping implementations:

```rust
pub trait SearchEngine: Send + Sync {
    fn add_document(&mut self, doc: SearchDocument);
    fn search(&self, query: &str, limit: usize) -> Vec<SearchResult>;
    fn clear(&mut self);
}
```

The default `DefaultSearchEngine` uses `BuiltSearchIndex`. You can implement your own engine (e.g., for FlexSearch WASM integration) as long as it satisfies the trait.

## WASM Integration

The search index is built at rustpress build time and serialized. In WASM:
1. The serialized index bytes are embedded in the WASM binary
2. On page load, the index is deserialized
3. Search queries run entirely client-side in the browser

This means **no server required** — the search works from static hosting.

## Example CLI Usage

```bash
# Build with search index
cargo run -p rustpress-cli -- build --input content/ --output site/

# The search index is embedded automatically
```

## Creating Custom Templates

The search component can be embedded in any template. See `shared/search.html` for the search overlay/modal UI.

### Required Variables
None — search is fully client-side after initial page load.

### CSS Classes
| Class | Purpose |
|-------|---------|
| `.search-input` | Search input field |
| `.search-results` | Results container |
| `.search-result` | Individual result item |
| `.search-result-title` | Result title |
| `.search-result-snippet` | Result snippet |

### JavaScript API
When the search index is available:
```js
// Search index is available as a global or via WASM export
const results = searchIndex.search("query", 10);
```

## Future: FlexSearch Integration

The plan includes a research task to evaluate FlexSearch WASM compatibility. If FlexSearch works with WASM and pre-built indexes, it can replace the default engine via the `SearchEngine` abstraction — without changing any consumer code.

See `PLAN.md` for details on the search engine swap research task.