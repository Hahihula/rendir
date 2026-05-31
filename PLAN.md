# Rustpress Roadmap — Feature Plan

---

## Template Customization (All Projects)

Users can create custom themes by deriving from rustpress default templates.

### How It Works
1. Copy the desired template folder (e.g., `blog/`, `slideshow/`) to their own folder
2. Modify CSS, HTML structure, components as needed
3. Pass the custom template folder via CLI: `--template my-theme/blog`
4. Custom template folder structure mirrors rustpress default: same template filenames expected

### Template Hierarchy
```
rustpress default templates/
├── blog/
│   ├── index.html
│   ├── post.html
│   └── ...
├── slideshow/
│   ├── index.html
│   └── ...
└── ...
```

### Custom Template Usage
```bash
# Blog with custom theme
rustpress build --input blog-content/ --output site/ --template my-themes/blog

# Slideshow with custom theme
rustpress build --input slides/ --output site/ --template my-themes/slideshow
```

### What Users Can Customize
- **CSS**: Override or extend default styles
- **Layout**: Modify HTML structure within templates
- **Components**: Add/replace component implementations
- **Variables**: Use Tera variables (`{{ title }}`, `{{ content | safe }}`, etc.)
- **Static assets**: Include custom JS, images alongside templates

### Template Variables Available
| Project | Variables |
|---------|-----------|
| All | `{{ content | safe }}`, `{{ title }}` |
| Blog | `{{ author }}`, `{{ date }}`, `{{ tags }}`, `{{ category }}`, `{{ prev_post }}`, `{{ next_post }}` |
| Cookbook | `{{ serves }}`, `{{ category }}`, `{{ tags }}`, `{{ ingredients }}` |
| Slideshow | `{{ slide_count }}`, `{{ slide_current }}` |
| mdbook | `{{ chapter_title }}`, `{{ prev_chapter }}`, `{{ next_chapter }}`, `{{ nav_tree }}` |

### Documentation for Custom Templates
Each example folder's README.md includes a **"Creating Custom Templates"** section explaining:
- Which template files exist and what each does
- Required Tera variables and filters
- CSS class conventions for components
- How to override specific parts while keeping others

---

## Research Tasks (Cross-cutting)
1. **FlexSearch WASM compatibility** — Verify FlexSearch can be compiled to WASM and index can be pre-built at build time from a static site generator environment. Also evaluate alternatives: **Fuse.js**, **TinySearch**, **Lunr.js**. Must support pre-built static index + fuzzy search.
2. **WebRTC library for WASM** — Identify lightweight Rust/WASM WebRTC library for presenter sync (iced/screenshot? or pure webrtc crate?)
3. **Search fallback plan** — Prepare abstraction layer so search engine can be swapped without re-implementing all index builders. If FlexSearch fails WASM compatibility, switch to alternative.

---

## Goal 1: Slideshow

### Slide Format
- HTML reveal.js-style structure (div-based slides)
- Each slide authored in **Markdown**, converted to HTML
- Slideshow container template: `slideshow/index.html`

### Slide Delimiter (source Markdown)
- Primary: `:::slide` component block with title
- Fallback: heading level 2 (`## Slide Title`) as implicit slide break

### Components Needed
1. **`slide`** — wraps slide content, accepts `title` attribute
   - Renders as `<div class="slide" data-slide-n>...</div>`
2. **`presenter-view`** — optional embeddable presenter notes panel
3. **`timer`** — countdown timer for presentations

### Navigation
- Keyboard (arrow keys, space)
- Mouse click
- Touch/swipe gestures
- Presenter view with notes + timer + prev/next
- WebRTC sync (toggle per session, host-controlled)

### WebRTC Sync
- Host starts session, gets a room code
- Audience connects via code (peer-to-host model)
- Host can toggle sync on/off
- Uses lightweight WebRTC (likely `webrtc` crate or similar)

### Search
- Pre-built FlexSearch index embedded in WASM
- Search accessible from slideshow mode

### WASM Compatibility
- Full slideshow must work in browser from WASM
- Slide transitions, navigation, WebRTC — all client-side

---

## Goal 2: mdbook Ingestion

### Ingestion Flow
1. Detect `book.toml` in input folder
2. Parse `book.toml` for structure config
3. Parse `SUMMARY.md` for chapter ordering (if present)
4. Fallback to filesystem order if no SUMMARY.md
5. Render each `.md` chapter with rustpress template
6. Generate prev/next navigation from parsed structure

### Structure Priority
1. `book.toml` (primary config source)
2. `SUMMARY.md` (chapter order and hierarchy)
3. Filesystem (fallback for unordered chapters)

### mdbook Templates (mdbook/ folder)
- `mdbook/index.html` — Book index/landing
- `mdbook/chapter.html` — Individual chapter
- Supports all mdbook features: cross-refs, Rust code blocks/admonitions, footnotes, etc.
- Navigation tree from SUMMARY.md

### Output
- Static HTML site matching mdbook structure
- Index page, chapter pages, nav sidebar

---

## Goal 3: Blog

### Post Discovery
- Scan input folder recursively for `.md` files
- Include if: has frontmatter AND `publish: true` OR has date in frontmatter
- `draft: true` in frontmatter → exclude

### URL Structure
- Pretty URLs: `/blog/post-title` (generates `post-title/index.html`)

### Frontmatter Fields
- `title`, `date`, `author`, `tags`, `category`
- `publish` (boolean, default false)
- `draft` (boolean, default false)
- `template` (optional override)

### Date Detection
- Date from frontmatter `date` field
- Falls back to file modification time

### Blog Templates (blog/ folder)
1. **`blog/index.html`** — Blog landing (recent posts + static hero content)
2. **`blog/post.html`** — Individual post
   - Header: title, date, author, tags
   - Content: `{{ content | safe }}`
   - Footer: prev/next post links, related posts by tag
3. **`blog/about.html`** — Custom MD-driven about/contact page
   - User specifies `template: blog-about` in frontmatter
4. **`blog/category.html`** — Category archive
5. **`blog/tag.html`** — Tag archive
6. **`blog/archive.html`** — Date-based archive

### Pagination
- Numbered pages: `/blog/page/2`, `/blog/category/rust/page/2`
- Configurable posts per page (default 10)

### Search
- Pre-built FlexSearch index (same WASM approach)
- Accessible from blog landing

### Theme Styling
- Clean, readable blog theme
- Responsive
- Dark/light mode toggle (optional stretch)

---

## Goal 4: Cookbook

### Recipe Discovery
- Scan input folder recursively for `.md` files
- Category: folder path + frontmatter `category` (fallback logic)
- Tags: frontmatter + auto-extract from content

### URL Structure
- Pretty URLs: `/cookbook/category/recipe-slug`

### Recipe Structure (Markdown + Frontmatter)
```yaml
---
title: "Chocolate Cake"
category: "desserts"
tags: ["chocolate", "baking", "dessert"]
serves: 4
---
```

### Ingredient Component Syntax
```markdown
:::ingredients
* 3 eggs
* 150g flour
* 100g sugar
:::
```
- `ingredients` component parses list items, detects amount patterns (e.g., `150g`, `1/2 tsp`, `3 cups`)
- Amounts stored as {value, unit} tuples for ratio recalculation
- Markdown inside list items supported (bold, italic, links)
- Steps: just ordinary ordered list (`1. Step one...`) — no special component needed
- If `title` missing from frontmatter, first heading (`# Title`) in document becomes title

### Serving Adjustment
- `serves` in frontmatter = base serving count
- `serving-calculator` component: user adjusts servings via slider
- Ratio factor: `new_servings / base_servings`
- All ingredient amounts multiply by ratio factor
- Metric/Imperial: stored as-is, ratio-only math (no unit conversion needed)
- Works as interactive WASM component (no server)

### Components Needed
1. **`recipe`** — full recipe card with ingredients, steps, nutrition
2. **`serving-calculator`** — interactive slider for portion scaling
3. **`ingredient-list`** — parsed and recalculatable ingredient list

### Cookbook Templates
1. **`cookbook-landing.html`** — Index page
   - Category navigation
   - Search bar (FlexSearch)
   - Featured/recent recipes
2. **`cookbook-category.html`** — Category page with recipe list
3. **`cookbook-recipe.html`** — Full recipe with live recalculator
4. **`cookbook-archive.html`** — Tag-based archive

### Search
- Pre-built FlexSearch index
- Full-text fuzzy search across recipe titles, ingredients, tags

### Tag Browsing
- Tags extracted from frontmatter AND auto-detected from content
- Browsable tag pages `/cookbook/tag/tag-name`
- Tag cloud on cookbook index

---

## Search (All Projects)

### Engine
- **FlexSearch** — fast, WASM-compatible, fuzzy search
- Index pre-built at build time, embedded in WASM binary
- Supports incremental updates if needed

### Index Building
- At static site build time: parse all content, build FlexSearch index JSON
- Index embedded in WASM module
- Search UI component: input field + results dropdown/modal

### Search Scope by Project
- Slideshow: search across slide titles + content
- Blog: search titles + content + tags
- Cookbook: search titles + ingredients + tags
- mdbook: search chapter titles + content

---

## Summary: Templates & Components

### New Templates
Organized in folders by project:

| Template Path | Project | Description |
|--------------|---------|-------------|
| `slideshow/index.html` | Slideshow | Main slideshow viewer |
| `slideshow/presenter.html` | Slideshow | Presenter view with notes + timer |
| `mdbook/index.html` | mdbook | Book index/landing |
| `mdbook/chapter.html` | mdbook | Individual chapter |
| `blog/index.html` | Blog | Blog landing (recent posts + static hero) |
| `blog/post.html` | Blog | Individual post |
| `blog/about.html` | Blog | Custom MD-driven page (template: blog-about) |
| `blog/category.html` | Blog | Category archive |
| `blog/tag.html` | Blog | Tag archive |
| `blog/archive.html` | Blog | Date-based archive |
| `cookbook/index.html` | Cookbook | Cookbook index with search |
| `cookbook/category.html` | Cookbook | Category page with recipe list |
| `cookbook/recipe.html` | Cookbook | Full recipe with live calculator |
| `cookbook/archive.html` | Cookbook | Tag archive |
| `shared/search.html` | All | Search overlay/modal |

### New Components
| Component | Project | Purpose |
|-----------|---------|---------|
| `slide` | Slideshow | Individual slide wrapper |
| `presenter-view` | Slideshow | Presenter notes panel |
| `timer` | Slideshow | Countdown timer |
| `ingredients` | Cookbook | Parses ingredient list, detects amounts, supports live recalc |
| `serving-calculator` | Cookbook | Interactive slider for portion scaling |
| `search` | All | Search input + results overlay |

### Metadata Extraction
| Project | Fields |
|---------|--------|
| Blog | title, date, author, tags, category, publish, draft |
| Cookbook | title, category, tags, serves, ingredients*, steps* |
| Slideshow | title, slides (count) |
| mdbook | chapter title, order, prev/next |

---

## Testing Requirements

Every functionality must be covered by tests. Testing approach:

- **Unit tests**: Test individual parsers, renderers, components
- **Integration tests**: Test end-to-end conversion flows
- **HTML balance tests**: All component HTML output must be balanced (no unclosed tags)
- **Template tests**: Render known Markdown with each template, verify output structure

Test naming convention: `test_<feature>_<scenario>`

All tests live in `rustpress-core/src/lib.rs` alongside existing 14 tests. New tests added per feature:

| Feature | Tests |
|---------|-------|
| Slideshow | `test_slide_component`, `test_presenter_view`, `test_timer`, `test_slide_syntax_fallback`, `test_webrtc_sync` |
| mdbook | `test_mdbook_book_toml`, `test_mdbook_summary_parsing`, `test_mdbook_chapter_order`, `test_mdbook_navigation_generation` |
| Blog | `test_blog_post_discovery`, `test_blog_url_structure`, `test_blog_pagination`, `test_blog_metadata_extraction`, `test_blog_custom_template_page` |
| Cookbook | `test_cookbook_recipe_discovery`, `test_cookbook_ingredients_parsing`, `test_cookbook_portion_calc`, `test_cookbook_category_inference`, `test_cookbook_tag_extraction` |
| Search | `test_search_index_builder`, `test_search_fuzzy_match`, `test_search_engine_swap` |

---

## Examples

Each feature has a working example in `examples/` folder with its own `README.md`.

Structure:
```
examples/
├── README.md                          # Overview of all examples
├── slideshow/
│   ├── README.md                      # How to use slideshow, all options
│   ├── content/
│   │   └── presentation.md           # Sample slideshow content
│   └── output/                        # Expected rendered output
├── mdbook/
│   ├── README.md                     # How to ingest mdbook project
│   └── content/
│       ├── book.toml
│       ├── SUMMARY.md
│       └── src/
│           └── chapter_1.md
├── blog/
│   ├── README.md                     # How to use blog, frontmatter options
│   └── content/
│       ├── landing.md
│       ├── posts/
│       │   ├── 2024-01-01-welcome.md
│       │   └── 2024-02-15-features.md
│       └── about.md
├── cookbook/
│   ├── README.md                     # How to use cookbook, recipe syntax
│   └── content/
│       └── desserts/
│           └── chocolate-cake.md
└── search/
    └── README.md                     # How search works across all projects
```

### Example README Template
Each example folder's `README.md` includes:
- **What this example demonstrates**
- **Frontmatter options** (all available fields)
- **Component syntax** (for relevant components)
- **CLI commands** to run it
- **Expected output**

### Example Update Policy
After each successful step (feature complete + tests passing), update the relevant example:
1. Add/modify the example Markdown content
2. Update the example's `README.md` to reflect new capabilities
3. Document newly added options, components, CLI flags
4. Run `cargo test` to ensure nothing regressed

If a feature is incomplete (work in progress), add a `IN_PROGRESS.md` note in that example folder.

---

## Implementation Order (Suggested)

### Phase 1: Search Infrastructure ✅
- ✅ Research FlexSearch WASM compatibility + alternatives
  - **FlexSearch**: JS library (not Rust) — works in browser/Node but not directly in Rust WASM. Would require wasm-bindgen wrapper. No pre-built index support documented.
  - **TinySearch** (mre/tinysearch on crates.io): Pure Rust! edition 2024, has lib with `has_lib: true`. Great WASM candidate — 1150 LOC, bloom filter based. **Best Rust-native option found.**
  - **Fuse.js**: Pure JS — not directly usable from Rust WASM without JS interop wrapper
  - **Lunr.js**: Pure JS — same issue as Fuse.js
  - **Conclusion**: Default engine is our own pure-Rust implementation. TinySearch is a strong candidate for Phase 1b if we want a more sophisticated engine. The abstraction layer is ready for engine swapping.
- ✅ Implement search abstraction layer (`SearchEngine` trait in `search.rs`)
- ✅ Implement pre-built index builder (`SearchIndex`, `BuiltSearchIndex`)
- ✅ `search` component + `shared/search.html` template
- ✅ `examples/search/` with README.md
- ✅ Search tests (5 new tests added)

**Note**: Default engine is our own pure-Rust implementation. TinySearch integration is optional Phase 1b.

### Phase 2: Slideshow ✅
- ✅ `slide` component
- ✅ `slideshow/index.html` and `slideshow/presenter.html` templates
- ✅ Navigation (keyboard + mouse + touch)
- ✅ Presenter view + timer
- ✅ WebRTC sync (stub — uses BroadcastChannel for same-origin tabs; full WebRTC requires a lightweight signaling service)
- ✅ `examples/slideshow/` with README.md
- ✅ Slideshow tests (3 new tests)

**WebRTC Note**: The `webrtc` crate exists but is not WASM-compatible. Current implementation uses `BroadcastChannel` API which works for same-origin browser tabs (presenter and audience in same browser/device). For production cross-origin WebRTC, a lightweight signaling service would be needed — this is a Phase 2b stretch goal.

### Phase 3: mdbook Ingestion ✅
- ✅ `book.toml` parser (`BookToml` struct with toml parsing)
- ✅ `SUMMARY.md` parser (`Summary` struct with chapter hierarchy)
- ✅ `mdbook/index.html` template (sidebar nav + chapter content + prev/next links)
- ✅ `examples/mdbook/` with README.md + sample project (book.toml, SUMMARY.md, src/ chapters)
- ⚠️ Navigation tree generator (partial — template includes nav_tree variable, but CLI doesn't yet wire up the full build pipeline for mdbook projects)

**Note**: The parsers, templates, and example are complete. The CLI's `build` command needs extension to auto-detect `book.toml` in input folder and route to mdbook processing. This is a Phase 3b task.

### Phase 4: Blog
- Post discovery + metadata extraction
- All blog templates (blog/, folder structure)
- Pagination
- Blog theme styling
- `examples/blog/` with README.md

### Phase 5: Cookbook
- `ingredients` component (parsing + amount detection)
- `serving-calculator` component (WASM interactive)
- `cookbook` templates
- Tag extraction + browsing
- Category pages

---

## Technical Notes

- **FlexSearch**: Check if `wasm-bindgen` compatible build exists. Alternative: build FlexSearch index as static JSON at build time, load in WASM at runtime.
- **WebRTC**: `webrtc` crate is pure Rust but heavy. Consider `ice` crate for ICE/STUN/TURN + manual signaling, or a lightweight signaling service for the presentation use case.
- **Portion math**: Only ratio matters. User edits `1cup` to `1.5cup` → factor `1.5`. All values multiply by same factor. No unit conversion needed — just proportional scaling.
- **Pretty URLs**: Generate `post-title/index.html` for each post. Server rewrite rules needed for static hosting (or `_redirects` for Netlify, `vercel.json`, etc.).
- **Image handling**: Non-markdown files (images, etc.) in the input folder are copied to the output folder during build. Images referenced via Markdown `![alt](path)` are resolved relative to the source file location. **Nice-to-have**: Base64-encoded inline images — on build, detect images used multiple times and embed as JS variables to avoid repeated HTTP requests.