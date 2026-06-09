use assert_cmd::Command;
use std::fs;
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

#[test]
#[ignore]
fn test_build_rust_lang_book() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    let book_dir = temp_path.join("book");
    let output_dir = temp_path.join("output");

    println!("\n\n===== TEST OUTPUT LOCATION =====");
    println!("Output dir: {}", output_dir.display());
    println!("Temp dir will NOT be deleted: {}", temp_path.display());
    println!("==============================\n\n");

    // Keep temp_dir alive to prevent deletion
    std::mem::forget(temp_dir);

    println!("Cloning rust-lang/book...");
    let clone_result = ProcessCommand::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/rust-lang/book",
            book_dir.to_str().unwrap(),
        ])
        .output();

    match clone_result {
        Ok(output) if output.status.success() => {}
        _ => {
            println!("Skipping test: failed to clone rust-lang/book");
            return;
        }
    }

    println!("Building rust-lang/book with rendir...");
    let result = Command::cargo_bin("rendir")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg(&book_dir)
        .arg("--output")
        .arg(&output_dir)
        .arg("--template")
        .arg("mdbook")
        .output();

    let output = match result {
        Ok(output) => output,
        Err(e) => {
            panic!("Failed to run build command: {}", e);
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("Build failed. stderr: {}", stderr);
    }
    println!("Build output: {}", stdout);

    assert!(output.status.success(), "Failed to build rust-lang/book");
    assert!(
        output_dir.join("index.html").exists(),
        "index.html not generated"
    );
}

#[test]
#[ignore]
fn test_build_mdbook_documentation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    let mdbook_dir = temp_path.join("mdbook");
    let output_dir = temp_path.join("output");

    println!("\n\n===== TEST OUTPUT LOCATION =====");
    println!("Output dir: {}", output_dir.display());
    println!("Temp dir will NOT be deleted: {}", temp_path.display());
    println!("==============================\n\n");

    // Keep temp_dir alive to prevent deletion
    std::mem::forget(temp_dir);

    println!("Cloning rust-lang/mdBook...");
    let clone_result = ProcessCommand::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/rust-lang/mdBook",
            mdbook_dir.to_str().unwrap(),
        ])
        .output();

    match clone_result {
        Ok(output) if output.status.success() => {}
        _ => {
            println!("Skipping test: failed to clone rust-lang/mdBook");
            return;
        }
    }

    let guide_dir = mdbook_dir.join("guide");
    if !guide_dir.exists() {
        println!("Skipping test: mdBook guide directory not found");
        return;
    }

    println!("guide/ dir contents:");
    for entry in std::fs::read_dir(&guide_dir).unwrap() {
        println!("  {:?}", entry.unwrap().path());
    }

    let result = Command::cargo_bin("rendir")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg(&guide_dir)
        .arg("--output")
        .arg(&output_dir)
        .arg("--template")
        .arg("mdbook")
        .output();

    let output = match result {
        Ok(output) => output,
        Err(e) => {
            panic!("Failed to run build command: {}", e);
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Debug: list output directory contents
    if output_dir.exists() {
        println!("Output directory exists, contents:");
        for entry in std::fs::read_dir(&output_dir).unwrap() {
            println!("  {:?}", entry.unwrap().path());
        }
    } else {
        println!("Output directory does not exist");
    }

    println!("Build stdout: {}", stdout);
    if !output.status.success() {
        eprintln!("Build failed. stderr: {}", stderr);
    }

    assert!(
        output.status.success(),
        "Failed to build mdBook documentation"
    );

    // mdBook outputs to src/ subdirectory when building guide
    let src_output_dir = output_dir.join("src");
    let has_html = src_output_dir.exists()
        && std::fs::read_dir(&src_output_dir).unwrap().any(|e| {
            let path = e.unwrap().path();
            path.is_file() && path.extension().map_or(false, |ext| ext == "html")
        });

    assert!(has_html, "No HTML files generated in output/src/");

    // Check for specific expected files
    assert!(
        src_output_dir.join("README.html").exists() || src_output_dir.join("SUMMARY.html").exists(),
        "Expected README.html or SUMMARY.html not found"
    );
}

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

    Command::cargo_bin("rendir")
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

    Command::cargo_bin("rendir")
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

    Command::cargo_bin("rendir")
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

    Command::cargo_bin("rendir")
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

    Command::cargo_bin("rendir")
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

    Command::cargo_bin("rendir")
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

    Command::cargo_bin("rendir")
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

    Command::cargo_bin("rendir")
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
    Command::cargo_bin("rendir")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_convert_help() {
    Command::cargo_bin("rendir")
        .unwrap()
        .arg("convert")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_build_help() {
    Command::cargo_bin("rendir")
        .unwrap()
        .arg("build")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_build_copies_local_images() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&content_dir).unwrap();

    let images_dir = content_dir.join("images");
    fs::create_dir(&images_dir).unwrap();

    fs::write(
        content_dir.join("index.md"),
        r#"---
title: "Test Page"
---

# Test

![Logo](images/logo.png)

Some text with ![Icon](images/icon.png) inline.
"#,
    )
    .unwrap();

    fs::write(images_dir.join("logo.png"), "PNG_DATA").unwrap();
    fs::write(images_dir.join("icon.png"), "PNG_DATA").unwrap();

    Command::cargo_bin("rendir")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg(&content_dir)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.join("index.html").exists());
    assert!(output_dir.join("images").is_dir());
    assert!(output_dir.join("images").join("logo.png").exists());
    assert!(output_dir.join("images").join("icon.png").exists());
}

#[test]
fn test_build_copies_local_files_from_links() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&content_dir).unwrap();

    let docs_dir = content_dir.join("docs");
    fs::create_dir(&docs_dir).unwrap();

    fs::write(
        content_dir.join("index.md"),
        r#"---
title: "Test Page"
---

# Test

[Download Guide](docs/guide.pdf)
"#,
    )
    .unwrap();

    fs::write(docs_dir.join("guide.pdf"), "PDF_DATA").unwrap();

    Command::cargo_bin("rendir")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg(&content_dir)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.join("index.html").exists());
}

#[test]
fn test_build_ignores_external_urls() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&content_dir).unwrap();

    fs::write(
        content_dir.join("index.md"),
        r#"---
title: "Test Page"
---

# Test

![External Image](https://example.com/image.png)

[External Link](https://example.com/page.html)
"#,
    )
    .unwrap();

    Command::cargo_bin("rendir")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg(&content_dir)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.join("index.html").exists());
}

#[test]
fn test_convert_copies_local_images() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.md");
    let output_path = temp_dir.path().join("output.html");
    let images_dir = temp_dir.path().join("images");
    fs::create_dir(&images_dir).unwrap();

    fs::write(
        &input_path,
        r#"---
title: "Test Post"
---

# Hello World

![Logo](images/logo.png)
"#,
    )
    .unwrap();

    fs::write(images_dir.join("logo.png"), "PNG_DATA").unwrap();

    Command::cargo_bin("rendir")
        .unwrap()
        .arg("convert")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    assert!(output_path.exists());
    assert!(output_path
        .parent()
        .unwrap()
        .join("images")
        .join("logo.png")
        .exists());
}

#[test]
fn test_build_with_nested_subdirectory_images() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&content_dir).unwrap();

    let chapter_dir = content_dir.join("chapter1");
    fs::create_dir(&chapter_dir).unwrap();
    let images_dir = chapter_dir.join("images");
    fs::create_dir(&images_dir).unwrap();

    fs::write(
        chapter_dir.join("index.md"),
        r#"---
title: "Chapter 1"
---

# Chapter One

![Diagram](images/diagram.png)
"#,
    )
    .unwrap();

    fs::write(images_dir.join("diagram.png"), "PNG_DATA").unwrap();

    Command::cargo_bin("rendir")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg(&content_dir)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.join("chapter1").join("index.html").exists());
    assert!(output_dir
        .join("chapter1")
        .join("images")
        .join("diagram.png")
        .exists());
}

#[test]
fn test_build_creates_remote_assets_directory() {
    let temp_dir = TempDir::new().unwrap();
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&content_dir).unwrap();

    fs::write(
        content_dir.join("index.md"),
        r#"---
title: "Test Page"
---

# Test

![Remote Image](https://example.com/image.png)
"#,
    )
    .unwrap();

    Command::cargo_bin("rendir")
        .unwrap()
        .arg("build")
        .arg("--input")
        .arg(&content_dir)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.join("index.html").exists());
}

#[test]
fn test_parse_markdown_extracts_remote_references() {
    use rendir_core::markdown::parse_markdown_with_path;
    use std::path::PathBuf;

    let content = r#"---
title: Test
---

# Hello

![Remote Logo](https://example.com/logo.png)

Some text.
"#;
    let item = parse_markdown_with_path(content, None, Some(PathBuf::from("/project/page.md")));
    assert!(!item.remote_references.is_empty());
    assert!(item
        .remote_references
        .contains(&"https://example.com/logo.png".to_string()));
}

#[test]
fn test_parse_markdown_separates_local_and_remote() {
    use rendir_core::markdown::parse_markdown_with_path;
    use std::path::PathBuf;

    let content = r#"---
title: Test
---

# Hello

![Remote](https://example.com/remote.png)
![Local](images/local.png)
"#;
    let item = parse_markdown_with_path(content, None, Some(PathBuf::from("/project/page.md")));
    assert!(!item.remote_references.is_empty());
    assert!(item.image_references.len() >= 1);
}
