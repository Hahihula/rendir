use std::collections::HashMap;

pub mod builtins;

/// Represents a custom component in markdown
pub struct Component {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub content: Option<String>,
}

/// Registry of available components
pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn ComponentRenderer>>,
}

/// Trait for component renderers
pub trait ComponentRenderer: Send + Sync {
    fn render(&self, component: &Component) -> String;
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a component renderer
    pub fn register<R>(&mut self, name: &str, renderer: R)
    where
        R: ComponentRenderer + 'static,
    {
        self.components.insert(name.to_string(), Box::new(renderer));
    }

    /// Check if a component is registered (case-insensitive)
    pub fn has_component(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.components
            .keys()
            .any(|k| k.to_lowercase() == name_lower)
    }

    /// Returns an iterator over all registered component names
    pub fn component_names(&self) -> impl Iterator<Item = &str> {
        self.components.keys().map(|s| s.as_str())
    }

    /// Render a component (case-insensitive lookup)
    pub fn render(&self, component: &Component) -> String {
        let name_lower = component.name.to_lowercase();
        if let Some(renderer) = self.components.get(&component.name) {
            renderer.render(component)
        } else if let Some((_, renderer)) = self
            .components
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
        {
            renderer.render(component)
        } else {
            format!("<!-- Unknown component: {} -->", component.name)
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
