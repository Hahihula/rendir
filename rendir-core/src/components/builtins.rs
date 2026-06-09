// rendir-core/src/components/builtins.rs
use super::{Component, ComponentRenderer};
use crate::markdown::parse_markdown_fragment;

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

        let parsed_content = parse_markdown_fragment(content);

        format!(
            "<div class=\"alert alert-{}\">{}{}</div>",
            alert_type, title, parsed_content
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
            let sections: Vec<&str> = content.split("\n## ").collect();
            let mut tabs_html = String::from("<div class=\"tabs\">");
            let mut tabs_content = String::from("<div class=\"tab-contents\">");

            let mut is_first = true;
            for (i, section) in sections.into_iter().enumerate() {
                if i == 0 && !section.starts_with("## ") {
                    continue;
                }
                let section = section.strip_prefix("## ").unwrap_or(section);
                let (tab_title, tab_body) = section.split_once('\n').unwrap_or((section, ""));
                let tab_body = tab_body.trim_end();

                let parsed_tab_content = parse_markdown_fragment(tab_body);

                let active = if is_first { " active" } else { "" };
                tabs_html.push_str(&format!(
                    r#"<button class="tab-btn{}" data-tab="tab-{}">{}</button>"#,
                    active, i, tab_title
                ));
                tabs_content.push_str(&format!(
                    r#"<div id="tab-{}" class="tab-content{}">{}</div>"#,
                    i, active, parsed_tab_content
                ));

                is_first = false;
            }

            if !is_first {
                tabs_html.push_str("</div>");
                tabs_content.push_str("</div>");
                let js = r#"
<script>
document.addEventListener('DOMContentLoaded', function() {
    const tabBtns = document.querySelectorAll('.tab-btn');
    tabBtns.forEach(btn => {
        btn.addEventListener('click', function() {
            const tabId = this.getAttribute('data-tab');
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            this.classList.add('active');
            document.getElementById(tabId).classList.add('active');
        });
    });
});
</script>
                "#;
                return format!(
                    "<div class=\"tabs-container\">{tabs_html}</div><div class=\"tab-contents\">{tabs_content}</div>{js}</div>",
                    tabs_html = tabs_html,
                    tabs_content = tabs_content,
                    js = js
                );
            }

            "<!-- Tabs component requires content -->".to_string()
        } else {
            "<!-- Tabs component requires content -->".to_string()
        }
    }
}

/// Slide component - wraps a single slide in a slideshow
pub struct SlideComponent;

impl ComponentRenderer for SlideComponent {
    fn render(&self, component: &Component) -> String {
        let title = component
            .attributes
            .get("title")
            .map(|t| t.as_str())
            .unwrap_or("");

        let slide_type = component
            .attributes
            .get("type")
            .map(|t| t.as_str())
            .unwrap_or("full");

        let content = component.content.as_deref().unwrap_or("");

        let parsed_content = parse_markdown_fragment(content);

        // Output HTML div with data attributes for Vue to pick up
        if !title.is_empty() {
            format!(
                r#"<div class="slide slide-{}" data-title="{}" data-type="{}"><div class="slide-content">{}</div></div>"#,
                slide_type,
                escape_html_attr(title),
                slide_type,
                parsed_content
            )
        } else {
            format!(
                r#"<div class="slide slide-{}" data-type="{}"><div class="slide-content">{}</div></div>"#,
                slide_type, slide_type, parsed_content
            )
        }
    }
}

/// Helper to escape HTML attribute values
fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Search UI component
pub struct SearchComponent;

impl ComponentRenderer for SearchComponent {
    fn render(&self, component: &Component) -> String {
        let placeholder = component
            .attributes
            .get("placeholder")
            .map(|p| p.as_str())
            .unwrap_or("Search...");
        let index_id = component
            .attributes
            .get("index")
            .map(|i| i.as_str())
            .unwrap_or("search-index");

        format!(
            r#"<div class="search-component" data-index="{index_id}">
<input type="text" class="search-input" placeholder="{placeholder}" />
<div class="search-results"></div>
</div>"#,
            index_id = index_id,
            placeholder = placeholder
        )
    }
}

/// Register all builtin components
pub fn register_builtin_components(registry: &mut super::ComponentRegistry) {
    registry.register("Alert", AlertComponent);
    registry.register("YouTube", YouTubeComponent);
    registry.register("Tabs", TabsComponent);
    registry.register("Slide", SlideComponent);
    registry.register("Search", SearchComponent);
}
