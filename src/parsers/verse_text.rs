use serde::{Deserialize, Serialize};

/// AST node representing an element within a Bible verse's text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerseNode {
    /// Regular text content.
    Text(String),
    /// Strong's number reference without prefix, e.g. "7225" from `<S>7225</S>`.
    StrongNumber(String),
    /// Morphology indication for preceding Strong's number from `<m>...</m>`.
    Morphology(String),
    /// Inserted/translator words from `<i>...</i>`.
    TranslatorAddition(Vec<VerseNode>),
    /// Words of Jesus / words of God from `<J>...</J>`.
    JesusWords(Vec<VerseNode>),
    /// Translator or study note from `<n>...</n>`.
    Note(String),
    /// Emphasized text from `<e>...</e>`.
    Emphasis(Vec<VerseNode>),
    /// Poetic indent / OT quotation from `<t>...</t>`.
    PoeticIndent(Vec<VerseNode>),
    /// Line break `<br/>`.
    LineBreak,
    /// Paragraph break `<pb/>`.
    ParagraphBreak,
    /// Footnote marker link from `<f>...</f>`, e.g. `[1]`.
    Footnote(String),
    /// Embedded subheading from `<h>...</h>`.
    EmbeddedSubheading(String),
}

/// Options configuring how verse AST nodes are rendered into HTML or plain text.
#[derive(Debug, Clone)]
pub struct VerseRenderOptions {
    pub show_strong_numbers: bool,
    pub strong_prefix: Option<String>,
    pub show_morphology: bool,
    pub show_notes: bool,
    pub show_footnotes: bool,
    pub show_subheadings: bool,
    pub red_letter_jesus_words: bool,
}

impl Default for VerseRenderOptions {
    fn default() -> Self {
        Self {
            show_strong_numbers: true,
            strong_prefix: None,
            show_morphology: true,
            show_notes: true,
            show_footnotes: true,
            show_subheadings: true,
            red_letter_jesus_words: true,
        }
    }
}

/// Parses the MyBible tagged verse text into a hierarchical AST (`Vec<VerseNode>`).
pub fn parse_verse_text(input: &str) -> Vec<VerseNode> {
    let mut parser = VerseParser::new(input);
    parser.parse_nodes(None)
}

struct VerseParser<'a> {
    chars: &'a str,
    pos: usize,
}

impl<'a> VerseParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input,
            pos: 0,
        }
    }

    fn remaining(&self) -> &'a str {
        &self.chars[self.pos..]
    }

    fn parse_nodes(&mut self, closing_tag: Option<&str>) -> Vec<VerseNode> {
        let mut nodes = Vec::new();
        let mut text_buf = String::new();

        while !self.remaining().is_empty() {
            let rem = self.remaining();

            // Check if we hit the expected closing tag
            if let Some(close) = closing_tag {
                if rem.starts_with(close) {
                    self.pos += close.len();
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(text_buf));
                    }
                    return nodes;
                }
            }

            // Check for tags
            if rem.starts_with('<') {
                if rem.starts_with("<br/>") || rem.starts_with("<br />") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.advance_past('>');
                    nodes.push(VerseNode::LineBreak);
                    continue;
                }
                if rem.starts_with("<pb/>") || rem.starts_with("<pb />") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.advance_past('>');
                    nodes.push(VerseNode::ParagraphBreak);
                    continue;
                }
                if rem.starts_with("<S>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner = self.read_until("</S>");
                    nodes.push(VerseNode::StrongNumber(inner.trim().to_string()));
                    continue;
                }
                if rem.starts_with("<m>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner = self.read_until("</m>");
                    nodes.push(VerseNode::Morphology(inner.trim().to_string()));
                    continue;
                }
                if rem.starts_with("<i>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner_nodes = self.parse_nodes(Some("</i>"));
                    nodes.push(VerseNode::TranslatorAddition(inner_nodes));
                    continue;
                }
                if rem.starts_with("<J>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner_nodes = self.parse_nodes(Some("</J>"));
                    nodes.push(VerseNode::JesusWords(inner_nodes));
                    continue;
                }
                if rem.starts_with("<n>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner = self.read_until("</n>");
                    nodes.push(VerseNode::Note(inner.to_string()));
                    continue;
                }
                if rem.starts_with("<e>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner_nodes = self.parse_nodes(Some("</e>"));
                    nodes.push(VerseNode::Emphasis(inner_nodes));
                    continue;
                }
                if rem.starts_with("<t>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner_nodes = self.parse_nodes(Some("</t>"));
                    nodes.push(VerseNode::PoeticIndent(inner_nodes));
                    continue;
                }
                if rem.starts_with("<f>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner = self.read_until("</f>");
                    nodes.push(VerseNode::Footnote(inner.to_string()));
                    continue;
                }
                if rem.starts_with("<h>") {
                    if !text_buf.is_empty() {
                        nodes.push(VerseNode::Text(std::mem::take(&mut text_buf)));
                    }
                    self.pos += 3;
                    let inner = self.read_until("</h>");
                    nodes.push(VerseNode::EmbeddedSubheading(inner.to_string()));
                    continue;
                }
            }

            // Regular character
            if let Some(ch) = rem.chars().next() {
                text_buf.push(ch);
                self.pos += ch.len_utf8();
            }
        }

        if !text_buf.is_empty() {
            nodes.push(VerseNode::Text(text_buf));
        }

        nodes
    }

    fn read_until(&mut self, closing: &str) -> &'a str {
        let rem = self.remaining();
        if let Some(idx) = rem.find(closing) {
            let res = &rem[..idx];
            self.pos += idx + closing.len();
            res
        } else {
            // Malformed, consume rest
            let res = rem;
            self.pos += rem.len();
            res
        }
    }

    fn advance_past(&mut self, ch: char) {
        let rem = self.remaining();
        if let Some(idx) = rem.find(ch) {
            self.pos += idx + ch.len_utf8();
        } else {
            self.pos += rem.len();
        }
    }
}

/// Convert nodes to clean, unformatted plain text.
pub fn nodes_to_plain_text(nodes: &[VerseNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            VerseNode::Text(s) => out.push_str(s),
            VerseNode::TranslatorAddition(children)
            | VerseNode::JesusWords(children)
            | VerseNode::Emphasis(children)
            | VerseNode::PoeticIndent(children) => {
                out.push_str(&nodes_to_plain_text(children));
            }
            VerseNode::LineBreak => out.push('\n'),
            VerseNode::ParagraphBreak => out.push_str("\n\n"),
            VerseNode::EmbeddedSubheading(h) => {
                out.push('\n');
                out.push_str(h);
                out.push('\n');
            }
            VerseNode::StrongNumber(_)
            | VerseNode::Morphology(_)
            | VerseNode::Note(_)
            | VerseNode::Footnote(_) => {}
        }
    }
    out
}

/// Render the parsed verse AST into HTML.
pub fn nodes_to_html(nodes: &[VerseNode], options: &VerseRenderOptions) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            VerseNode::Text(s) => out.push_str(s),
            VerseNode::StrongNumber(s) => {
                if options.show_strong_numbers {
                    let prefix = options.strong_prefix.as_deref().unwrap_or("");
                    out.push_str(&format!(
                        "<a class=\"strong\" href=\"S:{}{}\"><sup>{}</sup></a>",
                        prefix, s, s
                    ));
                }
            }
            VerseNode::Morphology(m) => {
                if options.show_morphology {
                    out.push_str(&format!("<span class=\"morphology\">&nbsp;{}</span>", m));
                }
            }
            VerseNode::TranslatorAddition(children) => {
                out.push_str("<i>");
                out.push_str(&nodes_to_html(children, options));
                out.push_str("</i>");
            }
            VerseNode::JesusWords(children) => {
                if options.red_letter_jesus_words {
                    out.push_str("<span class=\"jesus_words\">");
                }
                out.push_str(&nodes_to_html(children, options));
                if options.red_letter_jesus_words {
                    out.push_str("</span>");
                }
            }
            VerseNode::Note(n) => {
                if options.show_notes {
                    out.push_str(&format!("<span class=\"note\">({})</span>", n));
                }
            }
            VerseNode::Emphasis(children) => {
                out.push_str("<em>");
                out.push_str(&nodes_to_html(children, options));
                out.push_str("</em>");
            }
            VerseNode::PoeticIndent(children) => {
                out.push_str("<div class=\"poetic_indent\">");
                out.push_str(&nodes_to_html(children, options));
                out.push_str("</div>");
            }
            VerseNode::LineBreak => out.push_str("<br/>"),
            VerseNode::ParagraphBreak => out.push_str("<p class=\"paragraph_break\"/>"),
            VerseNode::Footnote(marker) => {
                if options.show_footnotes {
                    out.push_str(&format!(
                        "<a class=\"footnote\" href=\"#footnote\"><sup>{}</sup></a>",
                        marker
                    ));
                }
            }
            VerseNode::EmbeddedSubheading(h) => {
                if options.show_subheadings {
                    out.push_str(&format!("<div class=\"subheading\">{}</div>", h));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_verse() {
        let text = "In the beginning<S>7225</S> God<S>430</S> created<S>1254</S>";
        let nodes = parse_verse_text(text);
        assert_eq!(nodes.len(), 6);
        assert_eq!(nodes[0], VerseNode::Text("In the beginning".to_string()));
        assert_eq!(nodes[1], VerseNode::StrongNumber("7225".to_string()));
        assert_eq!(nodes[2], VerseNode::Text(" God".to_string()));
        assert_eq!(nodes[3], VerseNode::StrongNumber("430".to_string()));
        assert_eq!(nodes[4], VerseNode::Text(" created".to_string()));
        assert_eq!(nodes[5], VerseNode::StrongNumber("1254".to_string()));

        let plain = nodes_to_plain_text(&nodes);
        assert_eq!(plain, "In the beginning God created");
    }

    #[test]
    fn test_parse_nested_tags() {
        let text = "Jesus said, <J>Thou <J>shalt</J> love<S>25</S></J> <i>the</i> Lord";
        let nodes = parse_verse_text(text);
        let plain = nodes_to_plain_text(&nodes);
        assert_eq!(plain, "Jesus said, Thou shalt love the Lord");
    }

    #[test]
    fn test_footnote_and_morphology() {
        let text = "и<S>2532</S><m>CONJ</m> дух <f>[1]</f> Божий<pb/>";
        let nodes = parse_verse_text(text);
        assert_eq!(nodes[0], VerseNode::Text("и".to_string()));
        assert_eq!(nodes[1], VerseNode::StrongNumber("2532".to_string()));
        assert_eq!(nodes[2], VerseNode::Morphology("CONJ".to_string()));
        assert_eq!(nodes[3], VerseNode::Text(" дух ".to_string()));
        assert_eq!(nodes[4], VerseNode::Footnote("[1]".to_string()));
        assert_eq!(nodes[5], VerseNode::Text(" Божий".to_string()));
        assert_eq!(nodes[6], VerseNode::ParagraphBreak);
    }
}
