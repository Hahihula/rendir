---
title: "Understanding Ownership in Rust"
date: 2024-03-01
author: "Jane Doe"
tags: ["rust", "ownership", "memory-safety"]
category: "deep-dives"
publish: true
---

# Understanding Ownership in Rust

Rust's ownership system is one of its most distinctive features. Understanding it is key to writing safe, efficient Rust code.

## The Three Rules

1. Every value in Rust has a single owner
2. There can only be one owner at a time
3. When the owner goes out of scope, the value is dropped

## Move Semantics

```rust
let s1 = String::from("hello");
let s2 = s1; // s1 is moved to s2

// println!("{}", s1); // This would fail to compile!
println!("{}", s2); // This works fine
```

## Borrowing

Instead of transferring ownership, you can *borrow* values:

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}

let s1 = String::from("hello");
let len = calculate_length(&s1); // borrow, don't move
println!("The length of '{}' is {}.", s1, len); // s1 is still valid!
```

## Why This Matters

The ownership model means Rust can guarantee memory safety without a garbage collector. No dangling pointers, no double-frees, no null pointer exceptions!

Learn more about [ownership and borrowing](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) in the official book.