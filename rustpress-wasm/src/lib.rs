// rustpress-wasm/src/lib.rs
use rustpress_core::components::{ComponentRegistry, builtins::register_builtin_components};
use rustpress_core::{parse_markdown, render_html, render_with_template};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Rustpress {
    component_registry: ComponentRegistry,
}

#[wasm_bindgen]
impl Rustpress {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set up panic hook for better error messages
        console_error_panic_hook::set_once();

        // Initialize component registry
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
}
