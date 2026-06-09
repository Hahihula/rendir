# Rendir i18n Example

This example demonstrates Rendir internationalization (i18n) capabilities.

## Directory Structure

```
content/
├── en/          # English (default/fallback language)
│   ├── landing.md
│   ├── about.md
│   └── posts/
│       └── getting-started.md
├── de/          # German (fully translated)
│   ├── landing.md
│   ├── about.md
│   └── posts/
│       └── getting-started.md
├── cs/          # Czech (partial - no about.md, different posts)
│   ├── landing.md
│   └── posts/
│       └── rust-history.md  (different post, not translated from English)
└── zh/          # Chinese (minimal - no posts directory)
    ├── landing.md
    └── about.md
```

## Fallback Scenarios Tested

1. **cs/about.md missing** → falls back to `en/about.md`
2. **zh/posts/ entirely missing** → all posts fall back to English
3. **cs/posts/rust-history.md** exists only in Czech (no translation in other languages)

## Language Detection

Languages are automatically detected from top-level directories in the content folder.

## Running the Example

```bash
# Build with i18n support
cargo run -p rendir -- build \
  --input examples/i18n/content/ \
  --output /tmp/i18n/ \
  --template blog

# Development server with live reload
cargo run -p rendir -- dev \
  --input examples/i18n/content/ \
  --output /tmp/i18n/ \
  --template blog
```

## Expected Output Structure

```
/tmp/i18n/
├── en/
│   ├── landing.html
│   ├── about.html
│   └── posts/
│       └── getting-started.html
├── de/
│   ├── landing.html
│   ├── about.html
│   └── posts/
│       └── getting-started.html
├── cs/
│   ├── landing.html       (own content)
│   ├── about.html         (fallback from en/)
│   └── posts/
│       └── rust-history.html (own content, unique to Czech)
└── zh/
    ├── landing.html       (own content)
    ├── about.html         (own content)
    └── posts/
        └── getting-started.html  (fallback from en/)
```

## Testing Fallback Behavior

1. Build the example
2. Check `cs/about.html` - should show English content (fallback)
3. Check `zh/posts/getting-started.html` - should show English content (fallback)
4. Check `cs/posts/rust-history.html` - should show Czech content (only exists in Czech)
5. Check `de/posts/getting-started.html` - should show German content (full translation)
