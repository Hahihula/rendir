use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub author: Option<String>,
    pub pub_date: String,
    pub categories: Vec<String>,
    pub guid: String,
    pub content_html: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RssFeed {
    pub title: String,
    pub description: String,
    pub link: String,
    pub language: String,
    pub items: Vec<RssItem>,
    pub last_build_date: String,
}

impl RssFeed {
    pub fn new(title: &str, description: &str, link: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            link: link.to_string(),
            language: "en".to_string(),
            items: Vec::new(),
            last_build_date: now_rfc2822(),
        }
    }

    pub fn add_item(&mut self, item: RssItem) {
        self.items.push(item);
    }

    pub fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(
            "<rss version=\"2.0\" xmlns:content=\"http://purl.org/rss/1.0/modules/content/\">\n",
        );
        xml.push_str("  <channel>\n");
        append_xml(&mut xml, 2, "title", &self.title);
        append_xml(&mut xml, 2, "link", &self.link);
        append_xml(&mut xml, 2, "description", &self.description);
        append_xml(&mut xml, 2, "language", &self.language);
        append_xml(&mut xml, 2, "lastBuildDate", &self.last_build_date);
        append_xml(&mut xml, 2, "generator", "Rendir");
        for item in &self.items {
            xml.push_str("    <item>\n");
            append_xml(&mut xml, 4, "title", &item.title);
            append_xml(&mut xml, 4, "link", &item.link);
            append_xml(&mut xml, 4, "guid", &item.guid);
            append_xml(&mut xml, 4, "description", &item.description);
            append_xml(&mut xml, 4, "pubDate", &item.pub_date);
            if let Some(ref author) = item.author {
                append_xml(&mut xml, 4, "author", author);
            }
            for cat in &item.categories {
                append_xml(&mut xml, 4, "category", cat);
            }
            if let Some(ref html_content) = item.content_html {
                append_xml_cdata(&mut xml, 4, "content:encoded", html_content);
            }
            xml.push_str("    </item>\n");
        }
        xml.push_str("  </channel>\n");
        xml.push_str("</rss>\n");
        xml
    }

    pub fn write_to_file(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_xml())
    }
}

fn append_xml(xml: &mut String, indent: usize, tag: &str, value: &str) {
    let spaces = " ".repeat(indent);
    xml.push_str(&format!(
        "{}<{}>{}</{}>\n",
        spaces,
        tag,
        xml_escape(value),
        tag
    ));
}

fn append_xml_cdata(xml: &mut String, indent: usize, tag: &str, value: &str) {
    let spaces = " ".repeat(indent);
    xml.push_str(&format!(
        "{}<{}><![CDATA[{}]]></{}>\n",
        spaces, tag, value, tag
    ));
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity_buf = String::new();
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            '&' if !in_tag => {
                in_entity = true;
                entity_buf.clear();
                entity_buf.push('&');
            }
            ';' if in_entity => {
                in_entity = false;
                entity_buf.push(';');
                result.push_str(&decode_html_entity(&entity_buf));
            }
            _ if in_entity => entity_buf.push(c),
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    let mut cleaned = String::with_capacity(result.len());
    let mut prev_was_space = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                cleaned.push(' ');
                prev_was_space = true;
            }
        } else {
            cleaned.push(c);
            prev_was_space = false;
        }
    }
    cleaned.trim().to_string()
}

fn decode_html_entity(entity: &str) -> String {
    match entity {
        "&amp;" => "&".to_string(),
        "&lt;" => "<".to_string(),
        "&gt;" => ">".to_string(),
        "&quot;" => "\"".to_string(),
        "&apos;" => "'".to_string(),
        "&nbsp;" => " ".to_string(),
        _ => {
            if entity.starts_with("&#") && entity.ends_with(';') {
                let num_str = &entity[2..entity.len() - 1];
                if let Ok(code) = num_str.parse::<u32>()
                    && let Some(ch) = char::from_u32(code) {
                        return ch.to_string();
                    }
            }
            entity.to_string()
        }
    }
}

pub fn parse_date_to_rfc2822(date_str: &str) -> String {
    let trimmed = date_str.trim();
    if let Some(rfc) = try_parse_ymd(trimmed) {
        return rfc;
    }
    trimmed.to_string()
}

fn try_parse_ymd(s: &str) -> Option<String> {
    let s = s.split(['T', ' ']).next().unwrap_or(s);
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    Some(format_rfc2822_date(year, month, day))
}

fn format_rfc2822_date(year: i32, month: u32, day: u32) -> String {
    const MONTHS: &[&str] = &[
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const DAYS: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let dow = compute_dow(year, month, day);
    format!(
        "{}, {:02} {} {} 00:00:00 +0000",
        DAYS[dow as usize],
        day,
        MONTHS[month as usize - 1],
        year
    )
}

fn compute_dow(year: i32, month: u32, day: u32) -> u32 {
    static T: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    (y as u32 + y as u32 / 4 - y as u32 / 100 + y as u32 / 400 + T[month as usize - 1] + day) % 7
}

pub fn now_rfc2822() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (year, month, day, hour, min, sec) = secs_to_datetime(ts);
    let dow = compute_dow(year, month, day);
    const DAYS: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: &[&str] = &[
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} +0000",
        DAYS[dow as usize],
        day,
        MONTHS[month as usize - 1],
        year,
        hour,
        min,
        sec
    )
}

fn secs_to_datetime(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let z = secs / 86400 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    let ts = secs % 86400;
    (
        year,
        m as u32,
        d as u32,
        (ts / 3600) as u32,
        ((ts % 3600) / 60) as u32,
        (ts % 60) as u32,
    )
}
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_basic_xml() {
        let mut feed = RssFeed::new("My Blog", "A test blog", "/");
        feed.add_item(RssItem {
            title: "Hello World".to_string(),
            link: "hello.html".to_string(),
            description: "First post".to_string(),
            author: Some("Alice".to_string()),
            pub_date: "Mon, 15 Jan 2024 00:00:00 +0000".to_string(),
            categories: vec!["intro".to_string()],
            guid: "hello.html".to_string(),
            content_html: None,
        });
        let xml = feed.to_xml();
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<title>My Blog</title>"));
        assert!(xml.contains("<item>"));
        assert!(xml.contains("<author>Alice</author>"));
        assert!(xml.contains("<category>intro</category>"));
    }

    #[test]
    fn test_rss_with_content_encoded() {
        let mut feed = RssFeed::new("Blog", "Desc", "/");
        feed.add_item(RssItem {
            title: "Post".to_string(),
            link: "post.html".to_string(),
            description: "Excerpt".to_string(),
            author: None,
            pub_date: "Tue, 01 Feb 2024 00:00:00 +0000".to_string(),
            categories: vec![],
            guid: "post.html".to_string(),
            content_html: Some("<p>Full article</p>".to_string()),
        });
        let xml = feed.to_xml();
        assert!(xml.contains("<content:encoded>"));
        assert!(xml.contains("<![CDATA[<p>Full article</p>]]>"));
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(xml_escape("a & b < c"), "a &amp; b &lt; c");
    }

    #[test]
    fn test_strip_html_basic() {
        assert_eq!(strip_html("<p>Hello <b>world</b>!</p>"), "Hello world!");
    }

    #[test]
    fn test_strip_html_with_entities() {
        assert_eq!(strip_html("<p>Rust &amp; Roll</p>"), "Rust & Roll");
    }

    #[test]
    fn test_strip_html_nbsp() {
        assert_eq!(strip_html("<p>Hello&nbsp;World</p>"), "Hello World");
    }

    #[test]
    fn test_parse_date_ymd() {
        assert_eq!(
            parse_date_to_rfc2822("2024-01-15"),
            "Mon, 15 Jan 2024 00:00:00 +0000"
        );
        assert_eq!(
            parse_date_to_rfc2822("2024-03-01"),
            "Fri, 01 Mar 2024 00:00:00 +0000"
        );
    }

    #[test]
    fn test_parse_date_with_time() {
        assert_eq!(
            parse_date_to_rfc2822("2024-06-15T12:30:00"),
            "Sat, 15 Jun 2024 00:00:00 +0000"
        );
    }

    #[test]
    fn test_empty_feed_xml_validity() {
        let feed = RssFeed::new("Empty", "No items", "/");
        let xml = feed.to_xml();
        assert!(xml.contains("<channel>"));
        assert!(xml.contains("</channel>"));
        assert!(xml.contains("</rss>"));
        assert!(!xml.contains("<item>"));
    }

    #[test]
    fn test_write_to_file() {
        let mut feed = RssFeed::new("Test", "Test feed", "/");
        feed.add_item(RssItem {
            title: "Item".to_string(),
            link: "item.html".to_string(),
            description: "Desc".to_string(),
            author: None,
            pub_date: "Sun, 01 Jan 2024 00:00:00 +0000".to_string(),
            categories: vec![],
            guid: "item.html".to_string(),
            content_html: None,
        });
        let dir = std::env::temp_dir().join("rendir_rss_test");
        let path = dir.join("feed.xml");
        feed.write_to_file(&path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("<rss"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
