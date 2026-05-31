use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_i18n_build_with_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");

    // Create en/ and cs/ directories
    let en_dir = content_dir.join("en");
    let cs_dir = content_dir.join("cs");
    fs::create_dir_all(&en_dir).unwrap();
    fs::create_dir_all(&cs_dir).unwrap();

    // English has landing.md and about.md
    fs::write(
        en_dir.join("landing.md"),
        r#"---
title: "Welcome"
---
# Welcome to the Site
"#,
    )
    .unwrap();
    fs::write(
        en_dir.join("about.md"),
        r#"---
title: "About Us"
---
# About Us
"#,
    )
    .unwrap();

    // Czech only has landing.md (about.md is missing - should fallback)
    fs::write(
        cs_dir.join("landing.md"),
        r#"---
title: "Vítejte"
---
# Vítejte na stránkách
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

    // Check that root index.html exists (language selector page)
    assert!(output_dir.join("index.html").exists());
    let root_content = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(root_content.contains("Select your language"));

    // Check en/ directory exists
    assert!(output_dir.join("en").is_dir());
    assert!(output_dir.join("en").join("landing.html").exists());
    assert!(output_dir.join("en").join("about.html").exists());

    // Check cs/ directory exists
    assert!(output_dir.join("cs").is_dir());
    assert!(output_dir.join("cs").join("landing.html").exists());

    // cs/about.html should exist (fallback from English)
    assert!(output_dir.join("cs").join("about.html").exists());
    let cs_about = fs::read_to_string(output_dir.join("cs").join("about.html")).unwrap();
    // Should contain fallback content (English "About Us")
    assert!(cs_about.contains("About Us"));
}

#[test]
fn test_i18n_build_detects_all_languages() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");

    // Create en/, de/, fr/ directories
    for lang in &["en", "de", "fr"] {
        let lang_dir = content_dir.join(lang);
        fs::create_dir_all(&lang_dir).unwrap();
        fs::write(
            lang_dir.join("index.md"),
            &format!("---\ntitle: \"Home - {}\"\n---\n# {}", lang, lang),
        )
        .unwrap();
    }

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

    // All language directories should exist
    assert!(output_dir.join("en").is_dir());
    assert!(output_dir.join("de").is_dir());
    assert!(output_dir.join("fr").is_dir());

    // Check root index has links to all languages
    let root_content = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(root_content.contains("/en/"));
    assert!(root_content.contains("/de/"));
    assert!(root_content.contains("/fr/"));
}

#[test]
fn test_i18n_blog_landing_page() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");

    // Create en/ and de/ with landing pages
    let en_dir = content_dir.join("en");
    let de_dir = content_dir.join("de");
    fs::create_dir_all(&en_dir).unwrap();
    fs::create_dir_all(&de_dir).unwrap();

    fs::write(
        en_dir.join("landing.md"),
        r#"---
title: "My Blog"
---
# Welcome
"#,
    )
    .unwrap();
    fs::write(
        de_dir.join("landing.md"),
        r#"---
title: "Mein Blog"
---
# German Content
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

    // Should create en/landing.html
    assert!(output_dir.join("en").join("landing.html").exists());
    assert!(output_dir.join("de").join("landing.html").exists());

    // Check that hreflang tags are present
    let en_landing = fs::read_to_string(output_dir.join("en").join("landing.html")).unwrap();
    assert!(en_landing.contains("hreflang"));

    // Check German landing has German title
    let de_landing = fs::read_to_string(output_dir.join("de").join("landing.html")).unwrap();
    // Title is rendered via Vue, so check for the title tag content
    assert!(de_landing.contains("<title>Mein Blog</title>") || de_landing.contains("Mein Blog"));
}

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
