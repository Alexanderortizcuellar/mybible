use crate::types::{BookNumber, VerseRef};
use serde::{Deserialize, Serialize};

/// Target destination of a MyBible hyperlink.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HyperlinkTarget {
    Bible(BibleLocation),
    Dictionary(String),
    Commentary(VerseRef),
    External(String),
}

/// Bible place referenced in a hyperlink.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BibleLocation {
    /// Specific verse: e.g. `B:10 1:1`
    Verse(VerseRef),
    /// Verse range within a single chapter: e.g. `B:10 1:1-5`
    VerseRangeSameChapter {
        book: BookNumber,
        chapter: u32,
        start_verse: u32,
        end_verse: u32,
    },
    /// Verse range across chapters: e.g. `B:10 1:1-2:10`
    VerseRangeMultiChapter {
        book: BookNumber,
        start_chapter: u32,
        start_verse: u32,
        end_chapter: u32,
        end_verse: u32,
    },
    /// Discrete comma-separated verses in a chapter: e.g. `B:10 1:1,3,5`
    VerseList {
        book: BookNumber,
        chapter: u32,
        verses: Vec<u32>,
    },
    /// Entire chapter: e.g. `B:10 1`
    Chapter { book: BookNumber, chapter: u32 },
    /// Chapter range: e.g. `B:10 1-3`
    ChapterRange {
        book: BookNumber,
        start_chapter: u32,
        end_chapter: u32,
    },
}

impl BibleLocation {
    pub fn book_number(&self) -> BookNumber {
        match self {
            BibleLocation::Verse(v) => v.book,
            BibleLocation::VerseRangeSameChapter { book, .. } => *book,
            BibleLocation::VerseRangeMultiChapter { book, .. } => *book,
            BibleLocation::VerseList { book, .. } => *book,
            BibleLocation::Chapter { book, .. } => *book,
            BibleLocation::ChapterRange { book, .. } => *book,
        }
    }

    /// Return the starting verse reference for navigation.
    pub fn primary_verse_ref(&self) -> VerseRef {
        match self {
            BibleLocation::Verse(v) => *v,
            BibleLocation::VerseRangeSameChapter {
                book,
                chapter,
                start_verse,
                ..
            } => VerseRef::new(*book, *chapter, *start_verse),
            BibleLocation::VerseRangeMultiChapter {
                book,
                start_chapter,
                start_verse,
                ..
            } => VerseRef::new(*book, *start_chapter, *start_verse),
            BibleLocation::VerseList {
                book,
                chapter,
                verses,
            } => VerseRef::new(*book, *chapter, *verses.first().unwrap_or(&1)),
            BibleLocation::Chapter { book, chapter } => VerseRef::new(*book, *chapter, 1),
            BibleLocation::ChapterRange {
                book,
                start_chapter,
                ..
            } => VerseRef::new(*book, *start_chapter, 1),
        }
    }
}

/// Parse a MyBible href attribute into a `HyperlinkTarget`.
pub fn parse_hyperlink(href: &str) -> Option<HyperlinkTarget> {
    let href = href.trim();
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(HyperlinkTarget::External(href.to_string()));
    }
    if let Some(topic) = href.strip_prefix("S:") {
        return Some(HyperlinkTarget::Dictionary(topic.trim().to_string()));
    }
    if let Some(target) = href.strip_prefix("C:@") {
        return parse_commentary_target(target).map(HyperlinkTarget::Commentary);
    }
    if let Some(bible_target) = href.strip_prefix("B:") {
        return parse_bible_location(bible_target).map(HyperlinkTarget::Bible);
    }
    None
}

/// Parse a Bible location from a `B:` string or `<x>...</x>` tag content.
pub fn parse_bible_location(input: &str) -> Option<BibleLocation> {
    let input = input.trim();
    let space_idx = input.find(' ')?;
    let book_str = &input[..space_idx];
    let rest = input[space_idx + 1..].trim();

    let book_num: u16 = book_str.parse().ok()?;
    let book = BookNumber(book_num);

    // Case 1: Chapter range, e.g. "1-3" without colon
    if !rest.contains(':') {
        if let Some(dash_idx) = rest.find('-') {
            let start_ch: u32 = rest[..dash_idx].trim().parse().ok()?;
            let end_ch: u32 = rest[dash_idx + 1..].trim().parse().ok()?;
            return Some(BibleLocation::ChapterRange {
                book,
                start_chapter: start_ch,
                end_chapter: end_ch,
            });
        }
        // Single chapter: "1"
        let ch: u32 = rest.parse().ok()?;
        return Some(BibleLocation::Chapter { book, chapter: ch });
    }

    // Has colon: "chapter:verse..."
    let colon_idx = rest.find(':')?;
    let ch_str = &rest[..colon_idx];
    let chapter: u32 = ch_str.trim().parse().ok()?;
    let v_part = &rest[colon_idx + 1..].trim();

    // Comma-separated verses: "1:1,3,5"
    if v_part.contains(',') {
        let mut verses = Vec::new();
        for piece in v_part.split(',') {
            let p = piece.trim();
            if let Ok(v) = p.parse::<u32>() {
                verses.push(v);
            }
        }
        if !verses.is_empty() {
            return Some(BibleLocation::VerseList {
                book,
                chapter,
                verses,
            });
        }
    }

    // Range: "1:1-5" or "1:1-2:10"
    if let Some(dash_idx) = v_part.find('-') {
        let start_v_str = &v_part[..dash_idx].trim();
        let end_part = &v_part[dash_idx + 1..].trim();
        let start_verse: u32 = start_v_str.parse().ok()?;

        if end_part.contains(':') {
            // Multi-chapter: "2:10"
            let sub_colon = end_part.find(':')?;
            let end_ch: u32 = end_part[..sub_colon].trim().parse().ok()?;
            let end_verse: u32 = end_part[sub_colon + 1..].trim().parse().ok()?;
            return Some(BibleLocation::VerseRangeMultiChapter {
                book,
                start_chapter: chapter,
                start_verse,
                end_chapter: end_ch,
                end_verse,
            });
        } else {
            // Same chapter: "5"
            let end_verse: u32 = end_part.parse().ok()?;
            return Some(BibleLocation::VerseRangeSameChapter {
                book,
                chapter,
                start_verse,
                end_verse,
            });
        }
    }

    // Single verse: "1:1"
    let verse: u32 = v_part.parse().ok()?;
    Some(BibleLocation::Verse(VerseRef::new(book, chapter, verse)))
}

fn parse_commentary_target(input: &str) -> Option<VerseRef> {
    let input = input.trim();
    let space_idx = input.find(' ')?;
    let book: u16 = input[..space_idx].trim().parse().ok()?;
    let rest = input[space_idx + 1..].trim();
    let colon_idx = rest.find(':')?;
    let chapter: u32 = rest[..colon_idx].trim().parse().ok()?;
    let verse: u32 = rest[colon_idx + 1..].trim().parse().ok()?;
    Some(VerseRef::new(book, chapter, verse))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperlinks() {
        assert_eq!(
            parse_hyperlink("S:H1254"),
            Some(HyperlinkTarget::Dictionary("H1254".to_string()))
        );
        assert_eq!(
            parse_hyperlink("C:@10 3:8"),
            Some(HyperlinkTarget::Commentary(VerseRef::new(10, 3, 8)))
        );
        assert_eq!(
            parse_hyperlink("https://example.com"),
            Some(HyperlinkTarget::External("https://example.com".to_string()))
        );
    }

    #[test]
    fn test_bible_locations() {
        assert_eq!(
            parse_bible_location("10 1:1"),
            Some(BibleLocation::Verse(VerseRef::new(10, 1, 1)))
        );
        assert_eq!(
            parse_bible_location("10 1:1-5"),
            Some(BibleLocation::VerseRangeSameChapter {
                book: BookNumber(10),
                chapter: 1,
                start_verse: 1,
                end_verse: 5,
            })
        );
        assert_eq!(
            parse_bible_location("500 1:3-2:10"),
            Some(BibleLocation::VerseRangeMultiChapter {
                book: BookNumber(500),
                start_chapter: 1,
                start_verse: 3,
                end_chapter: 2,
                end_verse: 10,
            })
        );
        assert_eq!(
            parse_bible_location("10 1:1,3,7"),
            Some(BibleLocation::VerseList {
                book: BookNumber(10),
                chapter: 1,
                verses: vec![1, 3, 7],
            })
        );
        assert_eq!(
            parse_bible_location("10 5"),
            Some(BibleLocation::Chapter {
                book: BookNumber(10),
                chapter: 5,
            })
        );
        assert_eq!(
            parse_bible_location("10 1-3"),
            Some(BibleLocation::ChapterRange {
                book: BookNumber(10),
                start_chapter: 1,
                end_chapter: 3,
            })
        );
    }
}
