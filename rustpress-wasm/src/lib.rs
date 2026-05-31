// rustpress-wasm/src/lib.rs
use rustpress_core::components::{builtins::register_builtin_components, ComponentRegistry};
use rustpress_core::{
    parse_markdown, parse_markdown_with_path, render_html, render_slideshow_vue,
    render_with_template,
};
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Rustpress {
    component_registry: ComponentRegistry,
}

#[wasm_bindgen]
impl Rustpress {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        let mut registry = ComponentRegistry::new();
        register_builtin_components(&mut registry);

        Self {
            component_registry: registry,
        }
    }

    /// Render markdown to HTML with default template
    #[wasm_bindgen]
    pub fn render_markdown(&self, content: &str) -> String {
        let item = parse_markdown(content, Some(&self.component_registry));
        render_html(&item)
    }

    /// Render markdown to HTML with custom template
    #[wasm_bindgen]
    pub fn render_markdown_with_template(
        &self,
        content: &str,
        template: &str,
        template_name: &str,
    ) -> String {
        let item = parse_markdown(content, Some(&self.component_registry));
        render_with_template(&item, template_name, template)
    }

    /// Render markdown as a slideshow Vue SPA (returns full HTML page)
    #[wasm_bindgen]
    pub fn render_slideshow(&self, content: &str) -> String {
        let item = parse_markdown_with_path(
            content,
            Some(&self.component_registry),
            Some(PathBuf::from("presentation.md")),
        );
        render_slideshow_vue(&item)
    }
}
