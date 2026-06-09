---
title: "Welcome to Rendir"
author: "Rendir Team"
date: "2026-05-17"
slides:
  transition: "slide"
---

# Welcome to Rendir

A modern static site generator and presentation tool built with Rust.

![Rendir Logo](rendir_logo.png)

:::slide{title="What is Rendir?"}
## What is Rendir?

Rendir is a flexible content transformation tool that supports:

- **Static site generation** — blogs, documentation, cookbooks
- **Presentations** — reveal.js-style slideshows
- **WASM modules** — run anywhere in the browser

Built with Rust for speed and reliability.
:::

:::slide{title="Key Features"}
## Key Features

- Markdown with YAML frontmatter
- Custom components (`Alert`, `Tabs`, `YouTube`, `Slide`, ...)
- Tera templating engine
- WASM output for browser deployment
- Pre-built search index (pure Rust)

:::slide{title="Components Demo"}
## Components

rendir supports reusable components:

:::alert{type="info" title="Info"}
This is an alert component!
:::

:::alert{type="warning" title="Warning"}
Be careful with this!
:::

Try the Tabs component:

<Tabs>
## Tab A
Content for Tab A...

## Tab B
Content for Tab B...
</Tabs>
:::

:::slide{title="Code Support"}
## Code Blocks

rendir supports syntax highlighting via Markdown:

```rust
fn main() {
    println!("Hello, Rendir!");
}
```

```javascript
console.log("Hello from JavaScript!");
```

```python
print("Hello from Python!")
```
:::

:::slide{title="Tables & Lists"}
## Tables and Task Lists

| Feature | Supported |
|---------|-----------|
| Tables | Yes |
| Footnotes | Yes |
| Strikethrough | Yes |
| Task lists | Yes |

Task list:

- [x] Write documentation
- [x] Create examples
- [ ] Add more features
:::

:::slide{title="Search"}
## Built-in Search

rendir includes a **pure Rust search engine**:

- Pre-built index at compile time
- Title boosting for relevant results
- Snippets with query highlighting
- Works in WASM — no server needed!

```rust
let results = search_index.search("rust tutorial", 10);
```
:::

:::slide{title="Get Started"}
## Get Started

```bash
# Clone the repository
git clone https://gitlab.com/hahihula/rendir.git
cd rendir

# Build
cargo build --release

# Create a presentation
cargo run -p rendir -- convert \\
  --input my-presentation.md \\
  --output presentation.html \\
  --template slideshow
```

Learn more at the project repository.
:::

:::slide{title="Questions?"}
## Questions?

Thank you for your attention!

**Project**: https://gitlab.com/hahihula/rendir

**License**: Apache 2.0 / MIT
:::