use crate::types::ContentItem;

/// Default HTML template
const DEFAULT_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{title}}</title>
    <style>
        /* Basic styling */
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 1rem;
        }
        
        /* Component styles */
        .alert {
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
        }
        .alert-info {
            background-color: #e3f2fd;
            border-left: 5px solid #2196F3;
        }
        .alert-warning {
            background-color: #fff9c4;
            border-left: 5px solid #ffc107;
        }
        .alert-error {
            background-color: #ffebee;
            border-left: 5px solid #f44336;
        }
        
        /* Tab styles */
        .tabs-container {
            margin: 20px 0;
        }
        .tabs {
            display: flex;
            border-bottom: 1px solid #ddd;
        }
        .tab-btn {
            padding: 10px 15px;
            border: none;
            background: none;
            cursor: pointer;
        }
        .tab-btn.active {
            border-bottom: 2px solid #2196F3;
        }
        .tab-content {
            display: none;
            padding: 15px 0;
        }
        .tab-content.active {
            display: block;
        }
        
        /* Video container */
        .video-container {
            position: relative;
            padding-bottom: 56.25%;
            height: 0;
            overflow: hidden;
            max-width: 100%;
        }
        .video-container iframe {
            position: absolute;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
        }
    </style>
</head>
<body>
    <main>
        {{content}}
    </main>
</body>
</html>"#;

/// Render a ContentItem as HTML
pub fn render_html(item: &ContentItem) -> String {
    let default_content = String::new();

    let content = item.rendered_content.as_ref().unwrap_or(&default_content);
    let default_title = String::from("Rustpress Page");
    // Get title from metadata or use default
    let title = item.metadata.get("title").unwrap_or(&default_title);

    // Replace template placeholders
    DEFAULT_HTML_TEMPLATE
        .replace("{{title}}", title)
        .replace("{{content}}", content)
}

/// Render a ContentItem as HTML with a custom template
pub fn render_with_template(item: &ContentItem, template: &str) -> String {
    let default_content = String::new();
    let content = item.rendered_content.as_ref().unwrap_or(&default_content);

    // Replace template placeholders - this is very basic for now
    let mut result = template.replace("{{content}}", content);

    // Replace metadata placeholders
    for (key, value) in &item.metadata {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }

    result
}
