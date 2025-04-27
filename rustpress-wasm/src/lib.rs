use rustpress_core::{parse_markdown, render_html};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Rustpress {
    // Add fields as needed
}

#[wasm_bindgen]
impl Rustpress {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    /// Render markdown to HTML
    #[wasm_bindgen]
    pub fn render_markdown(&self, content: &str) -> String {
        let item = parse_markdown(content);
        render_html(&item)
    }
}
