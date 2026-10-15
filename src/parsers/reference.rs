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
    // "ΓΕΝΕΣΙΣ 14"
    static ref RE_REF_VERSE: Regex = Regex::new(
        r"^(?i)\s*(?:(\d\s+[\p{L}.]+|[\p{L}.]+|\d+)\s*)\s+(\d+)(?:\s*[:.]\s*(\d+)(?:\s*[-–—]\s*(\d+))?)?\s*$"
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

/// Normalizes accented characters (Latin and Greek diacritics) for fuzzy reference matching.
pub fn strip_accents(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ά' => 'α',
            'έ' => 'ε',
            'ή' => 'η',
            'ί' | 'ϊ' | 'ΐ' => 'ι',
            'ό' => 'ο',
            'ύ' | 'ϋ' | 'ΰ' => 'υ',
            'ώ' => 'ω',
            _ => c,
        })
        .collect()
}

/// Resolves a book name, abbreviation, or numeric ID into a canonical BookNumber and display title.
pub fn resolve_book(name: &str) -> Result<(BookNumber, String)> {
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

    // 3. Extended common aliases (English, Spanish, Greek, common shortcuts)
    let lower = cleaned.to_lowercase().replace('.', "");
    let normalized = lower.replace(' ', "");
    let unaccented = strip_accents(&normalized);

    let matched_num = match unaccented.as_str() {
        // Old Testament
        "ge" | "gn" | "gen" | "genesis" => Some(10),
        "ex" | "exo" | "exod" | "exodus" | "exodo" => Some(20),
        "le" | "lev" | "leviticus" | "levitico" => Some(30),
        "nu" | "num" | "numbers" | "numeros" => Some(40),
        "de" | "deut" | "dt" | "deuteronomy" | "deuteronomio" => Some(50),
        "jos" | "josh" | "joshua" | "josue" => Some(60),
        "jdg" | "judg" | "judges" | "jueces" => Some(70),
        "ru" | "rut" | "ruth" => Some(80),
        "1sam" | "1sa" | "1samuel" => Some(90),
        "2sam" | "2sa" | "2samuel" => Some(100),
        "1ki" | "1kings" | "1re" | "1reyes" => Some(110),
        "2ki" | "2kings" | "2re" | "2reyes" => Some(120),
        "1ch" | "1chr" | "1chron" | "1chronicles" | "1cr" | "1cronicas" => Some(130),
        "2ch" | "2chr" | "2chron" | "2chronicles" | "2cr" | "2cronicas" => Some(140),
        "ezr" | "ezra" | "esdras" => Some(150),
        "ne" | "neh" | "nehemiah" | "nehemias" => Some(160),
        "es" | "est" | "esth" | "esther" | "ester" => Some(190),
        "jb" | "job" => Some(220),
        "ps" | "psa" | "psalm" | "psalms" | "sal" | "salmo" | "salmos" => Some(230),
        "pr" | "pro" | "prov" | "proverbs" | "proverbios" => Some(240),
        "ec" | "ecc" | "eccles" | "ecclesiastes" | "eclesiastes" => Some(250),
        "so" | "song" | "songofsolomon" | "cantares" | "cantar" => Some(260),
        "is" | "isa" | "isaiah" | "isaias" => Some(290),
        "je" | "jer" | "jeremiah" | "jeremias" => Some(300),
        "la" | "lam" | "lamentations" | "lamentaciones" => Some(310),
        "ez" | "eze" | "ezek" | "ezekiel" | "ezequiel" => Some(330),
        "da" | "dan" | "daniel" => Some(340),
        "ho" | "hos" | "hosea" | "oseas" => Some(350),
        "joe" | "joel" => Some(360),
        "am" | "amos" => Some(370),
        "ob" | "oba" | "obad" | "obadiah" | "abdias" => Some(380),
        "jon" | "jonah" | "jonas" => Some(390),
        "mic" | "micah" | "miqueas" => Some(400),
        "na" | "nah" | "nahum" => Some(410),
        "hab" | "habakkuk" | "habacuc" => Some(420),
        "zep" | "zeph" | "zephaniah" | "sof" | "sofonias" => Some(430),
        "hag" | "haggai" | "hageo" => Some(440),
        "zec" | "zech" | "zechariah" | "zac" | "zacarias" => Some(450),
        "mal" | "malachi" | "malaquias" => Some(460),

        // New Testament
        "mt" | "mat" | "matt" | "matthew" | "mateo" => Some(470),
        "mk" | "mr" | "mar" | "mark" | "marcos" => Some(480),
        "lk" | "lu" | "luk" | "luke" | "lucas" => Some(490),
        "jn" | "joh" | "john" | "juan" | "иоанн" => Some(500),
        "ac" | "act" | "acts" | "hechos" => Some(510),
        "ro" | "rom" | "romans" | "romanos" => Some(520),
        "1co" | "1cor" | "1corinthians" | "1corintios" => Some(530),
        "2co" | "2cor" | "2corinthians" | "2corintios" => Some(540),
        "ga" | "gal" | "galatians" | "galatas" => Some(550),
        "eph" | "ephes" | "ephesians" | "efesios" => Some(560),
        "php" | "phil" | "philippians" | "filipenses" => Some(570),
        "col" | "coloss" | "colossians" | "colosenses" => Some(580),
        "1th" | "1thess" | "1thessalonians" | "1tes" | "1tesalonicenses" => Some(590),
        "2th" | "2thess" | "2thessalonians" | "2tes" | "2tesalonicenses" => Some(600),
        "1ti" | "1tim" | "1timothy" | "1timoteo" => Some(610),
        "2ti" | "2tim" | "2timothy" | "2timoteo" => Some(620),
        "tit" | "titus" | "tito" => Some(630),
        "phm" | "philem" | "philemon" | "filemon" => Some(640),
        "heb" | "hebr" | "hebrews" | "hebreos" => Some(650),
        "jas" | "jam" | "james" | "stg" | "santiago" => Some(660),
        "1pe" | "1pet" | "1peter" | "1ped" | "1pedro" => Some(670),
        "2pe" | "2pet" | "2peter" | "2ped" | "2pedro" => Some(680),
        "1jn" | "1john" | "1juan" => Some(690),
        "2jn" | "2john" | "2juan" => Some(700),
        "3jn" | "3john" | "3juan" => Some(710),
        "jud" | "jude" | "jds" | "judas" => Some(720),
        "re" | "rev" | "revelation" | "ap" | "apoc" | "apocalipsis" => Some(730),

        // Greek OT & NT aliases
        "γενεσις" | "γεν" | "γενεση" => Some(10),
        "εξοδος" | "εξ" => Some(20),
        "λευιτικον" | "λευ" => Some(30),
        "αριθμοι" | "αρ" => Some(40),
        "δευτερονομιον" | "δευτ" | "δτ" => Some(50),
        "ιησους" | "ιησουςναυη" => Some(60),
        "κριται" | "κριτ" => Some(70),
        "ρουθ" => Some(80),
        "1βασιλειων" | "1βασ" => Some(90),
        "2βασιλειων" | "2βασ" => Some(100),
        "3βασιλειων" | "3βασ" => Some(110),
        "4βασιλειων" | "4βασ" => Some(120),
        "1παραλειπομενων" | "1παρ" => Some(130),
        "2παραλειπομενων" | "2παρ" => Some(140),
        "εσδρας" => Some(150),
        "νεεμιας" => Some(160),
        "εσθηρ" => Some(190),
        "ιωβ" => Some(220),
        "ψαλμοι" | "ψαλ" => Some(230),
        "παροιμιαι" | "παρ" => Some(240),
        "εκκλησιαστης" | "εκκλ" => Some(250),
        "ασμαασματων" | "ασμα" => Some(260),
        "ησαιας" | "ησ" => Some(290),
        "ιερεμιας" | "ιερ" => Some(300),
        "θρηνοι" => Some(310),
        "ιεζεκιηλ" => Some(330),
        "δανιηλ" | "δαν" => Some(340),
        "ωσηε" => Some(350),
        "ιωηλ" => Some(360),
        "αμως" => Some(370),
        "οβδιου" | "αβδιου" => Some(380),
        "ιωνας" => Some(390),
        "μιχαιας" => Some(400),
        "ναουμ" => Some(410),
        "αββακουμ" => Some(420),
        "σοφονιας" => Some(430),
        "αγγαιος" => Some(440),
        "ζαχαριας" => Some(450),
        "μαλαχιας" => Some(460),
        "ματθαιος" | "ματθ" | "ματ" => Some(470),
        "μαρκος" | "μαρκ" | "μαρ" => Some(480),
        "λουκας" | "λουκ" | "λου" => Some(490),
        "ιωαννης" | "ιωαν" | "ιω" => Some(500),
        "πραξεις" | "πραξ" | "πρ" => Some(510),
        "ρωμαιους" | "ρωμ" | "ρω" => Some(520),
        "1κορινθιους" | "1κορ" | "ακορινθιους" => Some(530),
        "2κορινθιους" | "2κορ" | "βκορινθιους" => Some(540),
        "γαλατας" | "γαλ" => Some(550),
        "εφεσιους" | "εφεσ" => Some(560),
        "φιλιππησιους" | "φιλιπ" => Some(570),
        "κολοσσαεις" | "κολοσ" => Some(580),
        "1θεσσαλονικεις" | "1θεσ" | "αθεσσαλονικεις" => Some(590),
        "2θεσσαλονικεις" | "2θεσ" | "βθεσσαλονικεις" => Some(600),
        "1τιμοθεον" | "1τιμ" | "ατιμοθεον" => Some(610),
        "2τιμοθεον" | "2τιμ" | "βτιμοθεον" => Some(620),
        "τιτον" | "τιτ" => Some(630),
        "φιλημονα" | "φιλ" => Some(640),
        "εβραιους" | "εβρ" => Some(650),
        "ιακωβος" | "ιακ" => Some(660),
        "1πετρου" | "1πετρ" | "1πετ" | "απετρου" => Some(670),
        "2πετρου" | "2πετρ" | "2πετ" | "βπετρου" => Some(680),
        "1ιωαννου" | "1ιωαν" | "1ιω" | "αιωαννου" => Some(690),
        "2ιωαννου" | "2ιωαν" | "2ιω" | "βιωαννου" => Some(700),
        "3ιωαννου" | "3ιωαν" | "3ιω" | "γιωαννου" => Some(710),
        "ιουδα" | "ιουδ" => Some(720),
        "αποκαλυψη" | "αποκαλυψις" | "αποκ" => Some(730),
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
