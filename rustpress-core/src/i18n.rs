use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::types::{Language, Translation};

pub struct I18nBuilder {
    default_language: String,
    languages: Vec<Language>,
    translation_index: HashMap<String, HashMap<String, TranslationEntry>>,
}

struct TranslationEntry {
    full_path: PathBuf,
    rel_path: PathBuf,
    title: String,
    exists: bool,
}

impl I18nBuilder {
    pub fn new(default_language: impl Into<String>) -> Self {
        Self {
            default_language: default_language.into(),
            languages: Vec::new(),
            translation_index: HashMap::new(),
        }
    }

    pub fn with_languages(mut self, languages: Vec<Language>) -> Self {
        self.languages = languages;
        self
    }

    pub fn detect_languages(input_dir: &Path) -> Vec<Language> {
        let mut languages = Vec::new();

        if !input_dir.is_dir() {
            return languages;
        }

        let entries = match std::fs::read_dir(input_dir) {
            Ok(e) => e,
            Err(_) => return languages,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let code = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if !code.starts_with('.') && !code.is_empty() {
                    let has_markdown = std::fs::read_dir(&path)
                        .ok()
                        .map(|entries| {
                            entries.flatten().any(|e| {
                                e.path().extension().map(|ext| ext == "md").unwrap_or(false)
                            })
                        })
                        .unwrap_or(false);

                    if has_markdown {
                        languages.push(Language {
                            code: code.clone(),
                            name: Language::native_name(&code),
                            is_default: code == "en",
                        });
                    }
                }
            }
        }

        if languages.is_empty() {
            languages.push(Language {
                code: "en".to_string(),
                name: "English".to_string(),
                is_default: true,
            });
        }

        for lang in &mut languages {
            if lang.code == "en" {
                lang.is_default = true;
            }
        }

        languages
    }

    pub fn build_index(&mut self, input_dir: &Path) -> Result<(), std::io::Error> {
        let languages = Self::detect_languages(input_dir);
        let lang_codes: Vec<String> = languages.iter().map(|l| l.code.clone()).collect();
        self.languages = languages;

        for code in &lang_codes {
            let lang_dir = input_dir.join(code);
            self.index_language_dir(&lang_dir, code)?;
        }

        Ok(())
    }

    fn index_language_dir(
        &mut self,
        lang_dir: &Path,
        lang_code: &str,
    ) -> Result<(), std::io::Error> {
        if !lang_dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(lang_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                // Get path relative to language dir (e.g., "about.md" from "/tmp/content/en/about.md")
                let rel_path = path.strip_prefix(lang_dir).unwrap_or(path).to_path_buf();

                let content = std::fs::read_to_string(path)?;
                let title = Self::extract_title_from_content(&content).unwrap_or_else(|| {
                    path.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                });

                // Key is path without extension, relative to content root (NOT including lang code)
                let key = rel_path
                    .to_string_lossy()
                    .replace(".md", "")
                    .replace('\\', "/");

                let entry = self.translation_index.entry(key).or_default();
                entry.insert(
                    lang_code.to_string(),
                    TranslationEntry {
                        full_path: path.to_path_buf(),
                        rel_path,
                        title,
                        exists: true,
                    },
                );
            }
        }

        Ok(())
    }

    pub fn get_translations(&self, content_path: &str) -> Vec<Translation> {
        let content_path_normalized = content_path.replace("\\", "/");

        let entries = match self.translation_index.get(&content_path_normalized) {
            Some(e) => e,
            None => return Vec::new(),
        };

        let mut translations = Vec::new();

        for lang in &self.languages {
            let entry = entries.get(&lang.code);

            if let Some(e) = entry {
                let url = format!(
                    "/{}/{}",
                    lang.code,
                    e.rel_path.to_string_lossy().replace(".md", "")
                );
                translations.push(Translation {
                    language: lang.code.clone(),
                    url,
                    title: e.title.clone(),
                    exists: e.exists,
                });
            } else {
                translations.push(Translation {
                    language: lang.code.clone(),
                    url: String::new(),
                    title: String::new(),
                    exists: false,
                });
            }
        }

        translations
    }

    pub fn get_fallback_content(&self, content_path: &str) -> Option<(String, String)> {
        let content_path_normalized = content_path.replace("\\", "/");

        let entries = self.translation_index.get(&content_path_normalized)?;

        if let Some(entry) = entries.get(&self.default_language) {
            let content = std::fs::read_to_string(&entry.full_path).ok()?;
            Some((entry.title.clone(), content))
        } else {
            None
        }
    }

    pub fn languages(&self) -> &[Language] {
        &self.languages
    }

    pub fn default_language(&self) -> &str {
        &self.default_language
    }

    fn extract_title_from_content(content: &str) -> Option<String> {
        // Try to extract title from frontmatter first
        if let Some(first_dash) = content.find("---") {
            let after_first = &content[first_dash + 3..];
            if let Some(second_dash) = after_first.find("---") {
                let frontmatter = &after_first[..second_dash];
                for line in frontmatter.lines() {
                    let line = line.trim();
                    if let Some(title) = line.strip_prefix("title:") {
                        let title = title.trim().trim_matches('"').trim_matches('\'');
                        if !title.is_empty() {
                            return Some(title.to_string());
                        }
                    }
                }
            }
        }

        // Fall back to first heading
        let after_frontmatter = if let Some(first_dash) = content.find("---") {
            let after_first = &content[first_dash + 3..];
            if let Some(second_dash) = after_first.find("---") {
                &after_first[second_dash + 3..]
            } else {
                content
            }
        } else {
            content
        };

        for line in after_frontmatter.lines() {
            let line = line.trim();
            if let Some(title) = line.strip_prefix("# ") {
                return Some(title.trim().to_string());
            }
        }
        None
    }
}

impl Language {
    pub fn native_name(code: &str) -> String {
        match code {
            "en" => "English",
            "de" => "Deutsch",
            "cs" => "Čeština",
            "zh" => "中文",
            "fr" => "Français",
            "es" => "Español",
            "ja" => "日本語",
            "ko" => "한국어",
            "ru" => "Русский",
            "pt" => "Português",
            "it" => "Italiano",
            "pl" => "Polski",
            "nl" => "Nederlands",
            "sv" => "Svenska",
            "da" => "Dansk",
            "fi" => "Suomi",
            "no" => "Norsk",
            "tr" => "Türkçe",
            "el" => "Ελληνικά",
            "he" => "עברית",
            "ar" => "العربية",
            "hi" => "हिन्दी",
            "th" => "ไทย",
            "vi" => "Tiếng Việt",
            "id" => "Bahasa Indonesia",
            "ms" => "Bahasa Melayu",
            "uk" => "Українська",
            "bg" => "Български",
            "hr" => "Hrvatski",
            "sk" => "Slovenčina",
            "sl" => "Slovenščina",
            "ro" => "Română",
            _ => code,
        }
        .to_string()
    }
}

use walkdir::WalkDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_native_names() {
        assert_eq!(Language::native_name("en"), "English");
        assert_eq!(Language::native_name("de"), "Deutsch");
        assert_eq!(Language::native_name("cs"), "Čeština");
        assert_eq!(Language::native_name("zh"), "中文");
    }

    #[test]
    fn test_build_index_and_get_translations() {
        let temp = tempfile::tempdir().unwrap();

        // Create en/ and de/ directories with files
        let en_dir = temp.path().join("en");
        let de_dir = temp.path().join("de");
        std::fs::create_dir(&en_dir).unwrap();
        std::fs::create_dir(&de_dir).unwrap();

        // en/ has index.md and about.md
        std::fs::write(en_dir.join("index.md"), "---\ntitle: Welcome\n---\n# Hello").unwrap();
        std::fs::write(
            en_dir.join("about.md"),
            "---\ntitle: About Us\n---\n# About",
        )
        .unwrap();

        // de/ only has index.md (about.md missing - will fallback)
        std::fs::write(
            de_dir.join("index.md"),
            "---\ntitle: Willkommen\n---\n# Hallo",
        )
        .unwrap();

        // Build the index
        let mut builder = I18nBuilder::new("en");
        builder.build_index(temp.path()).unwrap();

        // Check languages detected
        assert_eq!(builder.languages().len(), 2);

        // Check translations for "index" (key is without extension)
        let translations = builder.get_translations("index");
        assert_eq!(translations.len(), 2);

        // English translation
        let en_trans = translations.iter().find(|t| t.language == "en").unwrap();
        assert!(en_trans.exists);
        assert_eq!(en_trans.title, "Welcome");

        // German translation
        let de_trans = translations.iter().find(|t| t.language == "de").unwrap();
        assert!(de_trans.exists);
        assert_eq!(de_trans.title, "Willkommen");

        // Check translations for "about" (only exists in English)
        let about_translations = builder.get_translations("about");
        assert_eq!(about_translations.len(), 2);

        let en_about = about_translations
            .iter()
            .find(|t| t.language == "en")
            .unwrap();
        assert!(en_about.exists);
        assert_eq!(en_about.title, "About Us");

        let de_about = about_translations
            .iter()
            .find(|t| t.language == "de")
            .unwrap();
        assert!(!de_about.exists); // German doesn't exist, should be marked
    }

    #[test]
    fn test_fallback_content() {
        let temp = tempfile::tempdir().unwrap();

        let en_dir = temp.path().join("en");
        let cs_dir = temp.path().join("cs");
        std::fs::create_dir(&en_dir).unwrap();
        std::fs::create_dir(&cs_dir).unwrap();

        std::fs::write(
            en_dir.join("about.md"),
            "---\ntitle: About\n---\n# About Content",
        )
        .unwrap();
        // cs/ has no about.md - should fallback to en/

        let mut builder = I18nBuilder::new("en");
        builder.build_index(temp.path()).unwrap();

        // about.md doesn't exist in cs/, so fallback should return English content
        // Key is without extension
        let fallback = builder.get_fallback_content("about");
        assert!(fallback.is_some());
        let (title, content) = fallback.unwrap();
        assert_eq!(title, "About");
        assert!(content.contains("# About Content"));
    }

    #[test]
    fn test_no_fallback_for_non_existent_page() {
        let temp = tempfile::tempdir().unwrap();

        let en_dir = temp.path().join("en");
        std::fs::create_dir(&en_dir).unwrap();
        std::fs::write(en_dir.join("index.md"), "# Index").unwrap();

        let mut builder = I18nBuilder::new("en");
        builder.build_index(temp.path()).unwrap();

        // This page doesn't exist at all (key is without extension)
        let fallback = builder.get_fallback_content("nonexistent");
        assert!(fallback.is_none());
    }

    #[test]
    fn test_translations_include_url() {
        let temp = tempfile::tempdir().unwrap();

        let en_dir = temp.path().join("en");
        let fr_dir = temp.path().join("fr");
        std::fs::create_dir(&en_dir).unwrap();
        std::fs::create_dir(&fr_dir).unwrap();

        std::fs::write(en_dir.join("about.md"), "# About").unwrap();
        std::fs::write(fr_dir.join("about.md"), "# À Propos").unwrap();

        let mut builder = I18nBuilder::new("en");
        builder.build_index(temp.path()).unwrap();

        let translations = builder.get_translations("about");

        let en_trans = translations.iter().find(|t| t.language == "en").unwrap();
        assert!(en_trans.url.contains("/en/about"));

        let fr_trans = translations.iter().find(|t| t.language == "fr").unwrap();
        assert!(fr_trans.url.contains("/fr/about"));
    }
}
