use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookConfig {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub src: String,
    pub language: Option<String>,
    pub inclusive_language: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RustConfig {
    pub edition: Option<String>,
    pub nightly_config: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookToml {
    pub book: BookConfig,
    pub rust: Option<RustConfig>,
}

impl BookToml {
    pub fn from_str(content: &str) -> Option<Self> {
        toml::from_str(content).ok()
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Self::from_str(&content)
    }
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub title: String,
    pub path: PathBuf,
    pub children: Vec<Chapter>,
    pub level: usize,
}

#[derive(Debug, Clone)]
pub struct Summary {
    pub chapters: Vec<Chapter>,
}

impl Summary {
    pub fn from_str(content: &str) -> Self {
        let mut result = Vec::new();
        parse_chapters_recursive(
            content.lines().collect::<Vec<_>>().as_slice(),
            &mut result,
            1,
        );
        Summary { chapters: result }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Some(Self::from_str(&content))
    }

    pub fn all_chapters(&self) -> Vec<&Chapter> {
        fn collect<'a>(chapters: &'a [Chapter], result: &mut Vec<&'a Chapter>) {
            for ch in chapters {
                result.push(ch);
                collect(&ch.children, result);
            }
        }
        let mut result = Vec::new();
        collect(&self.chapters, &mut result);
        result
    }
}

fn parse_chapters_recursive(lines: &[&str], output: &mut Vec<Chapter>, target_level: usize) {
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        let leading_spaces = line.len() - line.trim_start().len();
        let level_from_indent = if leading_spaces > 0 {
            (leading_spaces / 2) + 1
        } else {
            1
        };

        if level_from_indent < target_level {
            break;
        }
        if level_from_indent > target_level {
            i += 1;
            continue;
        }

        let is_link =
            trimmed.starts_with('-') && trimmed.len() > 2 && trimmed.chars().nth(1) == Some(' ');

        if is_link {
            let link_content = trimmed.trim_start_matches('-').trim_start();

            if let Some(link_end) = link_content.find(']') {
                let title = link_content[1..link_end].to_string();

                let path_start = link_content.find('(');
                let path = if let Some(ps) = path_start {
                    let rest = &link_content[ps + 1..];
                    rest.find(')')
                        .map(|e| link_content[ps + 1..ps + 1 + e].to_string())
                } else {
                    None
                };

                if let Some(path_str) = path {
                    let mut chapter = Chapter {
                        title,
                        path: PathBuf::from(path_str.trim_start_matches("./")),
                        children: Vec::new(),
                        level: target_level,
                    };

                    parse_chapters_recursive(
                        &lines[i + 1..],
                        &mut chapter.children,
                        target_level + 1,
                    );

                    output.push(chapter);
                }
            }
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_book_toml() {
        let content = r#"
[book]
title = "My Book"
author = "Test Author"
description = "A test book"
src = "src"

[rust]
edition = "2021"
"#;
        let config = BookToml::from_str(content).unwrap();
        assert_eq!(config.book.title, Some("My Book".to_string()));
        assert_eq!(config.book.author, Some("Test Author".to_string()));
        assert_eq!(config.book.src, "src");
    }

    #[test]
    fn test_parse_summary() {
        let content = r#"
# Summary

- [Chapter 1](./chapter_1.md)
- [Chapter 2](./chapter_2.md)
  - [Section 2.1](./chapter_2_1.md)
  - [Section 2.2](./chapter_2_2.md)
- [Chapter 3](./chapter_3.md)
"#;
        let summary = Summary::from_str(content);
        assert_eq!(summary.chapters.len(), 3);
        assert_eq!(summary.chapters[0].title, "Chapter 1");
        assert_eq!(summary.chapters[1].children.len(), 2);
    }

    #[test]
    fn test_all_chapters_flat() {
        let content = r#"
# Summary

- [Chapter 1](./chapter_1.md)
- [Chapter 2](./chapter_2.md)
  - [Section 2.1](./chapter_2_1.md)
"#;
        let summary = Summary::from_str(content);
        let all = summary.all_chapters();
        assert_eq!(all.len(), 3);
    }
}
