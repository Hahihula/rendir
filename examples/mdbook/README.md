# mdbook Ingestion Example

This example demonstrates Rendir's ability to ingest an existing mdbook project.

## Overview

Rendir can process an mdbook project (with `book.toml` and `SUMMARY.md`) and render it using its own templates. This allows you to:

- Customize the visual design of your documentation
- Add Rendir components (alerts, tabs, etc.) to existing content
- Integrate mdbook content into your own build pipeline

## File Structure

```
examples/mdbook/
├── README.md           # This file
├── book.toml          # mdbook configuration
├── SUMMARY.md         # Chapter ordering
└── src/
    ├── intro.md
    ├── getting-started.md
    ├── installation.md
    ├── first-project.md
    ├── advanced.md
    ├── configuration.md
    ├── deployment.md
    └── faq.md
```

## How It Works

### 1. Book Configuration (`book.toml`)

```toml
[book]
title = "Example Book"
author = "Rendir Team"
description = "A sample mdbook project"
src = "src"
```

### 2. Chapter Ordering (`SUMMARY.md`)

The `SUMMARY.md` file defines the book's structure:

```markdown
# Summary

- [Introduction](./src/intro.md)
- [Getting Started](./src/getting-started.md)
  - [Installation](./src/installation.md)
  - [First Project](./src/first-project.md)
- [Advanced Topics](./src/advanced.md)
  - [Configuration](./src/configuration.md)
  - [Deployment](./src/deployment.md)
- [FAQ](./src/faq.md)
```

### 3. Navigation Tree

Rendir parses `SUMMARY.md` to build a navigation tree. Chapters can be nested (indicated by indentation) to create sections.

## CLI Usage

```bash
# Build an mdbook project
cargo run -p rendir -- build \
  --input examples/mdbook/content/ \
  --output site/ \
  --template mdbook

# Single file conversion
cargo run -p rendir -- convert \
  --input examples/mdbook/content/src/intro.md \
  --output intro.html \
  --template mdbook
```

## Vue Store Data Structure

Rust provides data to the mdbook Vue template via a `MdBookStore` object embedded in the page:

```rust
// MdBookStore structure (from rendir-core/src/types.rs)
MdBookStore {
    title: String,                    // Book title from book.toml
    chapters: Vec<ChapterStore>,      // All chapters for sidebar navigation
    current_chapter: ChapterStore,   // Current chapter data
}

ChapterStore {
    title: String,                   // Chapter title
    url: String,                     // URL/path to this chapter
    content: String,                 // Rendered HTML content
    level: usize,                    // Nesting level (for indentation)
    children: Vec<ChapterStore>,     // Nested sub-chapters
    prev_chapter: Option<ChapterNav>, // Previous chapter link
    next_chapter: Option<ChapterNav>, // Next chapter link
}

ChapterNav {
    title: String,                   // Previous/next chapter title
    url: String,                     // Previous/next chapter URL
}
```

### Template Integration

The Vue template receives store data via a `<script id="store-data">` tag:

```html
<script id="store-data" type="application/json">{{STORE_DATA}}</script>
```

Vue then renders the sidebar navigation, prev/next links, and chapter content reactively.

## Generated Output

When building, Rendir creates:
- `index.html` — Book index/landing page (first chapter)
- Chapter files (`intro/index.html`, etc.) — Each chapter as a separate HTML file
- Sidebar navigation with links to all chapters

## Creating Custom mdbook Templates

Copy the default template folder:

```
my-templates/mdbook/
└── index.html   # Main book template
```

### Vue SPA Template Requirements

The mdbook template uses Vue SPA architecture with embedded store data:

**Required element:**
```html
<script id="store-data" type="application/json">{{STORE_DATA}}</script>
```

**Vue receives this data structure:**
```javascript
{
  title: "Book Title",
  chapters: [{ title, url, content, level, children, prev_chapter, next_chapter }, ...],
  current_chapter: { title, url, content, level, children, prev_chapter, next_chapter }
}
```

**CSS Classes for Navigation**

| Class | Purpose |
|-------|---------|
| `.chapter-list` | Navigation list container |
| `.section` | Nested chapter (indented) |

## CLI Flags

| Flag | Description |
|------|-------------|
| `--input` | Input folder (must contain `book.toml` and `SUMMARY.md`) |
| `--output` | Output directory |
| `--template mdbook` | Use the built-in mdbook template |

## Notes

- Rendir uses the `src` directory from `book.toml` to resolve chapter paths
- Images referenced in Markdown are copied to the output (same behavior as slideshow)
- The navigation tree preserves chapter hierarchy from `SUMMARY.md`
