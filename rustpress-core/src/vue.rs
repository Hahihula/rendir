use std::collections::HashMap;

/// Represents a parsed Vue.js component
pub struct VueComponent {
    pub name: String,
    pub template: String,
    pub script: Option<String>,
    pub style: Option<String>,
}

/// A registry for Vue.js components
pub struct VueRegistry {
    components: HashMap<String, VueComponent>,
}

impl VueRegistry {
    /// Create a new Vue registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a Vue component
    pub fn register(&mut self, component: VueComponent) {
        self.components.insert(component.name.clone(), component);
    }

    /// Generate Vue.js app initialization code
    pub fn generate_init_code(&self) -> String {
        let mut code = String::from(
            "
<script>
// Initialize Vue app
document.addEventListener('DOMContentLoaded', function() {
    const app = Vue.createApp({});
",
        );

        // Register all components
        for (name, component) in &self.components {
            code.push_str(&format!("\n    // Register component: {}\n", name));
            code.push_str(&format!("    app.component('{}', {{\n", name));

            // Add template
            code.push_str(&format!("        template: `{}`", component.template));

            // Add script if available
            if let Some(script) = &component.script {
                code.push_str(",\n");
                code.push_str(script);
            } else {
                code.push_str("\n");
            }

            code.push_str("    });\n");
        }

        // Mount the app
        code.push_str(
            "\n    app.mount('#app');\n});
</script>\n",
        );

        // Add styles
        for (_, component) in &self.components {
            if let Some(style) = &component.style {
                code.push_str(&format!("<style>\n{}\n</style>\n", style));
            }
        }

        code
    }
}

impl Default for VueRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a simple Vue.js component file
pub fn parse_vue_component(content: &str, name: &str) -> Result<VueComponent, &'static str> {
    let mut template = None;
    let mut script = None;
    let mut style = None;

    // Simple regex-based parsing for demonstration
    // In a real implementation, you'd want to use a proper parser

    // Find <template> section
    if let Some(template_start) = content.find("<template>") {
        if let Some(template_end) = content.find("</template>") {
            template = Some(
                content[template_start + 10..template_end]
                    .trim()
                    .to_string(),
            );
        }
    }

    // Find <script> section
    if let Some(script_start) = content.find("<script>") {
        if let Some(script_end) = content.find("</script>") {
            script = Some(content[script_start + 8..script_end].trim().to_string());
        }
    }

    // Find <style> section
    if let Some(style_start) = content.find("<style>") {
        if let Some(style_end) = content.find("</style>") {
            style = Some(content[style_start + 7..style_end].trim().to_string());
        }
    }

    if let Some(template_str) = template {
        Ok(VueComponent {
            name: name.to_string(),
            template: template_str,
            script,
            style,
        })
    } else {
        Err("No template found in Vue component")
    }
}
