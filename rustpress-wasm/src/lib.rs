use rustpress_core::components::{ComponentRegistry, builtins::register_builtin_components};
use rustpress_core::{parse_markdown, render_html};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Rustpress {
    component_registry: ComponentRegistry,
}

#[wasm_bindgen]
impl Rustpress {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut registry = ComponentRegistry::new();
        register_builtin_components(&mut registry);
        Self {
            component_registry: registry,
        }
    }

    /// Render markdown to HTML
    #[wasm_bindgen]
    pub fn render_markdown(&self, content: &str) -> String {
        let item = parse_markdown(content, Some(&self.component_registry));
        render_html(&item)
    }

    // More methods will be added later
}
