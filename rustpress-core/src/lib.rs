pub mod components;
pub mod markdown;
pub mod render;
pub mod types;
pub mod vue;

pub use components::ComponentRegistry;
pub use markdown::parse_markdown;
pub use render::{render_html, render_with_template};
pub use vue::{VueComponent, VueRegistry, parse_vue_component};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
