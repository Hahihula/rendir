// rustpress-core/src/components/builtins.rs
use super::{Component, ComponentRenderer};

/// Alert component (info/warning/error box)
pub struct AlertComponent;

impl ComponentRenderer for AlertComponent {
    fn render(&self, component: &Component) -> String {
        let default_alert_type = "info".to_string();
        let default_content = String::new();

        let alert_type = component
            .attributes
            .get("type")
            .unwrap_or(&default_alert_type);
        let title = component
            .attributes
            .get("title")
            .map(|t| format!("<strong>{}</strong><br>", t))
            .unwrap_or_default();
        let content = component.content.as_ref().unwrap_or(&default_content);

        format!(
            "<div class=\"alert alert-{}\">{}{}</div>",
            alert_type, title, content
        )
    }
}

/// YouTube embed component
pub struct YouTubeComponent;

impl ComponentRenderer for YouTubeComponent {
    fn render(&self, component: &Component) -> String {
        if let Some(id) = component.attributes.get("id") {
            format!(
                r#"<div class="video-container"><iframe width="560" height="315" src="https://www.youtube.com/embed/{}" frameborder="0" allowfullscreen></iframe></div>"#,
                id
            )
        } else {
            "<!-- YouTube component requires 'id' attribute -->".to_string()
        }
    }
}

/// Tabs component
pub struct TabsComponent;

impl ComponentRenderer for TabsComponent {
    fn render(&self, component: &Component) -> String {
        if let Some(content) = &component.content {
            // Parse tab content sections
            let tab_regex = regex::Regex::new(r"## (.*?)\n([\s\S]*?)(?=## |$)").unwrap();
            let mut tabs_html = String::from("<div class=\"tabs\">");
            let mut tabs_content = String::from("<div class=\"tab-contents\">");

            let mut is_first = true;
            for (i, cap) in tab_regex.captures_iter(content).enumerate() {
                let tab_title = &cap[1];
                let tab_content = &cap[2];

                let active = if is_first { " active" } else { "" };
                tabs_html.push_str(&format!(
                    r#"<button class="tab-btn{}" data-tab="tab-{}">{}</button>"#,
                    active, i, tab_title
                ));

                tabs_content.push_str(&format!(
                    r#"<div id="tab-{}" class="tab-content{}">{}</div>"#,
                    i, active, tab_content
                ));

                is_first = false;
            }

            tabs_html.push_str("</div>");
            tabs_content.push_str("</div>");

            // Add JavaScript for tab switching
            let js = r#"
            <script>
            document.addEventListener('DOMContentLoaded', function() {
                const tabBtns = document.querySelectorAll('.tab-btn');
                tabBtns.forEach(btn => {
                    btn.addEventListener('click', function() {
                        const tabId = this.getAttribute('data-tab');
                        
                        // Remove active class from all buttons and contents
                        document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
                        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
                        
                        // Add active class to current button and content
                        this.classList.add('active');
                        document.getElementById(tabId).classList.add('active');
                    });
                });
            });
            </script>
            "#;

            format!(
                "<div class=\"tabs-container\">{}{}{}</div>",
                tabs_html, tabs_content, js
            )
        } else {
            "<!-- Tabs component requires content -->".to_string()
        }
    }
}

/// Register all builtin components
pub fn register_builtin_components(registry: &mut super::ComponentRegistry) {
    registry.register("Alert", AlertComponent);
    registry.register("YouTube", YouTubeComponent);
    registry.register("Tabs", TabsComponent);
}
