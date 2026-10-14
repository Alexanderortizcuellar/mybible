use crate::error::Result;
use crate::lookup::normalizer::{normalize_form_1, normalize_form_2};
use crate::models::dictionary::DictionaryRecord;
use crate::readers::dictionary::DictionaryReader;
use crate::types::{BookNumber, DictionaryType, MatchingType};
use std::sync::Arc;

/// Operational mode when tapping words in the Bible text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LookupMode {
    /// Try finding an article in Strong's lexicon first, then general dictionaries.
    #[default]
    Strong,
    /// Try finding an article in general dictionaries first, then Strong's lexicon.
    Dictionaries,
}

/// A matched dictionary topic from a lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupMatch {
    pub dictionary_name: String,
    pub dictionary_type: DictionaryType,
    pub topic: String,
    pub article: Option<DictionaryRecord>,
}

/// In-memory dictionary lookup engine coordinating searches across multiple dictionaries.
pub struct DictionaryLookupEngine {
    dictionaries: Vec<Arc<DictionaryReader>>,
    current_strong_dictionary: Option<String>,
    current_general_dictionary: Option<String>,
    mode: LookupMode,
}

impl DictionaryLookupEngine {
    pub fn new() -> Self {
        Self {
            dictionaries: Vec::new(),
            current_strong_dictionary: None,
            current_general_dictionary: None,
            mode: LookupMode::default(),
        }
    }

    pub fn set_mode(&mut self, mode: LookupMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> LookupMode {
        self.mode
    }

    pub fn add_dictionary(&mut self, reader: Arc<DictionaryReader>) {
        if self.current_strong_dictionary.is_none()
            && reader.dictionary_type() == DictionaryType::StrongLexicon
        {
            self.current_strong_dictionary = Some(reader.abbreviation().to_string());
        }
        if self.current_general_dictionary.is_none()
            && reader.dictionary_type() != DictionaryType::StrongLexicon
        {
            self.current_general_dictionary = Some(reader.abbreviation().to_string());
        }
        self.dictionaries.push(reader);
    }

    pub fn set_current_strong(&mut self, abbrev: &str) {
        self.current_strong_dictionary = Some(abbrev.to_string());
    }

    pub fn set_current_general(&mut self, abbrev: &str) {
        self.current_general_dictionary = Some(abbrev.to_string());
    }

    /// Looks up an article for a Strong's number (e.g. "H1254", "G25", "7225").
    pub fn lookup_strong(&self, strong_number: &str) -> Result<Option<LookupMatch>> {
        let trimmed = strong_number.trim();
        // Priority to currently selected Strong's lexicon
        if let Some(ref current_name) = self.current_strong_dictionary {
            if let Some(dict) = self
                .dictionaries
                .iter()
                .find(|d| d.abbreviation() == current_name)
            {
                if let Some(entry) = self.find_in_dict(dict, trimmed)? {
                    return Ok(Some(entry));
                }
            }
        }

        // Try any Strong's lexicon
        for dict in &self.dictionaries {
            if dict.dictionary_type() == DictionaryType::StrongLexicon {
                if let Some(entry) = self.find_in_dict(dict, trimmed)? {
                    return Ok(Some(entry));
                }
            }
        }

        // Try any dictionary
        for dict in &self.dictionaries {
            if let Some(entry) = self.find_in_dict(dict, trimmed)? {
                return Ok(Some(entry));
            }
        }

        Ok(None)
    }

    fn find_in_dict(&self, dict: &DictionaryReader, query: &str) -> Result<Option<LookupMatch>> {
        // Direct match
        if let Some(article) = dict.get_topic(query)? {
            return Ok(Some(LookupMatch {
                dictionary_name: dict.abbreviation().to_string(),
                dictionary_type: dict.dictionary_type(),
                topic: article.topic.clone(),
                article: Some(article),
            }));
        }

        // If query has digits only, try with "H" and "G" prefixes
        if query.chars().all(|c| c.is_ascii_digit()) {
            for prefix in &["H", "G"] {
                let candidate = format!("{}{}", prefix, query);
                if let Some(article) = dict.get_topic(&candidate)? {
                    return Ok(Some(LookupMatch {
                        dictionary_name: dict.abbreviation().to_string(),
                        dictionary_type: dict.dictionary_type(),
                        topic: article.topic.clone(),
                        article: Some(article),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Looks up a word according to the MyBible dictionary lookup algorithm:
    /// Stage 1: NF1 and NF2
    /// Stage 2: NF2
    pub fn lookup_word(
        &self,
        word: &str,
        book: Option<BookNumber>,
        chapter: Option<u32>,
        verse: Option<u32>,
    ) -> Result<Vec<LookupMatch>> {
        let mut results = Vec::new();

        for dict in &self.dictionaries {
            let proc_rules = dict.words_processing()?;
            let nf1 = normalize_form_1(word, &proc_rules);
            let nf2 = normalize_form_2(word, &proc_rules);

            // 1. Check direct topic in dictionary
            if let Some(entry) = dict.get_topic(word)? {
                results.push(LookupMatch {
                    dictionary_name: dict.abbreviation().to_string(),
                    dictionary_type: dict.dictionary_type(),
                    topic: entry.topic.clone(),
                    article: Some(entry),
                });
                continue;
            }

            // 2. Check words table mappings
            let words = dict.words()?;
            for w in words {
                let matches_place = match (w.book_number, w.chapter_number, w.verse_number) {
                    (Some(b), Some(ch), Some(v)) => {
                        book == Some(b) && chapter == Some(ch) && verse == Some(v)
                    }
                    _ => true,
                };

                if !matches_place {
                    continue;
                }

                // Check variations (separated by '|')
                let variations: Vec<&str> = w.variation.split('|').collect();
                let is_match = variations.iter().any(|v| {
                    let v_nf1 = normalize_form_1(v, &proc_rules);
                    let v_nf2 = normalize_form_2(v, &proc_rules);
                    match dict.matching_type() {
                        MatchingType::Exact => v.eq_ignore_ascii_case(word),
                        MatchingType::NormalizedForm1 => v_nf1 == nf1,
                        MatchingType::NormalizedForm2 => v_nf2 == nf2,
                    }
                });

                if is_match {
                    if let Some(entry) = dict.get_topic(&w.standard_form)? {
                        results.push(LookupMatch {
                            dictionary_name: dict.abbreviation().to_string(),
                            dictionary_type: dict.dictionary_type(),
                            topic: entry.topic.clone(),
                            article: Some(entry),
                        });
                    }
                }
            }
        }

        Ok(results)
    }
}

impl Default for DictionaryLookupEngine {
    fn default() -> Self {
        Self::new()
    }
}
