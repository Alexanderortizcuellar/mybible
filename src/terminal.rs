use colored::*;
use regex::Regex;

lazy_static::lazy_static! {
    static ref RE_TAGS: Regex = Regex::new(r"</?([a-zA-Z0-9]+)[^>]*>").unwrap();
    static ref RE_LINK: Regex = Regex::new(r#"<a\s+[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#).unwrap();
    static ref RE_JESUS: Regex = Regex::new(r"<J>(.*?)</J>").unwrap();
    static ref RE_FOOTNOTE: Regex = Regex::new(r"<f>(.*?)</f>").unwrap();
    static ref RE_STRONG: Regex = Regex::new(r"<S>(.*?)</S>").unwrap();
    static ref RE_BOLD: Regex = Regex::new(r"<(?:b|strong)>(.*?)</(?:b|strong)>").unwrap();
    static ref RE_ITALIC: Regex = Regex::new(r"<(?:i|em)>(.*?)</(?:i|em)>").unwrap();
    static ref RE_HEADER: Regex = Regex::new(r"<h[1-6][^>]*>(.*?)</h[1-6]>").unwrap();
    static ref RE_XREF_TAG: Regex = Regex::new(r"<x>(.*?)</x>").unwrap();
}

/// Decodes standard HTML and XML entities into Unicode characters.
pub fn decode_html_entities(input: &str) -> String {
    input
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&#8212;", "—")
        .replace("&#8211;", "–")
        .replace("&#8216;", "‘")
        .replace("&#8217;", "’")
        .replace("&#8220;", "“")
        .replace("&#8221;", "”")
        .replace("&#160;", " ")
}

/// Renders HTML or MyBible tagged content into human-friendly terminal text with ANSI colors.
pub fn render_to_terminal(input: &str, use_color: bool) -> String {
    if !use_color {
        return render_plain_text(input);
    }

    let mut text = input.to_string();

    // 1. Line breaks and paragraphs
    text = text.replace("<br/>", "\n");
    text = text.replace("<br>", "\n");
    text = text.replace("<pb/>", "\n\n");
    text = text.replace("<p/>", "\n\n");
    text = text.replace("</p>", "\n\n");
    text = text.replace("</div>", "\n");

    // 2. Headings (h1..h6)
    text = RE_HEADER
        .replace_all(&text, |caps: &regex::Captures| {
            format!("\n{}\n", caps[1].trim().bright_yellow().bold())
        })
        .to_string();

    // 3. Words of Jesus (<J>)
    text = RE_JESUS
        .replace_all(&text, |caps: &regex::Captures| {
            caps[1].bright_red().to_string()
        })
        .to_string();

    // 4. Strong's numbers (<S>)
    text = RE_STRONG
        .replace_all(&text, |caps: &regex::Captures| {
            format!(" {}", format!("<{}>", &caps[1]).dimmed().magenta())
        })
        .to_string();

    // 5. Footnote markers (<f>)
    text = RE_FOOTNOTE
        .replace_all(&text, |caps: &regex::Captures| {
            caps[1].bright_yellow().bold().to_string()
        })
        .to_string();

    // 6. Hyperlinks (<a href="...">...</a>)
    text = RE_LINK
        .replace_all(&text, |caps: &regex::Captures| {
            let href = &caps[1];
            let label = &caps[2];
            if href.starts_with("B:") {
                // Bible reference link in cyan underline
                label.cyan().underline().to_string()
            } else if href.starts_with("S:") {
                // Strong's link in purple
                label.magenta().underline().to_string()
            } else {
                // General link
                label.cyan().underline().to_string()
            }
        })
        .to_string();

    // 7. Bold and Italic
    text = RE_BOLD
        .replace_all(&text, |caps: &regex::Captures| caps[1].bold().to_string())
        .to_string();

    text = RE_ITALIC
        .replace_all(&text, |caps: &regex::Captures| caps[1].italic().to_string())
        .to_string();

    // 8. Strip any remaining HTML tags (span, small, p class="...", etc.)
    text = RE_TAGS.replace_all(&text, "").to_string();

    // 9. Decode HTML entities
    text = decode_html_entities(&text);

    // 10. Clean up excessive consecutive newlines (limit to max 2)
    clean_newlines(&text)
}

/// Renders HTML or MyBible tagged content into plain unformatted text (no ANSI escape codes).
pub fn render_plain_text(input: &str) -> String {
    let mut text = input.to_string();

    text = text.replace("<br/>", "\n");
    text = text.replace("<br>", "\n");
    text = text.replace("<pb/>", "\n\n");
    text = text.replace("<p/>", "\n\n");
    text = text.replace("</p>", "\n\n");
    text = text.replace("</div>", "\n");

    // Strip all tags
    text = RE_TAGS.replace_all(&text, "").to_string();
    text = decode_html_entities(&text);

    clean_newlines(&text)
}

fn clean_newlines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut newline_count = 0;

    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            newline_count += 1;
            if newline_count <= 2 {
                result.push('\n');
            }
        } else {
            newline_count = 0;
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(trimmed);
            result.push('\n');
        }
    }

    result.trim_end().to_string()
}

/// Formats a subheading text, converting MyBible <x>...</x> cross-reference targets into readable citations.
pub fn render_subheading(input: &str, use_color: bool) -> String {
    let mut text = input.to_string();

    text = RE_XREF_TAG
        .replace_all(&text, |caps: &regex::Captures| {
            let inner = caps[1].trim();
            let formatted = format_xref_target(inner);
            if use_color {
                format!(" ({})", formatted.cyan())
            } else {
                format!(" ({})", formatted)
            }
        })
        .to_string();

    // Strip other tags
    text = RE_TAGS.replace_all(&text, "").to_string();
    decode_html_entities(&text).trim().to_string()
}

fn format_xref_target(target: &str) -> String {
    let mut parts = Vec::new();
    for piece in target.split(';') {
        let piece = piece.trim();
        if let Some((book_str, ref_str)) = piece.split_once(' ') {
            if let Ok(bnum) = book_str.parse::<u16>() {
                if let Some(info) = crate::books::find_book_by_number(bnum) {
                    parts.push(format!("{} {}", info.short_name_eng, ref_str.trim()));
                    continue;
                }
            }
        }
        parts.push(piece.to_string());
    }
    parts.join("; ")
}
