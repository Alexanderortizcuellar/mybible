use mybible::prelude::*;
use std::path::Path;

#[test]
fn test_search_mode_all_words() {
    let plain = "For God so loved the world that he gave his one and only Son";
    let query = SearchQuery::new("god loved son");
    let matches = search_verse_text("", plain, &query);
    assert!(matches.is_some());
    let spans = matches.unwrap();
    assert_eq!(spans.len(), 3);

    let query_miss = SearchQuery::new("god loved daughter");
    let matches_miss = search_verse_text("", plain, &query_miss);
    assert!(matches_miss.is_none());
}

#[test]
fn test_search_mode_phrase() {
    let plain = "In the beginning was the Word, and the Word was with God";
    let query = SearchQuery::new("\"Word was with God\"");
    assert_eq!(query.mode, SearchMode::Phrase);

    let matches = search_verse_text("", plain, &query);
    assert!(matches.is_some());
    let spans = matches.unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(&plain[spans[0].start..spans[0].end], "Word was with God");

    let query_fail = SearchQuery::new("\"Word God\"");
    let matches_fail = search_verse_text("", plain, &query_fail);
    assert!(matches_fail.is_none());
}

#[test]
fn test_search_mode_any_word() {
    let plain = "Grace and peace be multiplied unto you";
    let query = SearchQuery::new("peace wrath judgment").with_mode(SearchMode::AnyWord);
    let matches = search_verse_text("", plain, &query);
    assert!(matches.is_some());
    let spans = matches.unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(&plain[spans[0].start..spans[0].end], "peace");
}

#[test]
fn test_search_accent_and_case_insensitivity() {
    let plain = "Arrepiéntanse, porque el reino de los cielos se ha acercado.";
    let query = SearchQuery::new("arrepientanse reino");
    let matches = search_verse_text("", plain, &query);
    assert!(matches.is_some());
    let spans = matches.unwrap();
    assert_eq!(spans.len(), 2);

    let hit = SearchHit {
        bible_module: "TEST".to_string(),
        book_number: BookNumber(400),
        book_name: "Mateo".to_string(),
        chapter: 3,
        verse: 2,
        text: plain.to_string(),
        plain_text: plain.to_string(),
        matches: spans,
    };

    let rendered = hit.render_highlighted(false);
    assert!(rendered.contains("[Arrepiéntanse]"));
    assert!(rendered.contains("[reino]"));
}

#[test]
fn test_real_nlt_search() {
    let path = Path::new("modules/bibles/NLT.SQLite3");
    if !path.exists() {
        println!("Skipping real NLT search test: module not found");
        return;
    }

    let bible = BibleReader::open(path, "NLT").expect("Failed to open NLT");

    // 1. Search for "repent kingdom"
    let query = SearchQuery::new("repent kingdom");
    let hits = bible.search(&query).expect("Search failed");
    assert!(!hits.is_empty(), "Should find matches for 'repent kingdom'");

    // Check that Matthew 3:2 or Matthew 4:17 is in results
    let found_matthew_3_2 = hits
        .iter()
        .any(|h| h.book_number == BookNumber(470) && h.chapter == 3 && h.verse == 2);
    assert!(
        found_matthew_3_2,
        "Should find Matthew 3:2 (book 470) in 'repent kingdom' search"
    );

    // 2. Search scoped to New Testament only
    let nt_query = SearchQuery::new("beginning").with_scope(SearchScope::NewTestament);
    let nt_hits = bible.search(&nt_query).expect("NT search failed");
    for hit in &nt_hits {
        assert!(
            hit.book_number.0 >= 400 && hit.book_number.0 <= 660,
            "Verse should be in NT: {:?}",
            hit.format_reference()
        );
    }

    // 3. Search scoped to a specific Book (Genesis)
    let gen_query = SearchQuery::new("created").with_scope(SearchScope::Book(BookNumber(10)));
    let gen_hits = bible.search(&gen_query).expect("Gen search failed");
    for hit in &gen_hits {
        assert_eq!(hit.book_number, BookNumber(10));
    }
}

#[test]
fn test_engine_search_across_modules() {
    let path = Path::new("modules/bibles/NLT.SQLite3");
    if !path.exists() {
        return;
    }

    let mut engine = MyBibleEngine::new();
    engine.load_module_file(path).expect("Failed to load NLT");

    let query = SearchQuery::new("\"God loved the world\"");
    let results = engine
        .search(&query, &["NLT"])
        .expect("Engine search failed");
    assert_eq!(results.total_matches, 1);
    assert_eq!(results.hits[0].format_reference(), "John 3:16");
}
