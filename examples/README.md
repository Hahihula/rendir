# Rendir Examples

This folder contains working examples for each Rendir feature.

## Available Examples

### 1. Search (`search/`)
Demonstrates Rendir's built-in search engine.
- Pure Rust implementation (WASM compatible)
- Pre-built index at compile/build time
- Title boosting and snippet extraction
- `SearchEngine` trait for pluggable backends

**Content**: `examples/search/content/` (sample documents)

### 2. Slideshow (`slideshow/`)
Demonstrates reveal.js-style presentation creation.
- `:::slide` component or heading-level-2 fallback
- Keyboard + mouse + touch navigation
- Presenter view with timer and speaker notes
- WebRTC sync (toggle per session)

**Content**: `examples/slideshow/content/presentation.md`

### 3. Internationalization (`i18n/`)
Demonstrates multi-language site support with automatic fallback.
- Languages detected from directory structure (`en/`, `de/`, `cs/`, `zh/`)
- Automatic fallback to default language when translation is missing
- Language selector component
- SEO-friendly hreflang meta tags

**Content**: `examples/i18n/content/` (English, German, Czech, Chinese)

## What's Next (Planned Examples)

The following examples will be added as their features are implemented:

### 3. mdbook Ingestion (`mdbook/`)
- Import existing mdbook projects
- Parse `book.toml` and `SUMMARY.md`
- Generate nav tree from chapter structure

### 4. Blog (`blog/`)
- Blog post discovery and metadata extraction
- Pretty URLs (`/blog/post-title`)
- Pagination, categories, tags
- Custom MD-driven pages

### 5. Cookbook (`cookbook/`)
- Recipe discovery with folder-based categories
- `:::ingredients` component with amount parsing
- Live portion recalculation (WASM interactive)
- Tag browsing and category pages

---

## Using Examples

Each example folder contains:
- `README.md` — full documentation for that feature
- `content/` — sample input files

Run the CLI from the project root:

```bash
# Build slideshow
cargo run -p rendir -- build \
  --input examples/slideshow/content/presentation.md \
  --output /tmp/rendir-output/ \
  --template slideshow

# Convert with custom template
cargo run -p rendir -- convert \
  --input examples/slideshow/content/presentation.md \
  --output /tmp/presentation.html \
  --template slideshow
```

## Updating Examples

After implementing each phase (per `PLAN.md`), the relevant example is updated:
1. Add/modify the example content
2. Update the `README.md` with new capabilities
3. Verify `cargo test` passes

See `PLAN.md` for the full implementation roadmap.
