use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_convert_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.md");
    let output_path = temp_dir.path().join("output.html");

    fs::write(
        &input_path,
        r#"---
title: "Test Post"
---

# Hello World

This is a test.
"#,
    )
    .unwrap();

    Command::cargo_bin("rustpress-cli")
        .unwrap()
        .arg("convert")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Hello World"));
    assert!(content.contains("Test Post"));
}

#[test]
fn test_convert_with_template() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.md");
    let output_path = temp_dir.path().join("output.html");

    fs::write(
        &input_path,
        r#"---
title: "Test Post"
---

# Hello World
"#,
    )
    .unwrap();

    Command::cargo_bin("rustpress-cli")
        .unwrap()
        .arg("convert")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--template")
        .arg("slideshow")
        .assert()
        .success();

    assert!(output_path.exists());
}

#[test]
fn test_convert_nonexistent_input() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.html");

    Command::cargo_bin("rustpress-cli")
        .unwrap()
        .arg("convert")
        .arg("--input")
        .arg("/nonexistent/input.md")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .failure();
}

#[test]
fn test_build_blog_directory() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&content_dir).unwrap();
    fs::create_dir(&output_dir).unwrap();

    fs::write(
        content_dir.join("index.md"),
        r#"---
title: "Test Blog"
template: blog-landing
---

# Welcome
"#,
    )
    .unwrap();

    Command::cargo_bin("rustpress-cli")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg(&content_dir)
        .arg("--output")
        .arg(&output_dir)
        .arg("--template")
        .arg("blog")
        .assert()
        .success();

    assert!(output_dir.join("index.html").exists());
}

#[test]
fn test_build_nonexistent_directory() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");

    Command::cargo_bin("rustpress-cli")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg("/nonexistent/content")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .failure();
}

#[test]
fn test_help_flag() {
    Command::cargo_bin("rustpress-cli")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_convert_help() {
    Command::cargo_bin("rustpress-cli")
        .unwrap()
        .arg("convert")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_build_help() {
    Command::cargo_bin("rustpress-cli")
        .unwrap()
        .arg("build")
        .arg("--help")
        .assert()
        .success();
}
