use anyhow::Result;
use clap::{Parser, Subcommand};
use notify::{RecursiveMode, Watcher};
use rustpress_core::components::{builtins::register_builtin_components, ComponentRegistry};
use rustpress_core::{
    get_builtin_template,
    mdbook::{BookToml, Chapter, Summary},
    parse_markdown_with_path, render_blog_index_vue, render_html, render_mdbook_vue,
    render_slideshow_vue, render_with_template,
    rss::{parse_date_to_rfc2822, strip_html, RssFeed, RssItem},
    search::SearchDocument,
    types::{BlogIndexStore, BlogPostSummary, ChapterNav, ChapterStore, ContentItem, TagCount},
};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;
use tiny_http::{Header, Response, Server};
use walkdir::WalkDir;

type ParsedMdItem = (
    PathBuf,
    PathBuf,
    String,
    String,
    std::collections::HashMap<String, String>,
);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a markdown file to HTML
    Convert {
        /// Input markdown file
        #[arg(short, long)]
        input: PathBuf,

        /// Output HTML file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Custom HTML template file or builtin template name (e.g., slideshow)
        #[arg(short, long)]
        template: Option<PathBuf>,
    },
    /// Build a site from a directory of markdown files
    Build {
        /// Input directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Custom HTML template file or builtin template name (e.g., slideshow)
        #[arg(short, long)]
        template: Option<PathBuf>,
    },
    /// Watch the input/template for changes, rebuild on change, and serve the result locally
    Dev {
        /// Input directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Custom HTML template file or builtin template name (e.g., slideshow)
        #[arg(short, long)]
        template: Option<PathBuf>,

        /// Port for the local dev server
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

fn render_nav_tree(chapters: &[Chapter], _base_path: &Path) -> String {
    let mut html = String::new();
    for chapter in chapters {
        let path_str = chapter.path.to_string_lossy().replace(".md", "");
        let href = if let Some(stripped) = path_str.strip_prefix("src/") {
            format!("{}.html", stripped)
        } else {
            format!("{}.html", path_str)
        };
        html.push_str(&format!("<li><a href=\"{}\">{}</a>", href, chapter.title));
        if !chapter.children.is_empty() {
            html.push_str("<ul class=\"section\">");
            html.push_str(&render_nav_tree(&chapter.children, _base_path));
            html.push_str("</ul>");
        }
        html.push_str("</li>\n");
    }
    html
}

fn get_all_chapters(chapters: &[Chapter]) -> Vec<&Chapter> {
    let mut result = Vec::new();
    fn collect<'a>(chapters: &'a [Chapter], result: &mut Vec<&'a Chapter>) {
        for ch in chapters {
            result.push(ch);
            collect(&ch.children, result);
        }
    }
    collect(chapters, &mut result);
    result
}

/// Build ChapterStore from Chapter with prev/next navigation
#[allow(clippy::type_complexity)]
fn build_chapter_store(
    chapter: &Chapter,
    all_chapters: &[&Chapter],
    current_idx: usize,
    content: String,
    all_items: &[ParsedMdItem],
) -> ChapterStore {
    let url = chapter
        .path
        .to_string_lossy()
        .replace(".md", "")
        .replace("src/", "")
        + ".html";

    let prev_chapter = if current_idx > 0 {
        let prev = all_chapters[current_idx - 1];
        Some(ChapterNav {
            title: prev.title.clone(),
            url: prev
                .path
                .to_string_lossy()
                .replace(".md", "")
                .replace("src/", "")
                + ".html",
        })
    } else {
        None
    };

    let next_chapter = if current_idx < all_chapters.len() - 1 {
        let next = all_chapters[current_idx + 1];
        Some(ChapterNav {
            title: next.title.clone(),
            url: next
                .path
                .to_string_lossy()
                .replace(".md", "")
                .replace("src/", "")
                + ".html",
        })
    } else {
        None
    };

    let children: Vec<ChapterStore> = chapter
        .children
        .iter()
        .map(|child| {
            let child_idx = all_chapters
                .iter()
                .position(|c| c.path == child.path)
                .unwrap_or(current_idx);
            // Look up the child's content from all_items
            let child_content = all_items
                .iter()
                .find(|(rp, _, _, _, _)| {
                    rp.to_string_lossy().replace(".md", "").replace("src/", "")
                        == child
                            .path
                            .to_string_lossy()
                            .replace(".md", "")
                            .replace("src/", "")
                })
                .map(|(_, _, _, rendered, _)| rendered.clone())
                .unwrap_or_default();
            build_chapter_store(child, all_chapters, child_idx, child_content, all_items)
        })
        .collect();

    ChapterStore {
        title: chapter.title.clone(),
        url,
        content,
        level: chapter.level,
        children,
        prev_chapter,
        next_chapter,
    }
}

/// Build all chapters as ChapterStore for Vue SPA
#[allow(clippy::type_complexity)]
fn build_all_chapter_stores(chapters: &[Chapter], all_items: &[ParsedMdItem]) -> Vec<ChapterStore> {
    let all = get_all_chapters(chapters);
    let all_refs: Vec<&Chapter> = all.to_vec();

    chapters
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            let idx = all.iter().position(|c| c.path == ch.path).unwrap_or(i);
            let content = all_items
                .iter()
                .find(|(rp, _, _, _, _)| {
                    rp.to_string_lossy().replace(".md", "").replace("src/", "")
                        == ch
                            .path
                            .to_string_lossy()
                            .replace(".md", "")
                            .replace("src/", "")
                })
                .map(|(_, _, _, rendered, _)| rendered.clone())
                .unwrap_or_default();
            build_chapter_store(ch, &all_refs, idx, content, all_items)
        })
        .collect()
}

/// Flatten all chapters (including nested children) for searching
fn flatten_chapter_stores(chapters: &[ChapterStore]) -> Vec<ChapterStore> {
    let mut result = Vec::new();
    for ch in chapters {
        result.push(ch.clone());
        if !ch.children.is_empty() {
            result.extend(flatten_chapter_stores(&ch.children));
        }
    }
    result
}

fn render_prev_next(chapters: &[Chapter], current: &Path, _base_path: &Path) -> (String, String) {
    let all = get_all_chapters(chapters);
    let current_str = current
        .to_string_lossy()
        .replace(".md", "")
        .replace("src/", "");

    let (current_idx, _) = all
        .iter()
        .enumerate()
        .find(|(_, ch)| {
            ch.path
                .to_string_lossy()
                .replace(".md", "")
                .replace("src/", "")
                == current_str
        })
        .unwrap_or((0, &all[0]));

    let prev = if current_idx > 0 {
        let prev_ch = all[current_idx - 1];
        let prev_path = prev_ch
            .path
            .to_string_lossy()
            .replace(".md", "")
            .replace("src/", "");
        format!("<a href=\"{}.html\">← {}</a>", prev_path, prev_ch.title)
    } else {
        String::new()
    };

    let next = if current_idx < all.len() - 1 {
        let next_ch = all[current_idx + 1];
        let next_path = next_ch
            .path
            .to_string_lossy()
            .replace(".md", "")
            .replace("src/", "");
        format!("<a href=\"{}.html\">{} →</a>", next_path, next_ch.title)
    } else {
        String::new()
    };

    (prev, next)
}

/// Create a simple index.html that lists all HTML files
#[allow(dead_code)]
fn create_index_page(dir: &Path) -> Result<()> {
    let mut index_html =
        String::from("<!DOCTYPE html>\n<html>\n<head>\n<title>Site Index</title>\n");
    index_html.push_str("<style>body { font-family: system-ui, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; }</style>\n");
    index_html.push_str("</head>\n<body>\n");
    index_html.push_str("<h1>Site Index</h1>\n<ul>\n");

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "html") {
            let rel_path = path.strip_prefix(dir)?;
            let path_str = rel_path.to_string_lossy();

            if rel_path != Path::new("index.html") {
                index_html.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>\n",
                    path_str, path_str
                ));
            }
        }
    }

    index_html.push_str("</ul>\n</body>\n</html>");

    let index_path = dir.join("index.html");
    let mut file = File::create(index_path)?;
    file.write_all(index_html.as_bytes())?;

    Ok(())
}

/// Build a site from a directory of markdown files.
///
/// This is the same logic the `build` command uses; the `dev` command calls it
/// directly on every detected change rather than reimplementing it.
fn run_build(input: &Path, output: &Path, template: &Option<PathBuf>) -> Result<()> {
    let mut registry = ComponentRegistry::new();
    register_builtin_components(&mut registry);

    fs::create_dir_all(output)?;

    let book_toml = input
        .join("book.toml")
        .exists()
        .then(|| BookToml::from_path(&input.join("book.toml")).unwrap_or_default());

    let summary = input
        .join("SUMMARY.md")
        .exists()
        .then(|| Summary::from_path(&input.join("SUMMARY.md")))
        .flatten();

    let src_dir = input.join(
        book_toml
            .as_ref()
            .map(|b| b.book.src.as_str())
            .unwrap_or("src"),
    );
    let src_dir = if src_dir.exists() {
        src_dir
    } else {
        input.to_path_buf()
    };

    let builtin_template_name: Option<String> = template.as_ref().and_then(|p| {
        let name = p.to_string_lossy();

        match name.as_ref() {
            "blog" => Some("blog/index".to_string()),
            t if get_builtin_template(t).is_some() => Some(t.to_string()),
            _ => None,
        }
    });

    let is_blog_template = template
        .as_ref()
        .map(|p| p.to_string_lossy() == "blog")
        .unwrap_or(false);

    let custom_template_content = if let Some(template_path) = template {
        if builtin_template_name.is_some() || is_blog_template {
            None
        } else if template_path.exists() {
            Some(fs::read_to_string(template_path)?)
        } else {
            return Err(anyhow::anyhow!(
                "Template not found: {}",
                template_path.display()
            ));
        }
    } else {
        None
    };

    let nav_tree = summary
        .as_ref()
        .map(|s| render_nav_tree(&s.chapters, &src_dir))
        .unwrap_or_default();

    let book_title = book_toml
        .as_ref()
        .and_then(|b| b.book.title.clone())
        .unwrap_or_else(|| "Book".to_string());

    let mut search_docs: Vec<SearchDocument> = Vec::new();
    let mut all_items: Vec<ParsedMdItem> = Vec::new();

    for entry in WalkDir::new(&src_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            let content = fs::read_to_string(path)?;
            let item =
                parse_markdown_with_path(&content, Some(&registry), Some(PathBuf::from(path)));

            let rel_path = path.strip_prefix(&src_dir)?;
            let rel_path_str = rel_path.to_string_lossy().replace(".md", "");
            let file_stem = rel_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Chapter".to_string());

            let plain_content = item
                .rendered_content
                .as_deref()
                .unwrap_or("")
                .replace("<", " <")
                .split_whitespace()
                .filter(|w| !w.starts_with('<') || w.ends_with('>'))
                .collect::<Vec<_>>()
                .join(" ");

            search_docs.push(SearchDocument {
                id: rel_path_str.clone(),
                title: item
                    .metadata
                    .get("title")
                    .cloned()
                    .unwrap_or_else(|| file_stem.clone()),
                content: plain_content,
                url: format!("{}.html", rel_path_str.replace("src/", "")),
                tags: vec![],
            });

            all_items.push((
                path.strip_prefix(&src_dir).unwrap().into(),
                output.join(rel_path).with_extension("html"),
                file_stem,
                item.rendered_content.unwrap_or_default(),
                item.metadata,
            ));
        }
    }

    let search_index_len = search_docs.len();

    // Build ChapterStore data for Vue SPA (mdbook template)
    let chapter_stores: Vec<ChapterStore> = if let Some(ref summary) = summary {
        build_all_chapter_stores(&summary.chapters, &all_items)
    } else {
        Vec::new()
    };
    let search_index = if search_index_len > 0 {
        let mut idx = rustpress_core::search::SearchIndex::new();
        for doc in &search_docs {
            idx.add_document(doc.clone());
        }
        let built = idx.build();
        serde_json::to_string(&built).unwrap_or_else(|e| {
            eprintln!("Search JSON error: {}", e);
            String::new()
        })
    } else {
        String::new()
    };

    let landing_page_store = {
        let all_posts: Vec<_> = all_items
            .iter()
            .filter(|(rp, _, _, _, _)| rp.to_string_lossy().contains("posts/"))
            .collect();

        let posts: Vec<BlogPostSummary> = all_posts
            .iter()
            .map(|(rp, _, _, _, meta)| {
                let post_tags: Vec<String> = meta
                    .get("tags")
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();
                BlogPostSummary {
                    id: rp.to_string_lossy().replace(".md", ""),
                    title: meta.get("title").cloned().unwrap_or_default(),
                    date: meta.get("date").cloned().unwrap_or_default(),
                    author: meta.get("author").cloned().unwrap_or_default(),
                    excerpt: String::new(),
                    tags: post_tags,
                    url: format!("{}.html", rp.to_string_lossy().replace(".md", "")),
                }
            })
            .collect();

        let mut all_tags: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for (_, _, _, _, meta) in &all_posts {
            if let Some(tags) = meta.get("tags") {
                for tag in tags.split(',') {
                    *all_tags.entry(tag.trim().to_string()).or_insert(0) += 1;
                }
            }
        }
        let tags: Vec<TagCount> = all_tags
            .into_iter()
            .map(|(name, count)| TagCount { name, count })
            .collect();

        let recent_posts: Vec<BlogPostSummary> = all_posts
            .iter()
            .take(5)
            .map(|(rp, _, _, _, meta)| BlogPostSummary {
                id: rp.to_string_lossy().replace(".md", ""),
                title: meta.get("title").cloned().unwrap_or_default(),
                date: meta.get("date").cloned().unwrap_or_default(),
                author: meta.get("author").cloned().unwrap_or_default(),
                excerpt: String::new(),
                tags: meta
                    .get("tags")
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default(),
                url: format!("{}.html", rp.to_string_lossy().replace(".md", "")),
            })
            .collect();

        BlogIndexStore {
            title: book_title.clone(),
            description: "A blog about Rust".to_string(),
            content: String::new(),
            posts,
            tags,
            recent_posts,
            search_index: search_index.clone(),
        }
    };

    for (rel_path, output_path, file_stem, rendered, metadata) in &all_items {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let rel_path_str = rel_path.to_string_lossy().replace(".md", "");

        let html = if let Some(ref tmpl_name) = builtin_template_name {
            let is_post = rel_path_str.contains("posts/");
            let is_landing = rel_path_str == "landing";
            let is_mdbook = tmpl_name == "mdbook";
            let template_to_use = if is_landing {
                "blog/index"
            } else if is_post {
                "blog/post"
            } else {
                tmpl_name.as_str()
            };

            if is_landing {
                // Use Vue SPA rendering for blog index
                render_blog_index_vue(&landing_page_store)
            } else if is_mdbook {
                // Use Vue SPA rendering for mdbook
                // Flatten all chapters (including nested children) to find current
                let all_chapters_flat = flatten_chapter_stores(&chapter_stores);
                let current_store = all_chapters_flat
                    .iter()
                    .find(|ch| {
                        ch.url.contains(&rel_path_str)
                            || rel_path_str.ends_with(&ch.url.replace(".html", ""))
                    })
                    .cloned();
                if let Some(current) = current_store {
                    render_mdbook_vue(&book_title, &chapter_stores, &current, &search_index)
                } else {
                    render_with_template(
                        &ContentItem {
                            path: None,
                            content: String::new(),
                            metadata: metadata.clone(),
                            rendered_content: Some(rendered.clone()),
                            related_items: vec![],
                            image_references: vec![],
                        },
                        template_to_use,
                        get_builtin_template(template_to_use).unwrap_or_default(),
                    )
                }
            } else if let Some(template_content) = get_builtin_template(template_to_use) {
                let mut metadata = metadata.clone();

                if is_post {
                    metadata.insert("content".to_string(), rendered.clone());
                    // Tags already comma-separated in metadata
                } else {
                    let chapter_title = metadata
                        .get("title")
                        .cloned()
                        .unwrap_or_else(|| file_stem.clone());

                    metadata.insert("title".to_string(), book_title.clone());
                    metadata.insert("content".to_string(), rendered.clone());
                    metadata.insert("chapter_title".to_string(), chapter_title);
                    metadata.insert("nav_tree".to_string(), nav_tree.clone());
                    metadata.insert("search_index".to_string(), search_index.clone());

                    let (prev_ch, next_ch) = summary
                        .as_ref()
                        .map(|s| render_prev_next(&s.chapters, Path::new(&rel_path_str), &src_dir))
                        .unwrap_or((String::new(), String::new()));
                    metadata.insert("prev_chapter".to_string(), prev_ch);
                    metadata.insert("next_chapter".to_string(), next_ch);
                }

                let content_item = ContentItem {
                    path: None,
                    content: String::new(),
                    metadata,
                    rendered_content: Some(rendered.clone()),
                    related_items: vec![],
                    image_references: vec![],
                };

                render_with_template(&content_item, template_to_use, template_content)
            } else {
                render_html(&ContentItem {
                    path: None,
                    content: String::new(),
                    metadata: metadata.clone(),
                    rendered_content: Some(rendered.clone()),
                    related_items: vec![],
                    image_references: vec![],
                })
            }
        } else if let Some(ref content) = custom_template_content {
            render_with_template(
                &ContentItem {
                    path: None,
                    content: String::new(),
                    metadata: metadata.clone(),
                    rendered_content: Some(rendered.clone()),
                    related_items: vec![],
                    image_references: vec![],
                },
                "custom",
                content,
            )
        } else {
            render_html(&ContentItem {
                path: None,
                content: String::new(),
                metadata: metadata.clone(),
                rendered_content: Some(rendered.clone()),
                related_items: vec![],
                image_references: vec![],
            })
        };

        if html.len() < 500 {
            eprintln!(
                "WARNING: Generated file '{}' is very small ({} bytes). Possible template issue.",
                output_path.display(),
                html.len()
            );
        }

        fs::write(output_path, &html)?;

        println!("Generated: {}", &output_path.display());
    }

    if let Some(ref summary) = summary {
        let first_chapter = get_all_chapters(&summary.chapters).into_iter().next();
        let first_path = first_chapter.map(|c| {
            c.path
                .to_string_lossy()
                .replace(".md", "")
                .replace("src/", "")
        });

        let index_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <meta http-equiv="refresh" content="0; url='{}.html'">
</head>
<body>
    <p>Redirecting to <a href="{}.html">{}</a>...</p>
</body>
</html>"#,
            book_title,
            first_path.as_deref().unwrap_or("intro"),
            first_path.as_deref().unwrap_or("intro"),
            first_path.as_deref().unwrap_or("intro")
        );
        fs::write(output.join("index.html"), index_html)?;
        println!("Generated: {}/index.html", output.display());
    } else if builtin_template_name
        .as_ref()
        .is_some_and(|t| t == "blog" || t == "blog/index")
    {
        let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Blog</title>
    <meta http-equiv="refresh" content="0; url='landing.html'">
</head>
<body>
    <p>Redirecting to <a href="landing.html">blog</a>...</p>
</body>
</html>"#;
        fs::write(output.join("index.html"), index_html)?;
        println!("Generated: {}/index.html", output.display());
    }

    // Generate RSS feed.xml from any items with a date in metadata
    let dated_items: Vec<_> = all_items
        .iter()
        .filter(|(_, _, _, _, meta)| meta.contains_key("date"))
        .collect();

    if !dated_items.is_empty() {
        let feed_title = book_title.clone();
        let description = book_toml
            .as_ref()
            .and_then(|b| b.book.description.clone())
            .unwrap_or_else(|| "Rustpress site".to_string());

        let mut feed = RssFeed::new(&feed_title, &description, "/");

        // Sort by date descending
        let mut dated_sorted = dated_items.clone();
        dated_sorted.sort_by(|a, b| {
            let date_a = a.4.get("date").map(|d| d.as_str()).unwrap_or("");
            let date_b = b.4.get("date").map(|d| d.as_str()).unwrap_or("");
            date_b.cmp(date_a)
        });

        // Limit to 20 items for the feed
        for (rp, _, _, rendered, meta) in dated_sorted.iter().take(20) {
            let title = meta.get("title").cloned().unwrap_or_default();
            let date_str = meta.get("date").cloned().unwrap_or_default();
            let author = meta.get("author").cloned();
            let tags: Vec<String> = meta
                .get("tags")
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let rel_path = rp.to_string_lossy().replace(".md", "");
            let url = format!("{}.html", rel_path.replace("src/", ""));
            let pub_date = parse_date_to_rfc2822(&date_str);
            let description = strip_html(rendered);

            feed.add_item(RssItem {
                title,
                link: url.clone(),
                description,
                author,
                pub_date,
                categories: tags,
                guid: url,
                content_html: Some(rendered.clone()),
            });
        }

        let feed_path = output.join("feed.xml");
        feed.write_to_file(&feed_path)?;
        println!("Generated: {}/feed.xml", output.display());
    }

    println!("Build completed successfully.");
    Ok(())
}

/// Guess a Content-Type header value from a file extension.
fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "xml" => "application/xml",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

/// Minimal percent-decoding so URLs with %20 etc. map to real file names.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Serve a single request out of the static `root` directory.
fn handle_request(
    request: tiny_http::Request,
    root: &Path,
    version: &Arc<AtomicUsize>,
) -> Result<()> {
    let raw_url = request.url().to_string();
    let path_part = raw_url.split(['?', '#']).next().unwrap_or("/");
    let decoded = percent_decode(path_part);
    let trimmed = decoded.trim_start_matches('/');

    // Guard against path traversal.
    if trimmed.split('/').any(|seg| seg == "..") {
        let resp = Response::from_string("400 Bad Request").with_status_code(400);
        request.respond(resp)?;
        return Ok(());
    }

    // Special endpoint for live reload polling
    if trimmed == "__rustpress_poll" {
        let v = version.load(Ordering::SeqCst);
        let resp = Response::from_string(v.to_string())
            .with_header(Header::from_bytes(&b"Content-Type"[..], b"text/plain").unwrap());
        request.respond(resp)?;
        return Ok(());
    }

    let mut file_path = if trimmed.is_empty() {
        root.join("index.html")
    } else {
        root.join(trimmed)
    };

    if file_path.is_dir() {
        file_path = file_path.join("index.html");
    }

    let serve = |req: tiny_http::Request, p: &Path, inject: bool| -> Result<()> {
        if inject {
            let content = fs::read_to_string(p)?;
            let ctype = content_type(p);
            let header = Header::from_bytes(&b"Content-Type"[..], ctype.as_bytes())
                .map_err(|_| anyhow::anyhow!("invalid content-type header"))?;
            req.respond(Response::from_string(content).with_header(header))?;
        } else {
            let file = File::open(p)?;
            let ctype = content_type(p);
            let header = Header::from_bytes(&b"Content-Type"[..], ctype.as_bytes())
                .map_err(|_| anyhow::anyhow!("invalid content-type header"))?;
            req.respond(Response::from_file(file).with_header(header))?;
        }
        Ok(())
    };

    let inject_js = |html: String| -> String {
        let script = format!(
            r#"<script>
            let lastVersion = {};
            async function checkVersion() {{
                try {{
                    const res = await fetch('/__rustpress_poll');
                    const v = parseInt(await res.text(), 10);
                    if (v !== lastVersion) {{
                        lastVersion = v;
                        location.reload();
                    }}
                }} catch(e) {{}}
            }}
            setInterval(checkVersion, 1000);
            </script>"#,
            version.load(Ordering::SeqCst)
        );
        html.replacen("</body>", &format!("{}\n</body>", script), 1)
    };

    if file_path.exists() {
        let is_html = content_type(&file_path).contains("html");
        if is_html {
            let content = fs::read_to_string(&file_path)?;
            let content = inject_js(content);
            let header = Header::from_bytes(&b"Content-Type"[..], b"text/html")
                .map_err(|_| anyhow::anyhow!("invalid content-type header"))?;
            request.respond(Response::from_string(content).with_header(header))?;
        } else {
            serve(request, &file_path, false)?;
        }
    } else {
        // Allow extensionless routes like /intro to resolve to /intro.html
        let html_path = file_path.with_extension("html");
        if html_path.exists() {
            let is_html = content_type(&html_path).contains("html");
            if is_html {
                let content = fs::read_to_string(&html_path)?;
                let content = inject_js(content);
                let header = Header::from_bytes(&b"Content-Type"[..], b"text/html")
                    .map_err(|_| anyhow::anyhow!("invalid content-type header"))?;
                request.respond(Response::from_string(content).with_header(header))?;
            } else {
                serve(request, &html_path, false)?;
            }
        } else {
            let resp = Response::from_string("404 Not Found").with_status_code(404);
            request.respond(resp)?;
        }
    }

    Ok(())
}

/// Decide whether a filesystem event should trigger a rebuild.
///
/// Ignores pure access events and any event whose paths are all inside the
/// output directory (otherwise writing the build output would loop forever).
fn event_is_relevant(res: &notify::Result<notify::Event>, output_abs: Option<&Path>) -> bool {
    let event = match res {
        Ok(e) => e,
        Err(_) => return false,
    };

    if matches!(event.kind, notify::EventKind::Access(_)) {
        return false;
    }

    if let Some(out) = output_abs
        && !event.paths.is_empty()
            && event.paths.iter().all(|p| {
                p.canonicalize()
                    .map(|cp| cp.starts_with(out))
                    .unwrap_or_else(|_| p.starts_with(out))
            })
        {
            return false;
        }

    true
}

/// Watch input/template, rebuild on change, and serve `output` over HTTP.
fn run_dev(input: &Path, output: &Path, template: &Option<PathBuf>, port: u16) -> Result<()> {
    // Version counter for live reload - incremented after each rebuild.
    let version = Arc::new(AtomicUsize::new(0));
    let version_clone = Arc::clone(&version);

    // Initial build (don't bail on failure — let the watcher recover).
    if run_build(input, output, template).is_ok() {
        version_clone.fetch_add(1, Ordering::SeqCst);
    }

    // Start the static file server on a background thread.
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Failed to start server on {}: {}", addr, e))?;
    let server = Arc::new(server);
    println!("Dev server running at http://{}", addr);

    {
        let server = Arc::clone(&server);
        let root = output.to_path_buf();
        let version_for_poll = Arc::clone(&version);
        std::thread::spawn(move || {
            for request in server.incoming_requests() {
                if let Err(e) = handle_request(request, &root, &version_for_poll) {
                    eprintln!("Request error: {}", e);
                }
            }
        });
    }

    // Set up the file watcher.
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;

    watcher.watch(input, RecursiveMode::Recursive)?;
    println!("Watching: {}", input.display());

    if let Some(tmpl) = template
        && tmpl.exists() {
            let mode = if tmpl.is_dir() {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            watcher.watch(tmpl, mode)?;
            println!("Watching: {}", tmpl.display());
        }

    let output_abs = output.canonicalize().ok();
    let version_for_build = Arc::clone(&version);

    println!("Press Ctrl+C to stop.");

    while let Ok(first) = rx.recv() {
        let mut relevant = event_is_relevant(&first, output_abs.as_deref());

        // Debounce: collect any further events that land within a short window.
        std::thread::sleep(Duration::from_millis(200));
        while let Ok(res) = rx.try_recv() {
            if event_is_relevant(&res, output_abs.as_deref()) {
                relevant = true;
            }
        }

        if !relevant {
            continue;
        }

        println!("Change detected, rebuilding...");
        match run_build(input, output, template) {
            Ok(_) => {
                version_for_build.fetch_add(1, Ordering::SeqCst);
                println!("Rebuild complete.\n")
            }
            Err(e) => eprintln!("Rebuild failed: {}\n", e),
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut registry = ComponentRegistry::new();
    register_builtin_components(&mut registry);

    match &cli.command {
        Commands::Convert {
            input,
            output,
            template,
        } => {
            let content = fs::read_to_string(input)
                .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", input.display(), e))?;

            let item = parse_markdown_with_path(&content, Some(&registry), Some(input.clone()));

            let html = if template
                .as_ref()
                .is_some_and(|p| p.to_string_lossy().contains("slideshow"))
            {
                render_slideshow_vue(&item)
            } else if let Some(template_path) = template {
                let template_name_str = template_path.to_string_lossy();
                let (render_name, template_content) = if template_name_str == "blog" {
                    (
                        "blog/post",
                        get_builtin_template("blog/post").unwrap().to_string(),
                    )
                } else if let Some(builtin) = get_builtin_template(&template_name_str) {
                    (template_name_str.as_ref(), builtin.to_string())
                } else if template_path.exists() {
                    ("custom", fs::read_to_string(template_path)?)
                } else {
                    return Err(anyhow::anyhow!(
                        "Template not found: {}",
                        template_path.display()
                    ));
                };
                render_with_template(&item, render_name, &template_content)
            } else {
                render_html(&item)
            };

            match output {
                Some(path) => {
                    let mut file = fs::File::create(path)?;
                    file.write_all(html.as_bytes())?;

                    if let Some(parent) = path.parent() {
                        for img_path in &item.image_references {
                            if img_path.exists() {
                                let dest = parent.join(img_path.file_name().unwrap_or_default());
                                fs::copy(img_path, dest)?;
                            }
                        }
                    }
                }
                None => println!("{}", html),
            }
        }

        Commands::Build {
            input,
            output,
            template,
        } => {
            run_build(input, output, template)?;
        }

        Commands::Dev {
            input,
            output,
            template,
            port,
        } => {
            run_dev(input, output, template, *port)?;
        }
    }

    Ok(())
}
