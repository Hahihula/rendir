use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use notify::{RecursiveMode, Watcher};
use rendir_core::components::{ComponentRegistry, builtins::register_builtin_components};
use rendir_core::{
    get_builtin_template,
    i18n::I18nBuilder,
    mdbook::{BookToml, Chapter, Summary},
    parse_markdown_with_path, render_blog_index_vue, render_html, render_mdbook_vue,
    render_slideshow_vue, render_with_template,
    rss::{RssFeed, RssItem, parse_date_to_rfc2822, strip_html},
    search::{SearchDocument, SearchIndex},
    types::{
        BlogIndexStore, BlogPostSummary, ChapterNav, ChapterStore, ContentItem, Language, TagCount,
    },
};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Cursor};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::time::Duration;
use tiny_http::{Header, Response, Server};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a single markdown file to HTML.
    Convert {
        /// Input markdown file.
        #[arg(short, long)]
        input: PathBuf,
        /// Output HTML file (prints to stdout if omitted).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Built-in template name, a local template path, or a registry name.
        #[arg(short, long)]
        template: Option<PathBuf>,
    },
    /// Build a site from a directory of markdown files.
    Build {
        /// Input directory.
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory.
        #[arg(short, long)]
        output: PathBuf,
        /// Built-in template name, a local template path, or a registry name.
        #[arg(short, long)]
        template: Option<PathBuf>,
    },
    /// Watch the input/template, rebuild on change, and serve the result locally.
    Dev {
        /// Input directory.
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory.
        #[arg(short, long)]
        output: PathBuf,
        /// Built-in template name, a local template path, or a registry name.
        #[arg(short, long)]
        template: Option<PathBuf>,
        /// Port for the local dev server.
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

// ---------------------------------------------------------------------------
// Template resolution
// ---------------------------------------------------------------------------

/// Where the template(s) for a build come from.
///
/// Resolution order is: built-in name → existing local path → template
/// registry. The registry does not exist yet (see [`TemplateSource::from_registry`]).
enum TemplateSource {
    /// A recognised built-in template (`blog`, `slideshow`, `mdbook`, …),
    /// looked up by name via [`get_builtin_template`].
    Builtin { name: String },
    /// A custom template on the local filesystem (a single `.html` file, or a
    /// directory whose `index.html` is the entry template).
    Custom { path: PathBuf },
    /// A template fetched from the remote template registry.
    // TODO: constructed once the template registry exists.
    #[allow(dead_code)]
    Registry { name: String },
}

impl TemplateSource {
    /// Resolve the `--template` argument to a concrete source.
    fn resolve(arg: &Path) -> Result<Self> {
        let name = arg.to_string_lossy();

        // 1. A reserved built-in name wins (e.g. `blog`, `slideshow`, `mdbook`).
        if get_builtin_template(&name).is_some() {
            return Ok(Self::Builtin {
                name: name.into_owned(),
            });
        }
        // 2. An existing path on disk (file or directory).
        if arg.exists() {
            return Ok(Self::Custom {
                path: arg.to_path_buf(),
            });
        }
        // 3. Otherwise, fall back to the template registry.
        Self::from_registry(&name)
    }

    /// TODO: look the template up in the remote template repository once it
    /// exists, returning `Self::Registry { name }` on success or a descriptive
    /// error if it cannot be found.
    fn from_registry(_name: &str) -> Result<Self> {
        unimplemented!("template registry lookup is not implemented yet")
    }

    /// The built-in name, if this source is a built-in.
    fn builtin_name(&self) -> Option<&str> {
        match self {
            Self::Builtin { name } => Some(name),
            _ => None,
        }
    }
}

/// Read a custom template: the file itself, or `index.html` inside a directory.
fn read_custom_template(path: &Path) -> Result<String> {
    let file = if path.is_dir() {
        path.join("index.html")
    } else {
        path.to_path_buf()
    };
    fs::read_to_string(&file)
        .map_err(|e| anyhow!("failed to read custom template {}: {e}", file.display()))
}

// ---------------------------------------------------------------------------
// Remote assets
// ---------------------------------------------------------------------------

fn download_remote_asset(url: &str, dest: &Path) -> Result<()> {
    let response = reqwest::blocking::get(url)?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest)?;
    let mut content = Cursor::new(response.bytes()?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

fn get_remote_filename(url: &str) -> String {
    url.split('/')
        .next_back()
        .unwrap_or("asset")
        .split('?')
        .next()
        .unwrap_or("asset")
        .to_string()
}

// ---------------------------------------------------------------------------
// Site model + small shared helpers
// ---------------------------------------------------------------------------

struct SiteItem {
    rel_path: PathBuf,
    output_path: PathBuf,
    file_stem: String,
    rendered: String,
    metadata: HashMap<String, String>,
    is_fallback: bool,
    asset_references: Vec<PathBuf>,
    remote_references: Vec<String>,
}

impl SiteItem {
    /// Relative path with the `.md` extension stripped.
    fn rel_path_str(&self) -> String {
        self.rel_path.to_string_lossy().replace(".md", "")
    }

    /// A minimal [`ContentItem`] carrying just this item's metadata and HTML.
    fn content_item(&self) -> ContentItem {
        ContentItem {
            metadata: self.metadata.clone(),
            rendered_content: Some(self.rendered.clone()),
            ..Default::default()
        }
    }
}

/// Split a comma-separated `tags` metadata value into trimmed tags.
fn split_tags(raw: Option<&String>) -> Vec<String> {
    raw.map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default()
}

/// Slug for a chapter path: extension and any `src/` prefix removed.
fn chapter_slug(path: &Path) -> String {
    path.to_string_lossy().replace(".md", "").replace("src/", "")
}

/// Output URL for a chapter path (`chapter_slug` + `.html`).
fn chapter_url(path: &Path) -> String {
    format!("{}.html", chapter_slug(path))
}

fn warn_if_small(path: &Path, html: &str) {
    if html.len() < 500 {
        eprintln!(
            "WARNING: Generated file '{}' is very small ({} bytes). Possible template issue.",
            path.display(),
            html.len()
        );
    }
}

// ---------------------------------------------------------------------------
// Markdown scanning
// ---------------------------------------------------------------------------

fn scan_markdown_dir(
    src_dir: &Path,
    output_dir: &Path,
    registry: &ComponentRegistry,
    lang_code: Option<&str>,
) -> Result<(Vec<SearchDocument>, Vec<SiteItem>)> {
    let mut search_docs = Vec::new();
    let mut items = Vec::new();

    for entry in WalkDir::new(src_dir) {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() || path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let item = parse_markdown_with_path(&content, Some(registry), Some(path.to_path_buf()));

        let rel_path = path.strip_prefix(src_dir).unwrap_or(path);
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

        let (id, url) = match lang_code {
            Some(code) => (
                format!("{code}/{rel_path_str}"),
                format!("{code}/{}.html", rel_path_str.replace("src/", "")),
            ),
            None => (
                rel_path_str.clone(),
                format!("{}.html", rel_path_str.replace("src/", "")),
            ),
        };

        search_docs.push(SearchDocument {
            id,
            title: item
                .metadata
                .get("title")
                .cloned()
                .unwrap_or_else(|| file_stem.clone()),
            content: plain_content,
            url,
            tags: vec![],
        });

        items.push(SiteItem {
            output_path: output_dir.join(rel_path).with_extension("html"),
            rel_path: rel_path.to_path_buf(),
            file_stem,
            rendered: item.rendered_content.unwrap_or_default(),
            metadata: item.metadata,
            is_fallback: false,
            asset_references: item.image_references,
            remote_references: item.remote_references,
        });
    }

    Ok((search_docs, items))
}

/// Build the JSON search index from a set of documents (empty string if none).
fn build_search_index(docs: &[SearchDocument]) -> String {
    if docs.is_empty() {
        return String::new();
    }
    let mut idx = SearchIndex::new();
    for doc in docs {
        idx.add_document(doc.clone());
    }
    serde_json::to_string(&idx.build()).unwrap_or_else(|e| {
        eprintln!("Search JSON error: {e}");
        String::new()
    })
}

// ---------------------------------------------------------------------------
// mdbook navigation + chapter stores
// ---------------------------------------------------------------------------

fn render_nav_tree(chapters: &[Chapter]) -> String {
    let mut html = String::new();
    for chapter in chapters {
        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a>",
            chapter_url(&chapter.path),
            chapter.title
        ));
        if !chapter.children.is_empty() {
            html.push_str("<ul class=\"section\">");
            html.push_str(&render_nav_tree(&chapter.children));
            html.push_str("</ul>");
        }
        html.push_str("</li>\n");
    }
    html
}

/// Depth-first flattening of the chapter tree.
fn get_all_chapters(chapters: &[Chapter]) -> Vec<&Chapter> {
    fn collect<'a>(chapters: &'a [Chapter], out: &mut Vec<&'a Chapter>) {
        for ch in chapters {
            out.push(ch);
            collect(&ch.children, out);
        }
    }
    let mut out = Vec::new();
    collect(chapters, &mut out);
    out
}

/// Rendered HTML for the item matching a chapter path.
fn rendered_for(items: &[SiteItem], path: &Path) -> String {
    let slug = chapter_slug(path);
    items
        .iter()
        .find(|i| i.rel_path_str().replace("src/", "") == slug)
        .map(|i| i.rendered.clone())
        .unwrap_or_default()
}

fn build_chapter_store(
    chapter: &Chapter,
    all_chapters: &[&Chapter],
    current_idx: usize,
    content: String,
    all_items: &[SiteItem],
    plain_text_map: &HashMap<String, String>,
) -> ChapterStore {
    let url = chapter_url(&chapter.path);
    let nav = |ch: &Chapter| ChapterNav {
        title: ch.title.clone(),
        url: chapter_url(&ch.path),
    };

    let prev_chapter = (current_idx > 0).then(|| nav(all_chapters[current_idx - 1]));
    let next_chapter = (current_idx + 1 < all_chapters.len()).then(|| nav(all_chapters[current_idx + 1]));

    let children = chapter
        .children
        .iter()
        .map(|child| {
            let child_idx = all_chapters
                .iter()
                .position(|c| c.path == child.path)
                .unwrap_or(current_idx);
            let child_content = rendered_for(all_items, &child.path);
            build_chapter_store(
                child,
                all_chapters,
                child_idx,
                child_content,
                all_items,
                plain_text_map,
            )
        })
        .collect();

    ChapterStore {
        title: chapter.title.clone(),
        plain_text: plain_text_map
            .get(&url.replace(".html", ""))
            .cloned()
            .unwrap_or_default(),
        url,
        content,
        level: chapter.level,
        children,
        prev_chapter,
        next_chapter,
        translations: Vec::new(),
    }
}

/// Build a [`ChapterStore`] for every chapter (including nested ones).
fn build_all_chapter_stores(
    chapters: &[Chapter],
    all_items: &[SiteItem],
    plain_text_map: &HashMap<String, String>,
) -> Vec<ChapterStore> {
    let all = get_all_chapters(chapters);
    all.iter()
        .copied()
        .enumerate()
        .map(|(idx, ch)| {
            let content = rendered_for(all_items, &ch.path);
            build_chapter_store(ch, &all, idx, content, all_items, plain_text_map)
        })
        .collect()
}

/// Flatten chapter stores (including nested children) for lookups.
fn flatten_chapter_stores(chapters: &[ChapterStore]) -> Vec<ChapterStore> {
    let mut out = Vec::new();
    for ch in chapters {
        out.push(ch.clone());
        out.extend(flatten_chapter_stores(&ch.children));
    }
    out
}

/// Prev/next anchor HTML for the chapter at `current`.
fn render_prev_next(chapters: &[Chapter], current: &Path) -> (String, String) {
    let all = get_all_chapters(chapters);
    let current_slug = chapter_slug(current);
    let current_idx = all
        .iter()
        .position(|ch| chapter_slug(&ch.path) == current_slug)
        .unwrap_or(0);

    let prev = (current_idx > 0)
        .then(|| {
            let p = all[current_idx - 1];
            format!("<a href=\"{}\">← {}</a>", chapter_url(&p.path), p.title)
        })
        .unwrap_or_default();

    let next = (current_idx + 1 < all.len())
        .then(|| {
            let n = all[current_idx + 1];
            format!("<a href=\"{}\">{} →</a>", chapter_url(&n.path), n.title)
        })
        .unwrap_or_default();

    (prev, next)
}

// ---------------------------------------------------------------------------
// Blog store helpers (shared by the single-language and i18n builds)
// ---------------------------------------------------------------------------

fn post_summary(item: &SiteItem, plain_text: &HashMap<String, String>) -> BlogPostSummary {
    let id = item.rel_path_str();
    let url = format!("{id}.html");
    let content = plain_text.get(&id).cloned().unwrap_or_default();
    BlogPostSummary {
        title: item.metadata.get("title").cloned().unwrap_or_default(),
        date: item.metadata.get("date").cloned().unwrap_or_default(),
        author: item.metadata.get("author").cloned().unwrap_or_default(),
        excerpt: String::new(),
        content,
        tags: split_tags(item.metadata.get("tags")),
        url,
        id,
    }
}

fn aggregate_tags(posts: &[&SiteItem]) -> Vec<TagCount> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for item in posts {
        for tag in split_tags(item.metadata.get("tags")) {
            *counts.entry(tag).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(name, count)| TagCount { name, count })
        .collect()
}

fn build_blog_index_store(
    items: &[SiteItem],
    plain_text: &HashMap<String, String>,
    title: String,
    description: String,
    search_index: String,
    languages: Vec<Language>,
) -> BlogIndexStore {
    let posts_items: Vec<&SiteItem> = items
        .iter()
        .filter(|i| i.rel_path_str().contains("posts/"))
        .collect();

    BlogIndexStore {
        title,
        description,
        content: String::new(),
        posts: posts_items.iter().map(|i| post_summary(i, plain_text)).collect(),
        tags: aggregate_tags(&posts_items),
        recent_posts: posts_items
            .iter()
            .take(5)
            .map(|i| post_summary(i, plain_text))
            .collect(),
        search_index,
        languages,
    }
}

// ---------------------------------------------------------------------------
// Single-language build
// ---------------------------------------------------------------------------

/// Shared, read-only state passed to the per-page renderer.
struct BuildContext<'a> {
    book_title: String,
    nav_tree: String,
    search_index: String,
    landing_store: BlogIndexStore,
    chapter_stores: Vec<ChapterStore>,
    summary: Option<&'a Summary>,
}

/// Resolve the source directory: a configured non-default `book.src`, else
/// `<input>/src`, else `<input>` itself.
fn resolve_src_dir(input: &Path, book_toml: Option<&BookToml>) -> PathBuf {
    let configured = book_toml
        .and_then(|b| {
            let src = b.book.src.as_str();
            (!src.is_empty() && src != "src").then(|| input.join(src))
        })
        .filter(|p| p.exists());

    configured.unwrap_or_else(|| {
        let default = input.join("src");
        if default.exists() {
            default
        } else {
            input.to_path_buf()
        }
    })
}

fn run_build(input: &Path, output: &Path, template: &Option<PathBuf>) -> Result<()> {
    let mut registry = ComponentRegistry::new();
    register_builtin_components(&mut registry);
    fs::create_dir_all(output)?;

    let source = template.as_deref().map(TemplateSource::resolve).transpose()?;

    let book_toml = input
        .join("book.toml")
        .exists()
        .then(|| BookToml::from_path(&input.join("book.toml")).unwrap_or_default());

    let src_dir = resolve_src_dir(input, book_toml.as_ref());

    let summary = src_dir
        .join("SUMMARY.md")
        .exists()
        .then(|| Summary::from_path(&src_dir.join("SUMMARY.md")))
        .flatten();

    let languages = I18nBuilder::detect_languages(&src_dir);
    if languages.len() > 1 {
        return run_build_i18n(input, output, source.as_ref(), &languages);
    }

    let book_title = book_toml
        .as_ref()
        .and_then(|b| b.book.title.clone())
        .unwrap_or_else(|| "Book".to_string());

    let (search_docs, all_items) = scan_markdown_dir(&src_dir, output, &registry, None)?;

    let plain_text_map: HashMap<String, String> = search_docs
        .iter()
        .map(|d| (d.id.clone(), d.content.clone()))
        .collect();

    let search_index = build_search_index(&search_docs);

    let chapter_stores = summary
        .as_ref()
        .map(|s| build_all_chapter_stores(&s.chapters, &all_items, &plain_text_map))
        .unwrap_or_default();

    let landing_store = build_blog_index_store(
        &all_items,
        &plain_text_map,
        book_title.clone(),
        "A blog about Rust".to_string(),
        search_index.clone(),
        languages.clone(),
    );

    let ctx = BuildContext {
        book_title: book_title.clone(),
        nav_tree: summary.as_ref().map(|s| render_nav_tree(&s.chapters)).unwrap_or_default(),
        search_index,
        landing_store,
        chapter_stores,
        summary: summary.as_ref(),
    };

    for item in &all_items {
        if let Some(parent) = item.output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let html = match &source {
            Some(src) => render_page(src, item, &ctx)?,
            None => render_html(&item.content_item()),
        };

        warn_if_small(&item.output_path, &html);
        fs::write(&item.output_path, &html)?;
        copy_item_assets(item, &src_dir)?;
        println!("Generated: {}", item.output_path.display());
    }

    write_root_index(output, source.as_ref(), summary.as_ref(), &book_title)?;
    write_rss_feed(output, &all_items, &book_title, book_toml.as_ref())?;

    println!("Build completed successfully.");
    Ok(())
}

/// Render a single page using a resolved template source.
fn render_page(source: &TemplateSource, item: &SiteItem, ctx: &BuildContext) -> Result<String> {
    match source {
        TemplateSource::Builtin { name } => Ok(render_builtin_page(name, item, ctx)),
        TemplateSource::Custom { path } => {
            let content = read_custom_template(path)?;
            Ok(render_with_template(&item.content_item(), "custom", &content))
        }
        TemplateSource::Registry { .. } => {
            unimplemented!("template registry lookup is not implemented yet")
        }
    }
}

fn render_builtin_page(name: &str, item: &SiteItem, ctx: &BuildContext) -> String {
    let rel = item.rel_path_str();

    if rel == "landing" {
        return render_blog_index_vue(&ctx.landing_store);
    }
    if name == "mdbook" {
        return render_mdbook_page(&rel, item, ctx);
    }

    let is_post = rel.contains("posts/");
    let sub = if is_post { "blog/post" } else { name };

    let Some(template) = get_builtin_template(sub) else {
        return render_html(&item.content_item());
    };

    let metadata = if is_post {
        let mut m = item.metadata.clone();
        m.insert("content".to_string(), item.rendered.clone());
        m
    } else {
        build_chapter_metadata(item, ctx, &rel)
    };

    let content_item = ContentItem {
        metadata,
        rendered_content: Some(item.rendered.clone()),
        ..Default::default()
    };
    render_with_template(&content_item, sub, template)
}

fn render_mdbook_page(rel: &str, item: &SiteItem, ctx: &BuildContext) -> String {
    let current = flatten_chapter_stores(&ctx.chapter_stores)
        .into_iter()
        .find(|ch| ch.url.contains(rel) || rel.ends_with(ch.url.trim_end_matches(".html")));

    match current {
        Some(current) => {
            render_mdbook_vue(&ctx.book_title, &ctx.chapter_stores, &current, &ctx.search_index)
        }
        None => render_with_template(
            &item.content_item(),
            "mdbook",
            get_builtin_template("mdbook").unwrap_or_default(),
        ),
    }
}

/// Metadata for a non-post built-in chapter page (mdbook/slideshow chrome).
fn build_chapter_metadata(item: &SiteItem, ctx: &BuildContext, rel: &str) -> HashMap<String, String> {
    let mut m = item.metadata.clone();
    let chapter_title = m
        .get("title")
        .cloned()
        .unwrap_or_else(|| item.file_stem.clone());

    m.insert("title".to_string(), ctx.book_title.clone());
    m.insert("content".to_string(), item.rendered.clone());
    m.insert("chapter_title".to_string(), chapter_title);
    m.insert("nav_tree".to_string(), ctx.nav_tree.clone());
    m.insert("search_index".to_string(), ctx.search_index.clone());

    let (prev, next) = ctx
        .summary
        .map(|s| render_prev_next(&s.chapters, Path::new(rel)))
        .unwrap_or_default();
    m.insert("prev_chapter".to_string(), prev);
    m.insert("next_chapter".to_string(), next);
    m
}

// ---------------------------------------------------------------------------
// Asset copying / RSS / index pages
// ---------------------------------------------------------------------------

/// Copy local assets and download remote ones next to a generated page.
fn copy_item_assets(item: &SiteItem, src_dir: &Path) -> Result<()> {
    let Some(parent) = item.output_path.parent() else {
        return Ok(());
    };

    for asset in &item.asset_references {
        if !asset.exists() {
            continue;
        }
        let relative = asset.strip_prefix(src_dir).unwrap_or(asset);
        let dest = parent.join(asset_dest(&item.rel_path, relative));
        if let Some(dp) = dest.parent() {
            fs::create_dir_all(dp).ok();
        }
        if !dest.exists() {
            if let Err(e) = fs::copy(asset, &dest) {
                eprintln!("Warning: Failed to copy asset '{}': {e}", asset.display());
            }
        }
    }

    for url in &item.remote_references {
        let dest = parent.join("remote_assets").join(get_remote_filename(url));
        if !dest.exists() {
            match download_remote_asset(url, &dest) {
                Ok(_) => println!("Downloaded remote asset: {url}"),
                Err(e) => eprintln!("Warning: Failed to download '{url}': {e}"),
            }
        }
    }

    Ok(())
}

/// Asset destination relative to the page: for a nested page, drop the leading
/// content component so the asset lands beside the page.
fn asset_dest(rel_path: &Path, relative_to_content: &Path) -> PathBuf {
    let nested = matches!(rel_path.parent(), Some(p) if p != Path::new(""));
    if !nested {
        return relative_to_content.to_path_buf();
    }
    let comps: Vec<_> = relative_to_content.components().collect();
    if comps.len() > 1 {
        PathBuf::from_iter(comps[1..].iter())
    } else {
        PathBuf::from_iter(comps.iter())
    }
}

/// Generate `feed.xml` from any items carrying a `date` (newest first, max 20).
fn write_rss_feed(
    output: &Path,
    items: &[SiteItem],
    book_title: &str,
    book_toml: Option<&BookToml>,
) -> Result<()> {
    let mut dated: Vec<&SiteItem> = items
        .iter()
        .filter(|i| i.metadata.contains_key("date"))
        .collect();
    if dated.is_empty() {
        return Ok(());
    }

    dated.sort_by(|a, b| {
        let date = |i: &SiteItem| i.metadata.get("date").cloned().unwrap_or_default();
        date(b).cmp(&date(a))
    });

    let description = book_toml
        .and_then(|b| b.book.description.clone())
        .unwrap_or_else(|| "Rendir site".to_string());
    let mut feed = RssFeed::new(book_title, &description, "/");

    for item in dated.into_iter().take(20) {
        let url = format!("{}.html", item.rel_path_str().replace("src/", ""));
        let date = item.metadata.get("date").cloned().unwrap_or_default();
        feed.add_item(RssItem {
            title: item.metadata.get("title").cloned().unwrap_or_default(),
            link: url.clone(),
            description: strip_html(&item.rendered),
            author: item.metadata.get("author").cloned(),
            pub_date: parse_date_to_rfc2822(&date),
            categories: split_tags(item.metadata.get("tags")),
            guid: url,
            content_html: Some(item.rendered.clone()),
        });
    }

    let path = output.join("feed.xml");
    feed.write_to_file(&path)?;
    println!("Generated: {}/feed.xml", output.display());
    Ok(())
}

/// Write a `<meta refresh>` redirect page.
fn write_redirect_index(
    path: &Path,
    lang: &str,
    title: &str,
    target: &str,
    link_text: &str,
) -> Result<()> {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <meta http-equiv="refresh" content="0; url='{target}'">
</head>
<body>
    <p>Redirecting to <a href="{target}">{link_text}</a>...</p>
</body>
</html>"#
    );
    fs::write(path, html)?;
    Ok(())
}

fn is_blog_name(name: &str) -> bool {
    name == "blog" || name == "blog/index"
}

/// Root `index.html` for a single-language build: redirect to the first chapter
/// (mdbook) or to the blog landing page.
fn write_root_index(
    output: &Path,
    source: Option<&TemplateSource>,
    summary: Option<&Summary>,
    book_title: &str,
) -> Result<()> {
    let index = output.join("index.html");

    if let Some(summary) = summary {
        let first = get_all_chapters(&summary.chapters)
            .into_iter()
            .next()
            .map(|c| chapter_slug(&c.path))
            .unwrap_or_else(|| "intro".to_string());
        write_redirect_index(&index, "en", book_title, &format!("{first}.html"), &first)?;
        println!("Generated: {}/index.html", output.display());
    } else if source.and_then(TemplateSource::builtin_name).is_some_and(is_blog_name) {
        write_redirect_index(&index, "en", "Blog", "landing.html", "blog")?;
        println!("Generated: {}/index.html", output.display());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Multi-language (i18n) build
// ---------------------------------------------------------------------------

fn run_build_i18n(
    input: &Path,
    output: &Path,
    source: Option<&TemplateSource>,
    languages: &[Language],
) -> Result<()> {
    let is_slideshow = source
        .and_then(TemplateSource::builtin_name)
        .is_some_and(|n| n.contains("slideshow"));

    let mut registry = ComponentRegistry::new();
    register_builtin_components(&mut registry);

    let mut i18n = I18nBuilder::new("en");
    if let Err(e) = i18n.build_index(input) {
        eprintln!("Warning: i18n index build failed: {e}");
    }

    let book_toml = input
        .join("book.toml")
        .exists()
        .then(|| BookToml::from_path(&input.join("book.toml")).unwrap_or_default());
    let src_dir = resolve_src_dir(input, book_toml.as_ref());
    let book_title = book_toml
        .as_ref()
        .and_then(|b| b.book.title.clone())
        .unwrap_or_else(|| "Site".to_string());

    let hreflang_tags = generate_hreflang_tags(languages);

    for lang in languages {
        let lang_output = output.join(&lang.code);
        fs::create_dir_all(&lang_output)?;
        println!("Building {} ({}) site...", lang.name, lang.code);

        let lang_src_dir = src_dir.join(&lang.code);
        if !lang_src_dir.exists() {
            eprintln!(
                "Warning: Language directory '{}' does not exist, skipping",
                lang_src_dir.display()
            );
            continue;
        }

        let (mut search_docs, mut all_items) =
            scan_markdown_dir(&lang_src_dir, &lang_output, &registry, Some(&lang.code))?;

        merge_fallback_items(
            &src_dir,
            &lang_src_dir,
            &lang_output,
            &registry,
            &i18n,
            languages,
            lang,
            &mut search_docs,
            &mut all_items,
        )?;

        let plain_text_map: HashMap<String, String> = search_docs
            .iter()
            .map(|d| (d.id.clone(), d.content.clone()))
            .collect();

        let landing_store = build_blog_index_store(
            &all_items,
            &plain_text_map,
            book_title.clone(),
            format!("{book_title} - {}", lang.name),
            build_search_index(&search_docs),
            languages.to_vec(),
        );

        for item in &all_items {
            if let Some(parent) = item.output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let html = render_i18n_page(item, lang, is_slideshow, &landing_store, &hreflang_tags, &i18n);
            warn_if_small(&item.output_path, &html);
            fs::write(&item.output_path, &html)?;
            println!("Generated: {}", item.output_path.display());
        }

        write_lang_index(&lang_output, lang, &book_title, is_slideshow, &all_items)?;
    }

    write_language_selector(output, &book_title, languages, is_slideshow)?;
    println!("i18n Build completed successfully.");
    Ok(())
}

/// Pull in pages missing from `lang` from other languages, marked as fallback.
#[allow(clippy::too_many_arguments)]
fn merge_fallback_items(
    src_dir: &Path,
    lang_src_dir: &Path,
    lang_output: &Path,
    registry: &ComponentRegistry,
    i18n: &I18nBuilder,
    languages: &[Language],
    lang: &Language,
    search_docs: &mut Vec<SearchDocument>,
    all_items: &mut Vec<SiteItem>,
) -> Result<()> {
    for fallback_lang in languages {
        let fallback_dir = src_dir.join(&fallback_lang.code);
        if !fallback_dir.exists() || fallback_dir == lang_src_dir {
            continue;
        }

        let existing: Vec<String> = all_items.iter().map(SiteItem::rel_path_str).collect();
        let (fb_docs, fb_items) =
            scan_markdown_dir(&fallback_dir, lang_output, registry, Some(&lang.code))?;

        for (mut doc, item) in fb_docs.into_iter().zip(fb_items) {
            let rel = item.rel_path_str();
            if existing.contains(&rel) {
                continue;
            }
            let Some((title, _content)) = i18n.get_fallback_content(&rel) else {
                continue;
            };

            doc.title = title.clone();
            search_docs.push(doc);

            let mut metadata = item.metadata.clone();
            metadata.insert("title".to_string(), title);
            all_items.push(SiteItem {
                metadata,
                is_fallback: true,
                ..item
            });
        }
    }
    Ok(())
}

fn render_i18n_page(
    item: &SiteItem,
    lang: &Language,
    is_slideshow: bool,
    landing_store: &BlogIndexStore,
    hreflang_tags: &str,
    i18n: &I18nBuilder,
) -> String {
    let rel = item.rel_path_str();
    let translations = i18n.get_translations(&rel);

    // Slideshow presentation and the blog landing page render whole-store and
    // splice in their hreflang tags.
    if is_slideshow && rel == "presentation" {
        let content_item = ContentItem {
            metadata: item.metadata.clone(),
            rendered_content: Some(item.rendered.clone()),
            language: Some(lang.code.clone()),
            translations,
            is_fallback: item.is_fallback,
            ..Default::default()
        };
        return render_slideshow_vue(&content_item).replace("{{HREFLANG_TAGS}}", hreflang_tags);
    }
    if rel == "landing" {
        return render_blog_index_vue(landing_store).replace("{{HREFLANG_TAGS}}", hreflang_tags);
    }

    // Posts and other pages render through the blog templates.
    let is_post = rel.contains("posts/");
    let mut metadata = item.metadata.clone();
    metadata.insert(
        "translations".to_string(),
        serde_json::to_string(&translations).unwrap_or_default(),
    );
    if is_post {
        metadata.insert("content".to_string(), item.rendered.clone());
    }

    let template = if is_post { "blog/post" } else { "blog/index" };
    let content_item = ContentItem {
        metadata,
        rendered_content: Some(item.rendered.clone()),
        language: Some(lang.code.clone()),
        translations,
        is_fallback: item.is_fallback,
        ..Default::default()
    };
    render_with_template(&content_item, template, get_builtin_template(template).unwrap_or_default())
}

/// Per-language `index.html` redirect into that language's entry page.
fn write_lang_index(
    lang_output: &Path,
    lang: &Language,
    book_title: &str,
    is_slideshow: bool,
    all_items: &[SiteItem],
) -> Result<()> {
    let has_presentation = all_items.iter().any(|i| i.rel_path_str() == "presentation");
    let has_landing = all_items.iter().any(|i| i.rel_path_str() == "landing");

    let (should_write, target) = if is_slideshow {
        (
            has_landing || has_presentation,
            format!("/{}/presentation.html", lang.code),
        )
    } else {
        (has_landing, format!("/{}/landing.html", lang.code))
    };

    if should_write {
        write_redirect_index(
            &lang_output.join("index.html"),
            &lang.code,
            book_title,
            &target,
            &lang.name,
        )?;
        println!("Generated: {}/index.html", lang_output.display());
    }
    Ok(())
}

/// Root language-selector page listing every available language.
fn write_language_selector(
    output: &Path,
    book_title: &str,
    languages: &[Language],
    is_slideshow: bool,
) -> Result<()> {
    let list = languages
        .iter()
        .map(|l| {
            let href = if is_slideshow {
                format!("/{}/presentation.html", l.code)
            } else {
                format!("/{}/", l.code)
            };
            format!("<li><a href=\"{href}\">{}</a></li>", l.name)
        })
        .collect::<Vec<_>>()
        .join("\n        ");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{book_title}</title>
</head>
<body>
    <h1>{book_title}</h1>
    <p>Select your language:</p>
    <ul>
        {list}
    </ul>
</body>
</html>"#
    );

    fs::write(output.join("index.html"), html)?;
    println!("Generated: {}/index.html (language selector)", output.display());
    Ok(())
}

fn generate_hreflang_tags(languages: &[Language]) -> String {
    languages
        .iter()
        .map(|lang| {
            format!(
                r#"<link rel="alternate" hreflang="{}" href="/{}">"#,
                lang.code, lang.code
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ")
}

// ---------------------------------------------------------------------------
// Single-file convert
// ---------------------------------------------------------------------------

fn convert_file(
    input: &Path,
    output: &Option<PathBuf>,
    template: &Option<PathBuf>,
    registry: &ComponentRegistry,
) -> Result<()> {
    let content = fs::read_to_string(input)
        .map_err(|e| anyhow!("Failed to read file {}: {e}", input.display()))?;
    let item = parse_markdown_with_path(&content, Some(registry), Some(input.to_path_buf()));

    let html = render_single(&item, template)?;

    match output {
        Some(path) => {
            fs::write(path, &html)?;
            copy_single_assets(path, &item)?;
        }
        None => println!("{html}"),
    }
    Ok(())
}

/// Render a single [`ContentItem`] following the same name → path → registry
/// resolution as a full build (with the convert-specific slideshow/blog cases).
fn render_single(item: &ContentItem, template: &Option<PathBuf>) -> Result<String> {
    let Some(template) = template else {
        return Ok(render_html(item));
    };
    let name = template.to_string_lossy();

    if name.contains("slideshow") {
        return Ok(render_slideshow_vue(item));
    }
    if name == "blog" {
        let tmpl = get_builtin_template("blog/post").unwrap_or_default();
        return Ok(render_with_template(item, "blog/post", tmpl));
    }
    if let Some(builtin) = get_builtin_template(&name) {
        return Ok(render_with_template(item, &name, builtin));
    }
    if template.exists() {
        let content = read_custom_template(template)?;
        return Ok(render_with_template(item, "custom", &content));
    }

    // TODO: template registry lookup (see TemplateSource::from_registry).
    unimplemented!("template registry lookup is not implemented yet")
}

fn copy_single_assets(output: &Path, item: &ContentItem) -> Result<()> {
    let Some(parent) = output.parent() else {
        return Ok(());
    };
    for img in &item.image_references {
        if img.exists() {
            let dest = parent.join(img.file_name().unwrap_or_default());
            fs::copy(img, dest)?;
        }
    }
    for url in &item.remote_references {
        let dest = parent.join("remote_assets").join(get_remote_filename(url));
        if !dest.exists() {
            match download_remote_asset(url, &dest) {
                Ok(_) => println!("Downloaded remote asset: {url}"),
                Err(e) => eprintln!("Warning: Failed to download '{url}': {e}"),
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Dev server (file watching + live reload)
// ---------------------------------------------------------------------------

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

/// Minimal percent-decoding so URLs with `%20` etc. map to real file names.
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

/// The live-reload polling script injected into served HTML pages.
fn live_reload_script(version: usize) -> String {
    format!(
        r#"<script>
            let lastVersion = {version};
            async function checkVersion() {{
                try {{
                    const res = await fetch('/__rendir_poll');
                    const v = parseInt(await res.text(), 10);
                    if (v !== lastVersion) {{
                        lastVersion = v;
                        location.reload();
                    }}
                }} catch(e) {{}}
            }}
            setInterval(checkVersion, 1000);
            </script>"#
    )
}

fn respond_html(request: tiny_http::Request, html: String) -> Result<()> {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
        .map_err(|_| anyhow!("invalid content-type header"))?;
    request.respond(Response::from_string(html).with_header(header))?;
    Ok(())
}

fn respond_file(request: tiny_http::Request, path: &Path) -> Result<()> {
    let header = Header::from_bytes(&b"Content-Type"[..], content_type(path).as_bytes())
        .map_err(|_| anyhow!("invalid content-type header"))?;
    request.respond(Response::from_file(File::open(path)?).with_header(header))?;
    Ok(())
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

    // Live-reload polling endpoint.
    if trimmed == "__rendir_poll" {
        let resp = Response::from_string(version.load(Ordering::SeqCst).to_string())
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap());
        request.respond(resp)?;
        return Ok(());
    }

    // Resolve the request to a file, allowing directory indexes and
    // extensionless routes (`/intro` -> `/intro.html`).
    let mut file_path = if trimmed.is_empty() {
        root.join("index.html")
    } else {
        root.join(trimmed)
    };
    if file_path.is_dir() {
        file_path = file_path.join("index.html");
    }
    if !file_path.exists() {
        file_path = file_path.with_extension("html");
    }

    if !file_path.exists() {
        let resp = Response::from_string("404 Not Found").with_status_code(404);
        request.respond(resp)?;
        return Ok(());
    }

    if content_type(&file_path).contains("html") {
        let html = live_reload_inject(fs::read_to_string(&file_path)?, version);
        respond_html(request, html)
    } else {
        respond_file(request, &file_path)
    }
}

/// Inject the live-reload script just before `</body>`.
fn live_reload_inject(html: String, version: &Arc<AtomicUsize>) -> String {
    let script = live_reload_script(version.load(Ordering::SeqCst));
    html.replacen("</body>", &format!("{script}\n</body>"), 1)
}

/// Decide whether a filesystem event should trigger a rebuild.
///
/// Ignores pure access events and events whose paths are all inside the output
/// directory (otherwise writing build output would loop forever).
fn event_is_relevant(res: &notify::Result<notify::Event>, output_abs: Option<&Path>) -> bool {
    let Ok(event) = res else {
        return false;
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
    let version = Arc::new(AtomicUsize::new(0));

    // Initial build (don't bail on failure — let the watcher recover).
    if run_build(input, output, template).is_ok() {
        version.fetch_add(1, Ordering::SeqCst);
    }

    // Static file server on a background thread.
    let addr = format!("127.0.0.1:{port}");
    let server = Arc::new(
        Server::http(&addr).map_err(|e| anyhow!("Failed to start server on {addr}: {e}"))?,
    );
    println!("Dev server running at http://{addr}");

    {
        let server = Arc::clone(&server);
        let root = output.to_path_buf();
        let version = Arc::clone(&version);
        std::thread::spawn(move || {
            for request in server.incoming_requests() {
                if let Err(e) = handle_request(request, &root, &version) {
                    eprintln!("Request error: {e}");
                }
            }
        });
    }

    // File watcher.
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;

    watcher.watch(input, RecursiveMode::Recursive)?;
    println!("Watching: {}", input.display());

    if let Some(tmpl) = template
        && tmpl.exists()
    {
        let mode = if tmpl.is_dir() {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        watcher.watch(tmpl, mode)?;
        println!("Watching: {}", tmpl.display());
    }

    let output_abs = output.canonicalize().ok();
    println!("Press Ctrl+C to stop.");

    while let Ok(first) = rx.recv() {
        let mut relevant = event_is_relevant(&first, output_abs.as_deref());

        // Debounce: drain events landing within a short window.
        std::thread::sleep(Duration::from_millis(200));
        while let Ok(res) = rx.try_recv() {
            relevant |= event_is_relevant(&res, output_abs.as_deref());
        }

        if !relevant {
            continue;
        }

        println!("Change detected, rebuilding...");
        match run_build(input, output, template) {
            Ok(_) => {
                version.fetch_add(1, Ordering::SeqCst);
                println!("Rebuild complete.\n");
            }
            Err(e) => eprintln!("Rebuild failed: {e}\n"),
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut registry = ComponentRegistry::new();
    register_builtin_components(&mut registry);

    match &cli.command {
        Commands::Convert {
            input,
            output,
            template,
        } => convert_file(input, output, template, &registry),
        Commands::Build {
            input,
            output,
            template,
        } => run_build(input, output, template),
        Commands::Dev {
            input,
            output,
            template,
            port,
        } => run_dev(input, output, template, *port),
    }
}