# Blog Example

This example demonstrates rustpress's blog template functionality.

## Structure

```
content/
├── landing.md                    # Blog landing page (homepage)
└── posts/
    ├── 2024-01-15-welcome.md     # First post
    ├── 2024-02-01-getting-started-rust.md
    ├── 2024-02-15-building-web-apis.md
    └── 2024-03-01-understanding-ownership.md
```

## Architecture

```
Markdown → Rust: parse → Vue Store JSON → Vue SPA renders UI
```

Rust processes Markdown and outputs a **Vue Store** (JSON data structure) embedded in the HTML. Vue.js handles all rendering, filtering, and interactivity.

## Vue Store Data Structure

Rust provides this JSON to the Vue SPA via `<script id="store-data">`:

### BlogIndexStore (Landing Page)

```javascript
{
  title: "Blog Title",
  description: "A blog about Rust",
  content: "",
  posts: [
    {
      id: "posts/2024-01-15-welcome",
      title: "Welcome to My Blog",
      date: "2024-01-15",
      author: "Jane Doe",
      excerpt: "",
      tags: ["intro", "welcome"],
      url: "posts/2024-01-15-welcome.html"
    }
    // ... more posts
  ],
  tags: [
    { name: "rust", count: 3 },
    { name: "programming", count: 5 }
  ],
  recent_posts: [
    // same structure as posts, limited to 5
  ]
}
```

## Frontmatter Fields

All blog posts support these frontmatter fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Post title |
| `date` | date | Yes | Publication date (YYYY-MM-DD) |
| `author` | string | No | Author name (default: "Unknown Author") |
| `tags` | string | No | Comma-separated tags or YAML array |
| `category` | string | No | Category for grouping |
| `publish` | boolean | No | Must be `true` to appear (default: false) |
| `draft` | boolean | No | Set `true` to exclude from build |

## Tags Syntax

Tags can be YAML array or comma-separated string:

```yaml
---
title: "My Post"
tags: ["rust", "web", "tutorial"]
---
```

```yaml
---
title: "My Post"
tags: rust, web, tutorial
---
```

## Using the Blog Template

### Build entire blog:

```bash
cargo run -p rustpress-cli -- build \
  --input examples/blog/content/ \
  --output /tmp/blog/ \
  --template blog
```

### Convert single post:

```bash
cargo run -p rustpress-cli -- convert \
  --input content/posts/2024-01-15-welcome.md \
  --output output/post.html \
  --template blog
```

## Vue SPA Features

The landing page is a Vue.js SPA with:

- **Search** - Filter posts by title
- **Tag filtering** - Click tags to filter posts
- **Recent posts** - Sidebar listing of latest posts

## Customizing Templates

### Creating Custom Blog Templates

Copy `rustpress-core/src/templates/blog/` to your own folder:

```
my-templates/blog/
├── index.html      # Vue SPA for landing page
└── post.html       # Vue SPA for individual posts
```

### Required Data Embed

Your template must include this to receive data from Rust:

```html
<script id="store-data" type="application/json">{{STORE_DATA}}</script>
```

### Vue Store Access

```javascript
const store = JSON.parse(
  document.getElementById('store-data').textContent
);

// Access posts
store.posts.forEach(post => {
  console.log(post.title, post.url);
});
```

## WASM Compatibility

The blog works in WASM. When served from a WASM module:
- All filtering and navigation works client-side
- No server required