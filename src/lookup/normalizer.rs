use crate::models::dictionary::{WordsProcessingRecord, WordsProcessingType};
use unicode_normalization::UnicodeNormalization;

/// Checks if a Unicode character is a combining diacritical mark or accent.
pub fn is_diacritic_mark(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'..='\u{036F}'
            | '\u{1AB0}'..='\u{1AFF}'
            | '\u{1DC0}'..='\u{1DFF}'
            | '\u{20D0}'..='\u{20FF}'
            | '\u{FE20}'..='\u{FE2F}'
            | '\u{0591}'..='\u{05BD}'
            | '\u{05BF}'
            | '\u{05C1}'..='\u{05C2}'
            | '\u{05C4}'..='\u{05C5}'
            | '\u{05C7}'
            | '\u{0610}'..='\u{061A}'
            | '\u{064B}'..='\u{065F}'
            | '\u{0670}'
            | '\u{0730}'..='\u{074A}'
            | '\u{1FEF}'
            | '\u{1FFD}'
            | '`'
            | '´'
    )
}

/// Applies string replacements for pre- or post-processing.
fn apply_rules(
    mut text: String,
    rules: &[WordsProcessingRecord],
    target_type: WordsProcessingType,
) -> String {
    for rule in rules {
        if rule.proc_type == target_type && !rule.input.is_empty() {
            text = text.replace(&rule.input, &rule.output);
        }
    }
    text
}

/// Computes Normalized Form 0 (NF0):
/// 1. Language-specific pre-processing
/// 2. Decompose Unicode diacritics into separate codepoints (NFD)
/// 3. Language-specific post-processing
pub fn normalize_form_0(word: &str, rules: &[WordsProcessingRecord]) -> String {
    // 1. Pre-processing
    let pre = apply_rules(word.to_string(), rules, WordsProcessingType::Pre);
    // 2. Decompose diacritics (NFD)
    let nfd: String = pre.nfd().collect();
    // 3. Post-processing
    apply_rules(nfd, rules, WordsProcessingType::Post)
}

/// Computes Normalized Form 1 (NF1):
/// 1. Language-specific pre-processing
/// 2. Lowercase
/// 3. Decompose Unicode diacritics into separate codepoints (NFD)
/// 4. Language-specific post-processing
pub fn normalize_form_1(word: &str, rules: &[WordsProcessingRecord]) -> String {
    // 1. Pre-processing
    let pre = apply_rules(word.to_string(), rules, WordsProcessingType::Pre);
    // 2. Lowercase
    let lower = pre.to_lowercase();
    // 3. Decompose diacritics (NFD)
    let nfd: String = lower.nfd().collect();
    // 4. Post-processing
    apply_rules(nfd, rules, WordsProcessingType::Post)
}

/// Computes Normalized Form 2 (NF2):
/// 1. Language-specific pre-processing
/// 2. Lowercase
/// 3. Decompose Unicode diacritics and strip all diacritical characters
/// 4. Language-specific post-processing
pub fn normalize_form_2(word: &str, rules: &[WordsProcessingRecord]) -> String {
    // 1. Pre-processing
    let pre = apply_rules(word.to_string(), rules, WordsProcessingType::Pre);
    // 2. Lowercase
    let lower = pre.to_lowercase();
    // 3. Decompose diacritics and remove them
    let stripped: String = lower.nfd().filter(|&c| !is_diacritic_mark(c)).collect();
    // 4. Post-processing
    apply_rules(stripped, rules, WordsProcessingType::Post)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nf1_and_nf2_accents() {
        let empty_rules = [];
        let word = "Création"; // with acute accent

        let nf1 = normalize_form_1(word, &empty_rules);
        assert_eq!(nf1, "cre\u{0301}ation");

        let nf2 = normalize_form_2(word, &empty_rules);
        assert_eq!(nf2, "creation");
    }

    #[test]
    fn test_data_driven_rules() {
        let rules = vec![
            WordsProcessingRecord {
                proc_type: WordsProcessingType::Pre,
                input: "vv".to_string(),
                output: "w".to_string(),
            },
            WordsProcessingRecord {
                proc_type: WordsProcessingType::Post,
                input: "th".to_string(),
                output: "t".to_string(),
            },
        ];

        let res = normalize_form_2("vvat-th", &rules);
        assert_eq!(res, "wat-t");
    }
}
