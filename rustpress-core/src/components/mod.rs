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

    /// Check if a component is registered
    pub fn has_component(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    /// Render a component
    pub fn render(&self, component: &Component) -> String {
        if let Some(renderer) = self.components.get(&component.name) {
            renderer.render(component)
        } else {
            format!("<!-- Unknown component: {} -->", component.name)
        }
    }
}

/// Default implementation
impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
