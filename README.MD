# Rustpress

<div align="center">
<img src="./rustpress_logo.png" height="300" />
</div>

<div align="center">

[![Live Demo](https://img.shields.io/badge/GitLab%20Pages-Live%20Demo-FFC61B?style=for-the-badge&logo=gitlab)](https://hahihula.gitlab.io/rustpress)

</div>

Rustpress is a flexible static site generator written in Rust that converts Markdown content into static websites. It uses a **Vue SPA architecture** — Rust parses and provides data as JSON, and Vue.js handles all rendering and interactivity on the client side. A built-in **dev server with live reload** accelerates the development workflow.

## Architecture

```
Markdown → Rust: parse Markdown + components → Vue Store JSON → Vue SPA renders UI
```

Rust processes Markdown and outputs a **Vue Store** (JSON) embedded in the HTML. Vue.js reads this data and handles all rendering, filtering, and interactivity. This keeps the Rust side simple (data processing only) and the client side reactive.

## Templates

Rustpress comes with three built-in templates, each using the Vue SPA approach:

| Template | CLI Name | Description |
|----------|----------|-------------|
| **Blog** | `blog` | Blog landing with post cards, tag filtering, search |
| **Slideshow** | `slideshow` | Presentation slides with keyboard/touch nav, presenter view |
| **mdbook** | `mdbook` | Book/documentation with sidebar, prev/next, search |

All templates embed store data via `<script id="store-data" type="application/json">{{STORE_DATA}}</script>` and render with Vue 3.

## Features

- 📝 Markdown parsing with extended syntax
- 🧩 Custom component system (Alert, YouTube, Tabs, Slide, Search, etc.)
- 🖼️ **Vue SPA templates** — Blog, Slideshow, mdbook
- 🔍 **Full-text search** — Inverted index built at build time, searched client-side
- ⚡ **Dev server with live reload** — File watching + auto-rebuild + browser refresh
- 🌐 **Internationalization (i18n)** — Multi-language support with automatic fallback
- 🌐 WebAssembly compatibility
- 📂 Directory-based content organization
- 🔌 Extensible architecture with component registry

## Installation

### From Source

```bash
# Clone the repository
git clone https://gitlab.com/hahihula/rustpress.git
cd rustpress

# Build the project
cargo build --release

# Install the CLI tool (optional)
cargo install --path rustpress
```

## Usage

### CLI Tool

The CLI provides three main commands: `convert` for single files, `build` for site generation, and `dev` for development with live reload.

#### Build a Blog

```bash
cargo run -p rustpress -- build \
  --input examples/blog/content/ \
  --output /tmp/blog/ \
  --template blog
```

#### Build a Slideshow

```bash
cargo run -p rustpress -- build \
  --input examples/slideshow/content/ \
  --output /tmp/slideshow/ \
  --template slideshow
```

#### Build an mdbook Project

```bash
cargo run -p rustpress -- build \
  --input examples/mdbook/content/ \
  --output /tmp/mdbook/ \
  --template mdbook
```

#### Convert a Single File

```bash
# Basic conversion
rustpress convert --input example.md --output example.html

# Using a template
rustpress convert --input example.md --output example.html --template slideshow
```

#### Development Server with Live Reload

The `dev` command starts a development server with automatic rebuilding and live reload:

```bash
cargo run -p rustpress -- dev \
  --input examples/blog/content/ \
  --output /tmp/blog/ \
  --template blog
```

**Features:**
- **File watching** — watches both input directory and template directory for changes
- **Auto-rebuild** — rebuilds the site when source files change
- **Live reload** — browser automatically refreshes when rebuild completes
- **Debounced** — 200ms debounce prevents rapid rebuilds during batch edits

The server serves on `http://localhost:3000` by default. After each rebuild, a version counter increments and all open browser tabs poll this counter — when it changes, the page reloads automatically.

### Custom Components

Rustpress supports two syntaxes for custom components:

#### HTML-like Syntax

```markdown
<Alert type="info" title="Information">
This is an informational alert.
</Alert>
```

#### Special Markdown Syntax

```markdown
:::alert{type="warning" title="Warning"}
This is a warning alert.
:::
```

### Built-in Components

| Component | Syntax | Description |
|-----------|--------|-------------|
| Alert | `<Alert type="info">...</Alert>` | Callout boxes (info, warning, error) |
| YouTube | `<YouTube id="dQw4w9WgXcQ" />` | Embed YouTube videos |
| Tabs | `<Tabs>## Tab\nContent</Tabs>` | Tabbed content |
| Slide | `<Slide title="..." type="full">...</Slide>` | Slideshow slides |
| Search | `<Search placeholder="..." />` | Search interface |

#### Alert

```markdown
<Alert type="info" title="Information">
This is an informational alert.
</Alert>
```

Types: `info`, `warning`, `error`

#### YouTube

```markdown
<YouTube id="dQw4w9WgXcQ" />
```

#### Tabs

```markdown
<Tabs>
## Tab 1
Content for tab 1

## Tab 2
Content for tab 2

## Tab 3
Content for tab 3
</Tabs>
```

### Frontmatter

You can add metadata to your Markdown files using YAML frontmatter:

```markdown
---
title: My Page Title
author: Jane Smith
date: 2024-01-15
tags: [rust, static-site, markdown]
---

# Content starts here
```

### Vue Store Data

Each template receives its data via an embedded JSON script tag:

```html
<script id="store-data" type="application/json">{{STORE_DATA}}</script>
```

Vue parses this and uses it as reactive store data.

#### SlideshowStore

```javascript
{
  title: "Presentation Title",
  slides: [
    { id: 0, type: "full", title: "Slide", content: "<p>HTML</p>", notes: "", images: [] }
  ],
  current_slide_index: 0,
  images: {}
}
```

#### BlogIndexStore

```javascript
{
  title: "Blog Title",
  description: "A blog about...",
  posts: [{ id, title, date, author, excerpt, tags, url }],
  tags: [{ name: "rust", count: 3 }],
  recent_posts: [...],
  search_index: "{...}",  // JSON-serialized search index
  languages: [            // Available languages for i18n
    { code: "en", name: "English", is_default: true },
    { code: "de", name: "Deutsch", is_default: false }
  ]
}
```

#### MdBookStore

```javascript
{
  title: "Book Title",
  chapters: [{ title, url, content, level, children, prev_chapter, next_chapter }],
  current_chapter: { title, url, content, ... },
  search_index: "{...}"  // JSON-serialized search index
}
```

## Full-Text Search

Rustpress builds a **full-text search index** at build time:

1. **Rust** parses all documents, builds an inverted index (`BuiltSearchIndex`)
2. **Index** is serialized to JSON and embedded in the store's `search_index` field
3. **Client side** — a shared `ClientSearchIndex` class (in `templates/shared/search.js`) performs ranked search with:
   - Exact word matches (+1.0)
   - Title matches (+2.0)
   - Content matches (+1.0)
   - Snippet extraction with context

Both **blog** and **mdbook** templates use this same shared search component.

## Internationalization (i18n)

Rustpress supports multi-language sites with automatic language detection and fallback.

> **Note:** mdbook i18n works differently and is not yet supported. Only **blog** and **slideshow** templates support multi-language sites.

### How It Works

1. **Language Detection** — Languages are automatically detected from subdirectories (e.g., `en/`, `de/`, `cs/`, `zh/`)
2. **Directory Structure** — Each language has its own subdirectory with markdown files
3. **Automatic Fallback** — If a page doesn't exist in a language, content falls back to the default language
4. **hreflang Tags** — SEO-friendly hreflang meta tags are automatically generated

### Directory Structure

```
content/
├── en/
│   ├── landing.md
│   ├── about.md
│   └── posts/
│       └── getting-started.md
├── de/
│   ├── landing.md
│   ├── about.md
│   └── posts/
│       └── getting-started.md
├── cs/                     # Czech - about.md missing, falls back to en/
│   ├── landing.md
│   └── posts/
│       └── rust-history.md
└── zh/                     # Chinese - posts/ missing entirely, falls back to en/
    ├── landing.md
    └── about.md
```

### Building i18n Sites

The build command automatically detects i18n structure:

```bash
cargo run -p rustpress -- build \
  --input examples/i18n/content/ \
  --output /tmp/i18n-site/ \
  --template blog
```

This generates:

```
output/
├── index.html              # Language selector page
├── en/
│   ├── landing.html
│   ├── about.html
│   └── posts/
├── de/
│   └── ...
├── cs/                     # Falls back to en/ for missing pages
│   └── ...
└── zh/                     # Falls back to en/ for missing pages
    └── ...
```

### Fallback Behavior

| Scenario | Behavior |
|----------|----------|
| Page exists in language | Uses that translation |
| Page missing in language | Falls back to default language content |
| Page missing in all languages | Page not generated |

### Language Selector Component

The blog template includes a Vue 3 `LanguageSelector` component that:
- Auto-hides when only one language exists
- Shows a dropdown with all available languages
- Works on **any page** - clicking a language switches to the **same page** in another language
- Uses native language names (e.g., "Deutsch" not "German")
- Automatically determines the current page path and generates language-switched URLs

For example, if you're on `/en/posts/getting-started.html`:
- Clicking "Deutsch" takes you to `/de/posts/getting-started.html`
- Clicking "Čeština" takes you to `/cs/posts/getting-started.html`

### Language Codes

Supported language codes with native names:

| Code | Native Name |
|------|-------------|
| en | English |
| de | Deutsch |
| cs | Čeština |
| zh | 中文 |
| fr | Français |
| es | Español |
| ja | 日本語 |
| ko | 한국어 |
| ru | Русский |
| ... | (and many more) |

## WebAssembly Usage

For browser usage, you can use the WASM build:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Rustpress WASM Demo</title>
</head>
<body>
    <div id="output"></div>

    <script type="module">
        import init, { Rustpress } from './rustpress_wasm.js';

        async function run() {
            await init();
            const rustpress = new Rustpress();

            const markdown = `
# Hello from Rustpress

This is rendered using WebAssembly!

<Alert type="info" title="WebAssembly">
  Rustpress is running directly in your browser via WASM.
</Alert>
            `;

            const html = rustpress.render_markdown(markdown);
            document.getElementById('output').innerHTML = html;
        }

        run();
    </script>
</body>
</html>
```

## Project Structure

```
rustpress/
├── rustpress-core/      # Core library (parsing, rendering, components)
│   └── src/
│       ├── lib.rs           # Exports, tests
│       ├── markdown.rs      # Markdown parsing
│       ├── render.rs        # Template rendering + Vue store serialization
│       ├── search.rs        # Full-text search index builder
│       ├── types.rs         # Core types + Vue Store types
│       ├── vue.rs           # Vue component support
│       ├── mdbook.rs        # mdbook project parsing
│       ├── i18n.rs          # Internationalization support
│       ├── components/      # Component registry + builtins
│       └── templates/       # Built-in HTML templates
│           ├── blog/        # Blog Vue SPA templates
│           ├── slideshow/   # Slideshow Vue SPA templates
│           ├── mdbook/      # mdbook Vue SPA template
│           └── shared/      # Shared components (search.js, language-selector.js)
├── rustpress/       # CLI application
├── rustpress-wasm/      # WASM bindings
└── examples/            # Example projects
    ├── blog/            # Blog example
    ├── slideshow/       # Slideshow example
    ├── mdbook/          # mdbook ingestion example
    ├── search/          # Search functionality example
    └── i18n/            # Multi-language site example
```

## Development

### Prerequisites

- **Rust nightly** (required for `edition = "2024"`)
- wasm-pack (for WASM development)

### Building

```bash
# Build everything
cargo build

# Build WASM package
cd rustpress-wasm
wasm-pack build --target web
cp ./demo/index.html ./dist/
cp ./pkg/* ./dist/
```
than just serve the `./dist/` folder

### Running Tests

```bash
cargo test
```

### Examples

Each template has a working example in `examples/`:

```bash
# Blog
cargo run -p rustpress -- build -i examples/blog/content -o /tmp/blog -t blog

# Blog with live reload
cargo run -p rustpress -- dev -i examples/blog/content -o /tmp/blog -t blog

# Slideshow
cargo run -p rustpress -- build -i examples/slideshow/content -o /tmp/slideshow -t slideshow

# mdbook
cargo run -p rustpress -- build -i examples/mdbook/content -o /tmp/mdbook -t mdbook

# Multi-language (i18n) - blog and slideshow templates
cargo run -p rustpress -- build -i examples/i18n/content -o /tmp/i18n -t blog
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
