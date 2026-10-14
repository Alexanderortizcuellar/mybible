use mybible::prelude::*;
use std::path::Path;

#[test]
fn test_real_nlt_bible() {
    let path = Path::new("modules/bibles/NLT.SQLite3");
    if !path.exists() {
        println!("Skipping test: real NLT.SQLite3 not found");
        return;
    }

    let bible = BibleReader::open(path, "NLT").expect("Failed to open NLT.SQLite3");
    assert_eq!(bible.abbreviation(), "NLT");

    println!("=== NLT Bible Metadata ===");
    println!("Description: {:?}", bible.info().description());
    println!("Language: {:?}", bible.info().language());
    println!("Origin: {:?}", bible.info().origin());

    // Books
    let books = bible.books().expect("Failed to read books");
    println!("Total books defined: {}", books.len());
    assert!(!books.is_empty(), "Should have books in NLT");

    let present = bible.present_books().expect("Failed to read present books");
    println!("Present books: {}", present.len());

    // Check Genesis (Book 10)
    let gen = bible
        .book(BookNumber(10))
        .expect("Failed to lookup Gen")
        .expect("Gen should exist");
    println!("Book 10: {} ({})", gen.display_title(), gen.short_name);

    // Read Genesis 1:1
    let gen_1_1 = bible
        .verse(BookNumber(10), 1, 1)
        .expect("Failed to query Gen 1:1")
        .expect("Gen 1:1 should exist");
    println!("Gen 1:1 raw: {}", gen_1_1.text);

    let parsed = parse_verse_text(&gen_1_1.text);
    let plain = nodes_to_plain_text(&parsed);
    println!("Gen 1:1 plain: {}", plain);
    assert!(!plain.is_empty());

    // Read John 3:16 (Book 500)
    let john_3_16 = bible
        .verse(BookNumber(500), 3, 16)
        .expect("Failed to query John 3:16")
        .expect("John 3:16 should exist");
    println!("John 3:16 raw: {}", john_3_16.text);
    let plain_john = nodes_to_plain_text(&parse_verse_text(&john_3_16.text));
    println!("John 3:16 plain: {}", plain_john);
    assert!(
        plain_john.to_lowercase().contains("loved") || plain_john.to_lowercase().contains("god")
    );

    // Check chapter counts
    let gen_ch_count = bible.chapter_count(BookNumber(10)).expect("Gen ch count");
    println!("Genesis chapter count: {}", gen_ch_count);
    assert_eq!(gen_ch_count, 50);

    let gen_ch1_vcount = bible.verse_count(BookNumber(10), 1).expect("Gen 1 vcount");
    println!("Genesis 1 verse count: {}", gen_ch1_vcount);
    assert_eq!(gen_ch1_vcount, 31);
}

#[test]
fn test_real_nlt_commentaries_and_footnotes() {
    let path = Path::new("modules/commentaries/NLT.commentaries.SQLite3");
    if !path.exists() {
        println!("Skipping test: real NLT.commentaries.SQLite3 not found");
        return;
    }

    let comm =
        CommentaryReader::open(path, "NLT").expect("Failed to open NLT.commentaries.SQLite3");
    println!("=== NLT Commentaries / Footnotes ===");
    println!("Description: {:?}", comm.info().description());
    println!("Is Footnotes: {}", comm.is_footnotes());

    // Query footnotes/commentaries for Genesis 1:1
    let recs = comm
        .commentaries_for_verse(BookNumber(10), 1, 1)
        .expect("Query Gen 1:1 footnotes");
    println!("Gen 1:1 footnotes found: {}", recs.len());
    for r in &recs {
        println!("  Marker: {:?}, Text: {}", r.marker, r.text);
    }
}

#[test]
fn test_real_obx_crossreferences() {
    let path = Path::new("modules/crossreferences/OBX.crossreferences.SQLite3");
    if !path.exists() {
        println!("Skipping test: real OBX.crossreferences.SQLite3 not found");
        return;
    }

    let cr = CrossReferenceReader::open(path, "OBX")
        .expect("Failed to open OBX.crossreferences.SQLite3");
    println!("=== OBX Cross References ===");
    println!("Description: {:?}", cr.info().description());
    println!(
        "Requires reverse processing: {}",
        cr.info().requires_reverse_processing()
    );

    // Query cross-references for John 3:16 (Book 500, Ch 3, V 16)
    let refs = cr
        .references_for_verse(BookNumber(500), 3, 16)
        .expect("Query John 3:16 cross-references");
    println!("John 3:16 cross references count: {}", refs.len());
    assert!(
        !refs.is_empty(),
        "John 3:16 should have cross references in OBX"
    );

    for (i, r) in refs.iter().take(5).enumerate() {
        println!(
            "  {}. -> Book {} {}:{}-{} (votes: {:?})",
            i + 1,
            r.book_to.0,
            r.chapter_to,
            r.verse_to_start,
            r.verse_to_end.unwrap_or(r.verse_to_start),
            r.votes
        );
    }
}

#[test]
fn test_real_dbmh_dictionary() {
    let path = Path::new("modules/dictionaries/DBMH.dictionary.SQLite3");
    if !path.exists() {
        println!("Skipping test: real DBMH.dictionary.SQLite3 not found");
        return;
    }

    let dict =
        DictionaryReader::open(path, "DBMH").expect("Failed to open DBMH.dictionary.SQLite3");
    println!("=== DBMH Dictionary ===");
    println!("Description: {:?}", dict.info().description());
    println!("Type: {:?}", dict.dictionary_type());

    // Search topics prefix
    let sample_topics = dict.search_topics_prefix("A", 10).expect("Search prefix A");
    println!("Sample topics starting with A: {:?}", sample_topics);
    assert!(
        !sample_topics.is_empty(),
        "DBMH should have topics starting with A"
    );

    // Fetch the first topic definition
    let first_topic = &sample_topics[0];
    let entry = dict
        .get_topic(first_topic)
        .expect("Get topic")
        .expect("Topic should exist");
    println!(
        "Topic '{}': len(definition) = {}",
        entry.topic,
        entry.definition.len()
    );
    assert!(!entry.definition.is_empty());
}

#[test]
fn test_real_modules_full_engine() {
    let modules_dir = Path::new("modules");
    if !modules_dir.exists() {
        println!("Skipping test: modules folder not found");
        return;
    }

    let mut engine = MyBibleEngine::new();
    let loaded = engine
        .load_directory(modules_dir)
        .expect("Load modules directory");
    println!("=== Full MyBible Engine ===");
    println!("Total modules loaded from 'modules/': {}", loaded);
    assert!(loaded >= 4, "Should load at least 4 modules");

    println!(
        "Loaded Bibles: {:?}",
        engine.bibles.keys().collect::<Vec<_>>()
    );
    println!(
        "Loaded Commentaries: {:?}",
        engine.commentaries.keys().collect::<Vec<_>>()
    );
    println!(
        "Loaded CrossReferences: {:?}",
        engine.cross_references.keys().collect::<Vec<_>>()
    );
    println!(
        "Loaded Dictionaries: {:?}",
        engine.dictionaries.keys().collect::<Vec<_>>()
    );

    // Set active modules
    engine.active_bible = Some("NLT".to_string());
    engine.active_commentary = Some("NLT".to_string());
    engine.active_cross_references = Some("OBX".to_string());

    // Read a composite verse (e.g. Genesis 1:1 or John 3:16)
    let comp = engine
        .read_composite_verse(Some("NLT"), BookNumber(10), 1, 1)
        .expect("Read composite Gen 1:1");
    println!("Composite Verse Gen 1:1:");
    println!("  Verse Text: {}", comp.verse.text);
    println!(
        "  Plain Text: {}",
        nodes_to_plain_text(&parse_verse_text(&comp.verse.text))
    );
    println!("  Footnotes count: {}", comp.footnotes.len());
    println!("  Cross references count: {}", comp.cross_references.len());

    assert!(
        !comp.cross_references.is_empty(),
        "Should have cross references for Gen 1:1 in OBX"
    );

    // Read a composite chapter (Genesis 1)
    let comp_chapter = engine
        .read_composite_chapter(Some("NLT"), BookNumber(10), 1)
        .expect("Read Genesis 1 chapter");
    println!(
        "Composite Chapter: {} (verses: {})",
        comp_chapter.book.display_title(),
        comp_chapter.verses.len()
    );
    assert_eq!(comp_chapter.verses.len(), 31);
}

#[test]
fn test_real_esv_study_commentaries() {
    let path = Path::new("modules/commentaries/ESVstudy.commentaries.SQLite3");
    if !path.exists() {
        println!("Skipping: ESVstudy.commentaries.SQLite3 not found");
        return;
    }

    let comm = CommentaryReader::open(path, "ESVstudy").expect("Open ESVstudy");
    println!("=== ESV Study Bible Commentaries ===");
    println!("Description: {:?}", comm.info().description());
    println!("Is Footnotes: {}", comm.is_footnotes());
    assert!(
        !comm.is_footnotes(),
        "ESV Study Bible notes are commentaries, not footnotes"
    );

    // Query Genesis 1:1 notes
    let notes_gen_1_1 = comm
        .commentaries_for_verse(BookNumber(10), 1, 1)
        .expect("Notes for Gen 1:1");
    println!("Gen 1:1 notes count: {}", notes_gen_1_1.len());
    assert!(!notes_gen_1_1.is_empty());
    println!("Sample Gen 1:1 note: {}", &notes_gen_1_1[0].text[..120]);

    // Query John 3:16 notes (Book 500)
    let notes_john_3_16 = comm
        .commentaries_for_verse(BookNumber(500), 3, 16)
        .expect("Notes for John 3:16");
    println!("John 3:16 notes count: {}", notes_john_3_16.len());
    assert!(!notes_john_3_16.is_empty());
    println!("Sample John 3:16 note: {}", &notes_john_3_16[0].text[..120]);

    // Query Romans 8:28 notes (Book 520)
    let notes_rom_8_28 = comm
        .commentaries_for_verse(BookNumber(520), 8, 28)
        .expect("Notes for Rom 8:28");
    println!("Romans 8:28 notes count: {}", notes_rom_8_28.len());
    assert!(!notes_rom_8_28.is_empty());

    // Query the fixed 1 Chronicles 10 note (Book 130)
    let notes_1chron = comm
        .commentaries_for_verse(BookNumber(130), 10, 1)
        .expect("Notes for 1 Chron 10:1");
    assert!(!notes_1chron.is_empty());
}

#[test]
fn test_real_kjv_subheadings() {
    let home = dirs::home_dir().unwrap();
    let path = home
        .join(".mybible")
        .join("modules")
        .join("KJV-s.subheadings.SQLite3");
    if !path.exists() {
        println!("Skipping: KJV-s.subheadings.SQLite3 not found");
        return;
    }

    let sub = SubheadingReader::open(&path, "KJV-s").expect("Open KJV-s");
    println!("=== KJV Subheadings ===");
    println!("Description: {:?}", sub.info().description());

    let gen_1_subs = sub
        .subheadings_in_chapter(BookNumber(10), 1)
        .expect("Subheadings in Gen 1");
    println!("Gen 1 subheadings count: {}", gen_1_subs.len());
    for s in &gen_1_subs {
        println!("Verse {}: {}", s.verse, s.subheading);
    }
    assert!(!gen_1_subs.is_empty());

    let matt_3_subs = sub
        .subheadings_in_chapter(BookNumber(470), 3)
        .expect("Subheadings in Matt 3");
    println!("Matt 3 subheadings count: {}", matt_3_subs.len());
    for s in &matt_3_subs {
        println!("Verse {}: {}", s.verse, s.subheading);
    }
}
