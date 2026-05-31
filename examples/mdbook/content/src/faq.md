# FAQ

Frequently asked questions about mdbook and rustpress ingestion.

## Q: Can I use custom templates?

A: Yes! Rustpress lets you provide custom templates via `--template`.

## Q: How does navigation work?

A: Rustpress parses `SUMMARY.md` to build the navigation tree.

## Q: Is mdbook required for ingestion?

A: Yes, currently rustpress expects a `book.toml` and `SUMMARY.md` to properly structure the book.