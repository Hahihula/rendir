---
title: Welcome to Rendir
author: Rendir Team
date: 2026-05-17
tags: [rust, static-site-generator, markdown, wasm]
---

# Rendir

Rendir is a flexible static site generator written in Rust, designed to convert Markdown content into static websites. It offers both CLI and WebAssembly (WASM) interfaces, with support for custom components, templates, and more.

<YouTube id="5C_HPTJg5ek" />

## Features

<Alert type="info" title="Try it out">
This example page demonstrates all currently supported features in Rendir. You can convert this file using `rendir convert --input example.md --output example.html` and open it in your browser to see the rendered output.
</Alert>

## Component Syntax

Rendir supports **two syntaxes** for components.

### HTML-like Syntax

```markdown
<Alert type="warning" title="Warning">
This is a warning alert.
</Alert>
```

### Special Markdown Syntax

```markdown
:::alert{type="error" title="Error"}
This is an error alert.
:::
```

## Built-in Components

### Alerts

<Alert type="info" title="Information">
This is an informational alert. Use this for general notes or tips.
</Alert>

<Alert type="warning" title="Warning">
This is a warning alert. Use this to highlight important information.
</Alert>

<Alert type="error" title="Error">
This is an error alert. Use this to indicate critical issues.
</Alert>

### YouTube Embeds

:::alert{type="info" title="YouTube Component"}
The YouTube component embeds videos using an iframe. Above this section, you should see the video at https://www.youtube.com/watch?v=5C_HPTJg5ek.
:::

<YouTube id="5C_HPTJg5ek" />

### Tabs

<Tabs>
## Installation
Clone the repository and build with Cargo:

```bash
git clone https://gitlab.com/hahihula/rendir.git
cd rendir
cargo build --release
```

## Usage
Convert a single Markdown file:

```bash
rendir convert --input example.md --output example.html
```

Build an entire directory:

```bash
rendir build --input content/ --output site/
```

## WebAssembly
Rendir also ships as a WASM module for browser usage. See the project README for details.
</Tabs>

## Frontmatter

Metadata is extracted from YAML frontmatter at the top of each file. The frontmatter for this page includes:

- **title**: "Welcome to Rendir"
- **author**: "Rendir Team"
- **date**: "2026-05-17"
- **tags**: rust, static-site-generator, markdown, wasm

## Templates

Rendir uses the [Tera](https://tera.netlify.app/) templating engine. You can pass custom templates via the `--template` flag:

```bash
rendir convert --input example.md --output example.html --template my-template.html
```

Templates receive a `{{ content | safe }}` variable for the rendered HTML content and all frontmatter keys as variables.

## Markdown Extensions

Rendir supports extended Markdown syntax via `pulldown-cmark`:

- Tables
- Footnotes
- Strikethrough
- Task lists
- Extended emphasis

| Feature | Supported |
|---------|-----------|
| Tables | Yes |
| Footnotes | Yes |
| Strikethrough | Yes |
| Task lists | Yes |

- [x] Write documentation
- [x] Create example page
- [ ] Add more components

---

*Built with Rust and ❤️ by the Rendir team*
