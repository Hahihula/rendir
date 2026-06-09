# Slideshow Example

This example demonstrates how to create presentations with Rendir.

## Overview

Rendir supports slideshow-style presentations with:
- **Vue.js SPA** - JavaScript renders slides with reactive data binding
- **Markdown per slide** — each slide authored in Markdown, converted to HTML
- **Keyboard navigation** — arrow keys, space, Home/End
- **Mouse/touch support** — click navigation, swipe gestures
- **Presenter view** — separate window with notes, timer, next slide preview
- **BroadcastChannel sync** — real-time sync between presenter and audience

## Architecture

```
Markdown → Rust: parse components → Vue Store JSON → Vue SPA renders UI
```

Rust processes Markdown and outputs a **Vue Store** (JSON data structure) embedded in the HTML. Vue.js reads this data and handles all rendering, navigation, and interactivity.

## Vue Store Data Structure

Rust provides this JSON to the Vue SPA via `<script id="store-data">`:

```javascript
{
  title: "Presentation Title",
  slides: [
    {
      id: 0,
      type: "full",           // slide layout type
      title: "Slide Title",
      content: "<p>HTML content</p>",
      notes: "Speaker notes",
      images: ["image1.png"]
    },
    // ... more slides
  ],
  current_slide_index: 0,
  images: {
    "image1.png": "/path/to/image1.png"
  }
}
```

### Slide Layout Types

| Type | Description |
|------|-------------|
| `full` | Centered content, max-width |
| `split-left` | Content left, image/list right |
| `split-right` | Image/list left, content right |
| `center` | Centered title and content |
| `list` | Bulleted or numbered content |
| `image-bg` | Full-width image background with overlay |

## Slide Syntax

### Primary: `<Slide>` Component

```markdown
:::slide{title="Welcome to Rendir"}
# Hello!

This is the first slide. You can use **Markdown** here.

- Item 1
- Item 2
:::

:::slide{title="Features" type="split-left"}
## What can you do?

- Build static sites
- Create presentations
- Generate WASM modules
:::
```

### Fallback: Heading Level 2

If no `:::slide` component is found, **heading level 2** (`## Slide Title`) acts as a slide delimiter:

```markdown
## Slide One

Content for slide one...

## Slide Two

Content for slide two...
```

When using heading-level fallback, the heading text becomes the slide title and layout defaults to `full`.

## Slide Component Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `title` | No | Slide title displayed in navigation and presenter |
| `type` | No | Slide layout type (default: `full`) |
| `notes` | No | Speaker notes (visible in presenter view) |

## Presentation YAML Frontmatter

```yaml
---
title: "My Presentation"
author: "Your Name"
date: "2026-05-17"
---
```

| Frontmatter Key | Description |
|-----------------|-------------|
| `title` | Presentation title |
| `author` | Presenter name |
| `date` | Date of presentation |

## Navigation

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `→` or `Space` | Next slide |
| `←` | Previous slide |
| `Home` | First slide |
| `End` | Last slide |

### Mouse/Touch

- Click navigation dots to jump to a slide
- Swipe left/right on touch devices

### Presenter View

Click the **"Present"** link (top-right) to open presenter view. Presenter view receives the same store data and syncs with the main view via BroadcastChannel.

## CLI Usage

```bash
# Build a slideshow
cargo run -p rendir -- build \
  --input examples/slideshow/content/ \
  --output site/ \
  --template slideshow

# Single file conversion
cargo run -p rendir -- convert \
  --input examples/slideshow/content/presentation.md \
  --output /tmp/presentation.html \
  --template slideshow
```

### Image Handling

Images referenced in Markdown (`![alt](path)`) are automatically copied to the output directory.

## File Structure

```
examples/slideshow/
├── README.md              # This file
└── content/
    └── presentation.md     # Sample presentation source
```

## Customizing Templates

### Creating Custom Slideshow Templates

Copy `rendir-core/src/templates/slideshow/` to your own folder and customize:

```
my-templates/slideshow/
├── index.html      # Main slideshow Vue SPA
└── presenter.html  # Presenter view Vue SPA
```

### Required Data Embed

Your template must include this to receive data from Rust:

```html
<script id="store-data" type="application/json">{{STORE_DATA}}</script>
```

Vue will parse this JSON and use it as reactive store data.

### Vue Store Access

```javascript
const store = JSON.parse(
  document.getElementById('store-data').textContent
);

// Access slide data
store.slides[store.currentSlideIndex].content
```

## WASM Compatibility

The slideshow works in WASM. When served from a WASM module:
- All navigation works client-side
- BroadcastChannel sync works in modern browsers
- Timer and presenter notes work without server
