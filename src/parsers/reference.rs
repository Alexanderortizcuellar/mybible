use crate::books::{find_book_by_name, find_book_by_number};
use crate::error::{MyBibleError, Result};
use crate::types::BookNumber;
use regex::Regex;

lazy_static::lazy_static! {
    // Regex matching patterns like:
    // "Gen 1:1"
    // "Genesis 1:1-5"
    // "John 3:16"
    // "Psalm 23"
    // "1 Cor 13:4-8"
    // "10 1:1"
    static ref RE_REF_VERSE: Regex = Regex::new(
        r"^(?i)\s*(?:(\d\s+[a-zA-ZáéíóúÁÉÍÓÚа-яА-ЯёЁ]+|[a-zA-ZáéíóúÁÉÍÓÚа-яА-ЯёЁ]+|\d+)\s*)\s+(\d+)(?:\s*[:.]\s*(\d+)(?:\s*[-–—]\s*(\d+))?)?\s*$"
    ).unwrap();
}

/// A parsed Bible passage reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedReference {
    pub book: BookNumber,
    pub book_name: String,
    pub chapter: u32,
    pub verse_start: Option<u32>,
    pub verse_end: Option<u32>,
}

impl ParsedReference {
    /// Formats the reference back to a human-readable display string (e.g. "Genesis 1:1-5" or "John 3:16").
    pub fn display_string(&self) -> String {
        match (self.verse_start, self.verse_end) {
            (Some(start), Some(end)) if start == end => {
                format!("{} {}:{}", self.book_name, self.chapter, start)
            }
            (Some(start), Some(end)) => {
                format!("{} {}:{}-{}", self.book_name, self.chapter, start, end)
            }
            (Some(start), None) => {
                format!("{} {}:{}", self.book_name, self.chapter, start)
            }
            (None, _) => {
                format!("{} {}", self.book_name, self.chapter)
            }
        }
    }
}

/// Parses a user-input reference string into a structured `ParsedReference`.
///
/// Supported formats:
/// - `"John 3:16"` -> John ch 3, v 16
/// - `"Gen 1:1-10"` -> Genesis ch 1, v 1..10
/// - `"Psalm 23"` -> Psalm ch 23 (whole chapter)
/// - `"1 Cor 13:4-7"` -> 1 Corinthians ch 13, v 4..7
/// - `"10 1:1"` -> Book 10 (Genesis) ch 1, v 1
/// - Spanish: `"Juan 3:16"`, `"Génesis 1:1"`, `"Salmos 23"`
/// - Russian: `"Бытие 1:1"`, `"Иоанн 3:16"`
pub fn parse_passage_reference(input: &str) -> Result<ParsedReference> {
    let trimmed = input.trim();

    if let Some(caps) = RE_REF_VERSE.captures(trimmed) {
        let book_str = caps.get(1).map_or("", |m| m.as_str().trim());
        let ch_str = caps.get(2).map_or("", |m| m.as_str());
        let v_start_str = caps.get(3).map(|m| m.as_str());
        let v_end_str = caps.get(4).map(|m| m.as_str());

        let chapter: u32 = ch_str.parse().map_err(|_| {
            MyBibleError::InvalidFormat(format!("Invalid chapter number: {}", ch_str))
        })?;

        let (book_num, canonical_name) = resolve_book(book_str)?;

        let verse_start = v_start_str.and_then(|s| s.parse::<u32>().ok());
        let verse_end = v_end_str
            .and_then(|s| s.parse::<u32>().ok())
            .or(verse_start);

        return Ok(ParsedReference {
            book: book_num,
            book_name: canonical_name,
            chapter,
            verse_start,
            verse_end,
        });
    }

    Err(MyBibleError::InvalidFormat(format!(
        "Could not parse Bible passage reference: '{}'. Expected format like 'John 3:16', 'Gen 1:1-5', or 'Psalm 23'",
        input
    )))
}

/// Resolves a book name, abbreviation, or numeric ID into a canonical BookNumber and display title.
fn resolve_book(name: &str) -> Result<(BookNumber, String)> {
    let cleaned = name.trim();

    // 1. Check if it's already a numeric ID (e.g. "10", "500")
    if let Ok(num) = cleaned.parse::<u16>() {
        let b = BookNumber(num);
        if let Some(info) = find_book_by_number(b) {
            return Ok((b, info.long_name_eng.to_string()));
        }
    }

    // 2. Direct lookup via standard book table (English & Russian)
    if let Some(info) = find_book_by_name(cleaned) {
        return Ok((info.book_number, info.long_name_eng.to_string()));
    }

    // 3. Extended common aliases (English, Spanish, common shortcuts)
    let lower = cleaned.to_lowercase().replace('.', "");
    let normalized = lower.replace(' ', "");

    let matched_num = match normalized.as_str() {
        // Old Testament
        "ge" | "gn" | "gen" | "genesis" | "génesis" => Some(10),
        "ex" | "exo" | "exod" | "exodus" | "éxodo" | "exodo" => Some(20),
        "le" | "lev" | "leviticus" | "levítico" | "levitico" => Some(30),
        "nu" | "num" | "numbers" | "números" | "numeros" => Some(40),
        "de" | "deut" | "dt" | "deuteronomy" | "deuteronomio" => Some(50),
        "jos" | "josh" | "joshua" | "josué" | "josue" => Some(60),
        "jdg" | "judg" | "judges" | "jueces" => Some(70),
        "ru" | "rut" | "ruth" => Some(80),
        "1sam" | "1sa" | "1samuel" => Some(90),
        "2sam" | "2sa" | "2samuel" => Some(100),
        "1ki" | "1kings" | "1re" | "1reyes" => Some(110),
        "2ki" | "2kings" | "2re" | "2reyes" => Some(120),
        "1ch" | "1chr" | "1chron" | "1chronicles" | "1cr" | "1crónicas" | "1cronicas" => Some(130),
        "2ch" | "2chr" | "2chron" | "2chronicles" | "2cr" | "2crónicas" | "2cronicas" => Some(140),
        "ezr" | "ezra" | "esdras" => Some(150),
        "ne" | "neh" | "nehemiah" | "nehemías" | "nehemias" => Some(160),
        "es" | "est" | "esth" | "esther" | "ester" => Some(190),
        "jb" | "job" => Some(220),
        "ps" | "psa" | "psalm" | "psalms" | "sal" | "salmo" | "salmos" => Some(230),
        "pr" | "pro" | "prov" | "proverbs" | "proverbios" => Some(240),
        "ec" | "ecc" | "eccles" | "ecclesiastes" | "eclesiastés" | "eclesiastes" => Some(250),
        "so" | "song" | "songofsolomon" | "cantares" | "cantar" => Some(260),
        "is" | "isa" | "isaiah" | "isaías" | "isaias" => Some(290),
        "je" | "jer" | "jeremiah" | "jeremías" | "jeremias" => Some(300),
        "la" | "lam" | "lamentations" | "lamentaciones" => Some(310),
        "ez" | "eze" | "ezek" | "ezekiel" | "ezequiel" => Some(330),
        "da" | "dan" | "daniel" => Some(340),
        "ho" | "hos" | "hosea" | "oseas" => Some(350),
        "joe" | "joel" => Some(360),
        "am" | "amos" | "amós" => Some(370),
        "ob" | "oba" | "obad" | "obadiah" | "abdías" | "abdias" => Some(380),
        "jon" | "jonah" | "jonás" | "jonas" => Some(390),
        "mic" | "micah" | "miqueas" => Some(400),
        "na" | "nah" | "nahum" | "nahúm" => Some(410),
        "hab" | "habakkuk" | "habacuc" => Some(420),
        "zep" | "zeph" | "zephaniah" | "sof" | "sofonías" | "sofonias" => Some(430),
        "hag" | "haggai" | "hageo" => Some(440),
        "zec" | "zech" | "zechariah" | "zac" | "zacarías" | "zacarias" => Some(450),
        "mal" | "malachi" | "malaquías" | "malaquias" => Some(460),

        // New Testament
        "mt" | "mat" | "matt" | "matthew" | "mateo" => Some(470),
        "mk" | "mr" | "mar" | "mark" | "marcos" => Some(480),
        "lk" | "lu" | "luk" | "luke" | "lucas" => Some(490),
        "jn" | "joh" | "john" | "juan" | "иоанн" => Some(500),
        "ac" | "act" | "acts" | "hechos" => Some(510),
        "ro" | "rom" | "romans" | "romanos" => Some(520),
        "1co" | "1cor" | "1corinthians" | "1corintios" => Some(530),
        "2co" | "2cor" | "2corinthians" | "2corintios" => Some(540),
        "ga" | "gal" | "galatians" | "gálatas" | "galatas" => Some(550),
        "eph" | "ephes" | "ephesians" | "efesios" => Some(560),
        "php" | "phil" | "philippians" | "filipenses" => Some(570),
        "col" | "coloss" | "colossians" | "colosenses" => Some(580),
        "1th" | "1thess" | "1thessalonians" | "1tes" | "1tesalonicenses" => Some(590),
        "2th" | "2thess" | "2thessalonians" | "2tes" | "2tesalonicenses" => Some(600),
        "1ti" | "1tim" | "1timothy" | "1timoteo" => Some(610),
        "2ti" | "2tim" | "2timothy" | "2timoteo" => Some(620),
        "tit" | "titus" | "tito" => Some(630),
        "phm" | "philem" | "philemon" | "filemón" | "filemon" => Some(640),
        "heb" | "hebr" | "hebrews" | "hebreos" => Some(650),
        "jas" | "jam" | "james" | "stg" | "santiago" => Some(660),
        "1pe" | "1pet" | "1peter" | "1ped" | "1pedro" => Some(670),
        "2pe" | "2pet" | "2peter" | "2ped" | "2pedro" => Some(680),
        "1jn" | "1john" | "1juan" => Some(690),
        "2jn" | "2john" | "2juan" => Some(700),
        "3jn" | "3john" | "3juan" => Some(710),
        "jud" | "jude" | "jds" | "judas" => Some(720),
        "re" | "rev" | "revelation" | "ap" | "apoc" | "apocalipsis" => Some(730),
        _ => None,
    };

    if let Some(num) = matched_num {
        let b = BookNumber(num);
        if let Some(info) = find_book_by_number(b) {
            return Ok((b, info.long_name_eng.to_string()));
        }
    }

    Err(MyBibleError::InvalidFormat(format!(
        "Unknown Bible book: '{}'",
        name
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_references() {
        let p1 = parse_passage_reference("John 3:16").unwrap();
        assert_eq!(p1.book, BookNumber(500));
        assert_eq!(p1.chapter, 3);
        assert_eq!(p1.verse_start, Some(16));
        assert_eq!(p1.verse_end, Some(16));

        let p2 = parse_passage_reference("Gen 1:1-5").unwrap();
        assert_eq!(p2.book, BookNumber(10));
        assert_eq!(p2.chapter, 1);
        assert_eq!(p2.verse_start, Some(1));
        assert_eq!(p2.verse_end, Some(5));

        let p3 = parse_passage_reference("Psalm 23").unwrap();
        assert_eq!(p3.book, BookNumber(230));
        assert_eq!(p3.chapter, 23);
        assert_eq!(p3.verse_start, None);

        let p4 = parse_passage_reference("1 Cor 13:4-8").unwrap();
        assert_eq!(p4.book, BookNumber(530));
        assert_eq!(p4.chapter, 13);
        assert_eq!(p4.verse_start, Some(4));
        assert_eq!(p4.verse_end, Some(8));

        let p5 = parse_passage_reference("Juan 3:16").unwrap();
        assert_eq!(p5.book, BookNumber(500));

        let p6 = parse_passage_reference("10 1:1").unwrap();
        assert_eq!(p6.book, BookNumber(10));
    }
}
