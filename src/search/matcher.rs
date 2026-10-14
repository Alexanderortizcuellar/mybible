use crate::lookup::normalizer::is_diacritic_mark;
use crate::search::types::{MatchSpan, SearchMode, SearchQuery};
use unicode_normalization::UnicodeNormalization;

/// Represents normalized text alongside a mapping from normalized byte indices back
/// to original byte indices in the input text.
pub struct NormalizedText {
    pub text: String,
    pub byte_map: Vec<usize>,
    pub raw_len: usize,
}

impl NormalizedText {
    pub fn new(raw: &str, case_sensitive: bool, ignore_accents: bool) -> Self {
        let mut text = String::with_capacity(raw.len());
        let mut byte_map = Vec::with_capacity(raw.len() + 1);

        for (byte_start, c) in raw.char_indices() {
            if ignore_accents {
                for decomp_c in c.nfd() {
                    if is_diacritic_mark(decomp_c) {
                        continue;
                    }
                    if !case_sensitive {
                        for lowered in decomp_c.to_lowercase() {
                            let mut buf = [0u8; 4];
                            let encoded = lowered.encode_utf8(&mut buf);
                            for _ in 0..encoded.len() {
                                byte_map.push(byte_start);
                            }
                            text.push_str(encoded);
                        }
                    } else {
                        let mut buf = [0u8; 4];
                        let encoded = decomp_c.encode_utf8(&mut buf);
                        for _ in 0..encoded.len() {
                            byte_map.push(byte_start);
                        }
                        text.push_str(encoded);
                    }
                }
            } else if !case_sensitive {
                for lowered in c.to_lowercase() {
                    let mut buf = [0u8; 4];
                    let encoded = lowered.encode_utf8(&mut buf);
                    for _ in 0..encoded.len() {
                        byte_map.push(byte_start);
                    }
                    text.push_str(encoded);
                }
            } else {
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                for _ in 0..encoded.len() {
                    byte_map.push(byte_start);
                }
                text.push_str(encoded);
            }
        }

        byte_map.push(raw.len());

        Self {
            text,
            byte_map,
            raw_len: raw.len(),
        }
    }

    /// Maps normalized byte range [norm_start, norm_end) back to original byte range [start, end).
    pub fn map_span(&self, norm_start: usize, norm_end: usize) -> Option<MatchSpan> {
        if norm_start >= norm_end || norm_end > self.text.len() {
            return None;
        }

        let orig_start = self.byte_map[norm_start];
        let mut orig_end = self.byte_map[norm_end];

        if orig_start >= orig_end {
            for &b in &self.byte_map[norm_end..] {
                if b > orig_start {
                    orig_end = b;
                    break;
                }
            }
        }

        if orig_start >= orig_end || orig_end > self.raw_len {
            return None;
        }

        Some(MatchSpan {
            start: orig_start,
            end: orig_end,
        })
    }
}

/// Normalizes a query string using the same accent and case options.
pub fn normalize_query_str(query: &str, case_sensitive: bool, ignore_accents: bool) -> String {
    let norm = NormalizedText::new(query, case_sensitive, ignore_accents);
    norm.text
}

/// Extracts clean tokens from a query string.
pub fn extract_query_tokens(
    query: &str,
    case_sensitive: bool,
    ignore_accents: bool,
) -> Vec<String> {
    let norm = normalize_query_str(query, case_sensitive, ignore_accents);
    norm.split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-'))
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// Tests whether a verse matches the given search query.
///
/// Returns `Some(matches)` containing byte ranges in `plain_text` if matched,
/// or `None` if the verse does not match.
pub fn search_verse_text(
    raw_text: &str,
    plain_text: &str,
    query: &SearchQuery,
) -> Option<Vec<MatchSpan>> {
    let trimmed_query = query.text.trim();
    if trimmed_query.is_empty() {
        return None;
    }

    let norm_verse = NormalizedText::new(plain_text, query.case_sensitive, query.ignore_accents);

    match query.mode {
        SearchMode::Phrase => {
            let norm_query =
                normalize_query_str(trimmed_query, query.case_sensitive, query.ignore_accents);
            if norm_query.is_empty() {
                return None;
            }

            let mut matches = Vec::new();
            let mut search_from = 0;
            while let Some(pos) = norm_verse.text[search_from..].find(&norm_query) {
                let norm_start = search_from + pos;
                let norm_end = norm_start + norm_query.len();
                if let Some(span) = norm_verse.map_span(norm_start, norm_end) {
                    matches.push(span);
                }
                search_from = norm_end;
                if norm_query.is_empty() {
                    break;
                }
            }

            if !matches.is_empty() {
                Some(matches)
            } else if check_strongs_fallback(raw_text, trimmed_query) {
                Some(Vec::new())
            } else {
                None
            }
        }
        SearchMode::AllWords => {
            let tokens =
                extract_query_tokens(trimmed_query, query.case_sensitive, query.ignore_accents);
            if tokens.is_empty() {
                return None;
            }

            let mut all_spans = Vec::new();
            for token in &tokens {
                let mut token_found = false;
                let mut search_from = 0;
                while let Some(pos) = norm_verse.text[search_from..].find(token) {
                    let norm_start = search_from + pos;
                    let norm_end = norm_start + token.len();
                    if let Some(span) = norm_verse.map_span(norm_start, norm_end) {
                        all_spans.push(span);
                    }
                    token_found = true;
                    search_from = norm_end;
                    if token.is_empty() {
                        break;
                    }
                }

                if !token_found {
                    if check_strongs_fallback(raw_text, trimmed_query) {
                        return Some(Vec::new());
                    }
                    return None;
                }
            }

            Some(all_spans)
        }
        SearchMode::AnyWord => {
            let tokens =
                extract_query_tokens(trimmed_query, query.case_sensitive, query.ignore_accents);
            if tokens.is_empty() {
                return None;
            }

            let mut any_spans = Vec::new();
            for token in &tokens {
                let mut search_from = 0;
                while let Some(pos) = norm_verse.text[search_from..].find(token) {
                    let norm_start = search_from + pos;
                    let norm_end = norm_start + token.len();
                    if let Some(span) = norm_verse.map_span(norm_start, norm_end) {
                        any_spans.push(span);
                    }
                    search_from = norm_end;
                    if token.is_empty() {
                        break;
                    }
                }
            }

            if !any_spans.is_empty() {
                Some(any_spans)
            } else if check_strongs_fallback(raw_text, trimmed_query) {
                Some(Vec::new())
            } else {
                None
            }
        }
    }
}

/// Fallback check for Strong's number queries embedded in raw tags.
fn check_strongs_fallback(raw_text: &str, query: &str) -> bool {
    let clean = query.trim().trim_start_matches(['G', 'g', 'H', 'h']);
    if !clean.is_empty() && clean.chars().all(|c| c.is_ascii_digit()) {
        let pattern1 = format!("<S>{}</S>", clean);
        let pattern2 = format!("<S>{}</S>", query.trim());
        let pattern3 = format!("<S>G{}</S>", clean);
        let pattern4 = format!("<S>H{}</S>", clean);
        if raw_text.contains(&pattern1)
            || raw_text.contains(&pattern2)
            || raw_text.contains(&pattern3)
            || raw_text.contains(&pattern4)
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accent_insensitive_matching() {
        let plain = "Porque de tal manera amó Dios al mundo que ha dado a su Hijo";
        let query = SearchQuery::new("amo dios");
        let matches = search_verse_text("", plain, &query);
        assert!(matches.is_some());
        let spans = matches.unwrap();
        assert!(!spans.is_empty());

        let hit = crate::search::types::SearchHit {
            bible_module: "TEST".to_string(),
            book_number: crate::types::BookNumber(430),
            book_name: "John".to_string(),
            chapter: 3,
            verse: 16,
            text: plain.to_string(),
            plain_text: plain.to_string(),
            matches: spans,
        };
        let highlighted = hit.render_highlighted(false);
        assert!(highlighted.contains("[amó]"));
        assert!(highlighted.contains("[Dios]"));
    }

    #[test]
    fn test_phrase_matching() {
        let plain = "In the beginning was the Word, and the Word was with God";
        let query = SearchQuery::new("\"Word was with God\"");
        let matches = search_verse_text("", plain, &query);
        assert!(matches.is_some());
        let spans = matches.unwrap();
        assert_eq!(spans.len(), 1);
        assert_eq!(&plain[spans[0].start..spans[0].end], "Word was with God");
    }
}
