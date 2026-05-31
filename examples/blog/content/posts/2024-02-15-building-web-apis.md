---
title: "Building Web APIs with Axum"
date: 2024-02-15
author: "Jane Doe"
tags: ["rust", "web", "api", "axum"]
category: "tutorials"
publish: true
---

# Building Web APIs with Axum

Axum is a popular Rust web framework that's fast, ergonomic, and type-safe. In this post, we'll build a REST API from scratch.

## Project Setup

```bash
cargo new my-api
cd my-api
cargo add axum tokio serde
```

## Creating Your First Route

```rust
use axum::{routing::get, Router};

async fn hello() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Key Features

- **Type-safe routing** with compile-time guarantees
- **Middleware support** for logging, auth, etc.
- **Easy JSON handling** with Serde
- **WebSocket support** built-in

Axum makes it easy to build production-ready APIs in Rust!