# Deployment

Deploy your mdbook to various platforms:

## GitHub Pages

```bash
mdbook build
```

Push the `book/` folder to gh-pages branch.

## Netlify

Configure `netlify.toml`:

```toml
[build]
command = "mdbook build"
publish = "book"
```