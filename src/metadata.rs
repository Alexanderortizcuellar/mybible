use crate::types::{BookNumber, DictionaryType, MatchingType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Wrapper around the `info` table in MyBible SQLite databases.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoTable {
    pub raw: HashMap<String, String>,
}

impl InfoTable {
    pub fn new(entries: HashMap<String, String>) -> Self {
        Self { raw: entries }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.raw.get(key).map(|s| s.as_str())
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        match self.get(key) {
            Some(v) => {
                let v = v.trim().to_lowercase();
                v == "true" || v == "1" || v == "yes"
            }
            None => default,
        }
    }

    pub fn get_f32(&self, key: &str, default: f32) -> f32 {
        self.get(key)
            .and_then(|v| v.trim().parse::<f32>().ok())
            .unwrap_or(default)
    }

    pub fn get_u32(&self, key: &str, default: u32) -> u32 {
        self.get(key)
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or(default)
    }

    // Common properties
    pub fn origin(&self) -> Option<&str> {
        self.get("origin")
    }

    pub fn history_of_changes(&self) -> Option<&str> {
        self.get("history_of_changes")
    }

    pub fn language(&self) -> Option<&str> {
        self.get("language")
    }

    pub fn description(&self) -> Option<&str> {
        self.get("description")
    }

    pub fn detailed_info(&self) -> Option<&str> {
        self.get("detailed_info")
    }

    pub fn region(&self) -> Option<&str> {
        self.get("region")
    }

    pub fn russian_numbering(&self) -> bool {
        self.get_bool("russian_numbering", false)
    }

    pub fn hyperlink_languages(&self) -> Vec<&str> {
        self.get("hyperlink_languages")
            .map(|s| {
                s.split('/')
                    .map(|p| p.trim())
                    .filter(|p| !p.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn html_style(&self) -> Option<&str> {
        self.get("html_style")
    }

    // Bible-specific properties
    pub fn chapter_string(&self) -> Option<&str> {
        self.get("chapter_string")
    }

    pub fn chapter_string_ot(&self) -> Option<&str> {
        self.get("chapter_string_ot")
    }

    pub fn chapter_string_nt(&self) -> Option<&str> {
        self.get("chapter_string_nt")
    }

    pub fn chapter_string_ps(&self) -> Option<&str> {
        self.get("chapter_string_ps")
    }

    pub fn chapter_string_for_book(&self, book: BookNumber) -> &str {
        if book.0 == 230 {
            if let Some(s) = self.chapter_string_ps() {
                return s;
            }
        }
        if book.is_new_testament() {
            if let Some(s) = self.chapter_string_nt() {
                return s;
            }
        } else if let Some(s) = self.chapter_string_ot() {
            return s;
        }
        self.chapter_string().unwrap_or("Chapter")
    }

    pub fn format_chapter_title(&self, book: BookNumber, chapter: u32) -> String {
        let pattern = self.chapter_string_for_book(book);
        let ch_str = self.format_digits(chapter);
        if pattern.contains("%s") {
            pattern.replace("%s", &ch_str)
        } else {
            format!("{} {}", pattern, ch_str)
        }
    }

    pub fn introduction_string(&self) -> &str {
        self.get("introduction_string").unwrap_or("Introduction")
    }

    pub fn strong_numbers(&self) -> bool {
        self.get_bool("strong_numbers", false)
    }

    pub fn right_to_left(&self) -> bool {
        self.get_bool("right_to_left", false)
    }

    pub fn right_to_left_for_book(&self, book: BookNumber) -> bool {
        if book.is_new_testament() {
            if let Some(v) = self.get("right_to_left_nt") {
                return v.trim().eq_ignore_ascii_case("true");
            }
        } else if let Some(v) = self.get("right_to_left_ot") {
            return v.trim().eq_ignore_ascii_case("true");
        }
        self.right_to_left()
    }

    pub fn book_list_right_to_left(&self) -> bool {
        if let Some(v) = self.get("book_list_right_to_left") {
            v.trim().eq_ignore_ascii_case("true")
        } else {
            self.right_to_left()
        }
    }

    pub fn digits(&self) -> Option<[char; 10]> {
        let val = self.get("digits0-9")?;
        let chars: Vec<char> = val.chars().collect();
        if chars.len() == 10 {
            let mut arr = ['0'; 10];
            arr.copy_from_slice(&chars[..10]);
            Some(arr)
        } else {
            None
        }
    }

    pub fn format_digits(&self, number: u32) -> String {
        let s = number.to_string();
        if let Some(digits) = self.digits() {
            s.chars()
                .map(|c| {
                    if let Some(d) = c.to_digit(10) {
                        digits[d as usize]
                    } else {
                        c
                    }
                })
                .collect()
        } else {
            s
        }
    }

    pub fn font_scale(&self) -> f32 {
        self.get_f32("font_scale", 1.0)
    }

    pub fn strong_numbers_prefix(&self) -> Option<&str> {
        self.get("strong_numbers_prefix")
    }

    pub fn contains_accents(&self) -> bool {
        self.get_bool("contains_accents", false)
    }

    pub fn add_space_before_footnote_marker(&self) -> bool {
        self.get_bool("add_space_before_footnote_marker", false)
    }

    pub fn associated_theme(&self) -> Option<&str> {
        self.get("associated_theme")
    }

    pub fn morphology_topic_reference(&self, lang: Option<&str>) -> Option<&str> {
        if let Some(l) = lang {
            let key = format!("morphology_topic_reference_{}", l);
            if let Some(val) = self.get(&key) {
                return Some(val);
            }
        }
        self.get("morphology_topic_reference")
    }

    pub fn left_guillemet_on_the_right(&self) -> bool {
        if let Some(v) = self.get("left_guillemet_on_the_right") {
            v.trim().eq_ignore_ascii_case("true")
        } else {
            // Default is true for "de", "sv", "fi", "nb", "da"
            if let Some(lang) = self.language() {
                matches!(
                    lang.to_lowercase().as_str(),
                    "de" | "sv" | "fi" | "nb" | "da"
                )
            } else {
                false
            }
        }
    }

    // Commentaries properties
    pub fn is_footnotes(&self) -> bool {
        self.get_bool("is_footnotes", false)
    }

    // Cross-references / Reading plan language description
    pub fn description_for_lang(&self, lang: &str) -> &str {
        let key = format!("description_{}", lang);
        if let Some(desc) = self.get(&key) {
            return desc;
        }
        self.description().unwrap_or("")
    }

    pub fn requires_reverse_processing(&self) -> bool {
        self.get_bool("requires_reverse_processing", false)
    }

    // Subheadings properties
    pub fn font_size(&self) -> Option<&str> {
        self.get("font_size")
    }

    pub fn font_bold(&self) -> bool {
        self.get_bool("font_bold", false)
    }

    pub fn font_italic(&self) -> bool {
        self.get_bool("font_italic", false)
    }

    pub fn font_color_day(&self) -> Option<&str> {
        self.get("font_color_day")
    }

    pub fn font_color_night(&self) -> Option<&str> {
        self.get("font_color_night")
    }

    // Dictionary properties
    pub fn articles_language(&self) -> &str {
        self.get("articles_language")
            .or_else(|| self.language())
            .unwrap_or("en")
    }

    pub fn is_strong(&self) -> bool {
        self.get_bool("is_strong", true)
    }

    pub fn dictionary_type(&self) -> DictionaryType {
        if let Some(t) = self.get("type") {
            DictionaryType::from_str_info(t)
        } else if self.is_strong() {
            DictionaryType::StrongLexicon
        } else {
            DictionaryType::Explanatory
        }
    }

    pub fn standard_form_matching_type(&self) -> MatchingType {
        if let Some(m) = self.get("standard_form_matching_type") {
            MatchingType::from_str_info(m)
        } else {
            MatchingType::NormalizedForm2
        }
    }

    pub fn cognate_strong_numbers_info(&self) -> Option<&str> {
        self.get("cognate_strong_numbers_info")
    }

    pub fn synonymous_strong_numbers_info(&self) -> Option<&str> {
        self.get("synonymous_strong_numbers_info")
    }

    pub fn informative_references_to_verses(&self) -> bool {
        self.get_bool("informative_references_to_verses", false)
    }

    pub fn default_topic(&self) -> Option<&str> {
        self.get("default_topic")
    }
}
