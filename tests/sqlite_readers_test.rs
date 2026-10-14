use mybible::prelude::*;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn test_bible_reader_full_flow() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("TEST.SQLite3");

    // Create a mock MyBible Bible SQLite database
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "
        CREATE TABLE info (name TEXT, value TEXT);
        INSERT INTO info VALUES ('description', 'Test Bible Module');
        INSERT INTO info VALUES ('language', 'en');
        INSERT INTO info VALUES ('chapter_string', 'Chapter');
        INSERT INTO info VALUES ('strong_numbers', 'true');

        CREATE TABLE books_all (
            book_number NUMERIC,
            book_color TEXT,
            short_name TEXT,
            title TEXT,
            long_name TEXT,
            is_present BOOLEAN,
            sorting_order NUMERIC
        );
        INSERT INTO books_all VALUES (10, '#ccccff', 'Gen', 'The Book of Genesis', 'Genesis', 1, 10);
        INSERT INTO books_all VALUES (20, '#ccccff', 'Exo', 'Exodus', 'Exodus', 1, 20);

        CREATE TABLE verses (
            book_number NUMERIC,
            chapter NUMERIC,
            verse NUMERIC,
            text TEXT
        );
        INSERT INTO verses VALUES (10, 1, 1, 'In the beginning<S>7225</S> God created.');
        INSERT INTO verses VALUES (10, 1, 2, 'And the earth was without form, and void.');
        INSERT INTO verses VALUES (10, 1, 3, 'And God said, Let there be light.');
        INSERT INTO verses VALUES (10, 2, 1, 'Thus the heavens and the earth were finished.');

        CREATE TABLE introductions (
            book_number NUMERIC,
            introduction TEXT,
            PRIMARY KEY (book_number)
        );
        INSERT INTO introductions VALUES (0, '<h1>General Introduction</h1>');
        INSERT INTO introductions VALUES (10, '<h2>Genesis Introduction</h2>');

        CREATE TABLE stories (
            book_number NUMERIC,
            chapter NUMERIC,
            verse NUMERIC,
            order_if_several NUMERIC,
            title TEXT
        );
        INSERT INTO stories VALUES (10, 1, 1, 1, 'The Creation');
        ",
    )
    .unwrap();
    drop(conn);

    // Now test BibleReader
    let reader = BibleReader::open(&db_path, "TEST").unwrap();
    assert_eq!(reader.abbreviation(), "TEST");
    assert_eq!(reader.info().description(), Some("Test Bible Module"));
    assert!(reader.info().strong_numbers());

    // Books
    let books = reader.books().unwrap();
    assert_eq!(books.len(), 2);
    assert_eq!(books[0].short_name, "Gen");
    assert_eq!(books[0].display_title(), "The Book of Genesis");

    // Verses
    let v1 = reader.verse(10, 1, 1).unwrap().unwrap();
    assert_eq!(v1.text, "In the beginning<S>7225</S> God created.");

    let parsed = parse_verse_text(&v1.text);
    assert_eq!(
        nodes_to_plain_text(&parsed),
        "In the beginning God created."
    );

    let ch1 = reader.verses_in_chapter(10, 1).unwrap();
    assert_eq!(ch1.len(), 3);

    // Counts
    assert_eq!(reader.chapter_count(10).unwrap(), 2);
    assert_eq!(reader.verse_count(10, 1).unwrap(), 3);

    // Introductions
    assert_eq!(
        reader.module_introduction().unwrap(),
        Some("<h1>General Introduction</h1>".to_string())
    );
    assert_eq!(
        reader.book_introduction(10).unwrap(),
        Some("<h2>Genesis Introduction</h2>".to_string())
    );

    // Stories
    let stories = reader.stories_for_verse(10, 1, 1).unwrap();
    assert_eq!(stories.len(), 1);
    assert_eq!(stories[0].title, "The Creation");

    // Search
    let search = reader.search_text("light", 10).unwrap();
    assert_eq!(search.len(), 1);
    assert_eq!(search[0].verse, 3);
}

#[test]
fn test_commentary_and_footnote_flow() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("TEST.commentaries.SQLite3");

    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "
        CREATE TABLE info (name TEXT, value TEXT);
        INSERT INTO info VALUES ('is_footnotes', 'true');

        CREATE TABLE commentaries (
            book_number NUMERIC,
            chapter_number_from NUMERIC,
            verse_number_from NUMERIC,
            chapter_number_to NUMERIC,
            verse_number_to NUMERIC,
            is_preceding NUMERIC,
            marker TEXT,
            text TEXT
        );
        INSERT INTO commentaries VALUES (10, 1, 1, NULL, NULL, 0, '[1]', '<p>Or, in the start.</p>');
        INSERT INTO commentaries VALUES (10, 1, 1, 1, 3, 0, NULL, '<p>Section on creation week</p>');
        ",
    )
    .unwrap();
    drop(conn);

    let reader = CommentaryReader::open(&db_path, "TEST").unwrap();
    assert!(reader.is_footnotes());

    let footnote = reader
        .commentary_for_footnote(10, 1, 1, "[1]")
        .unwrap()
        .unwrap();
    assert_eq!(footnote.text, "<p>Or, in the start.</p>");

    let comments = reader.commentaries_for_verse(10, 1, 2).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text, "<p>Section on creation week</p>");
}

#[test]
fn test_cross_references_reverse_processing() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("TEST.crossreferences.SQLite3");

    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "
        CREATE TABLE info (name TEXT, value TEXT);
        INSERT INTO info VALUES ('requires_reverse_processing', 'true');

        CREATE TABLE cross_references (
            book NUMERIC,
            chapter NUMERIC,
            verse NUMERIC,
            verse_end NUMERIC,
            book_to NUMERIC,
            chapter_to NUMERIC,
            verse_to_start NUMERIC,
            verse_to_end NUMERIC,
            votes NUMERIC
        );
        -- Direct: Gen 1:1 -> John 1:1
        INSERT INTO cross_references VALUES (10, 1, 1, 0, 500, 1, 1, 0, 15);
        -- Reverse: Heb 11:3 -> Gen 1:1 (so when asking for Gen 1:1, we get Heb 11:3!)
        INSERT INTO cross_references VALUES (650, 11, 3, 0, 10, 1, 1, 0, 20);
        ",
    )
    .unwrap();
    drop(conn);

    let reader = CrossReferenceReader::open(&db_path, "TEST").unwrap();
    let refs = reader.references_for_verse(10, 1, 1).unwrap();

    // Should include both John 1:1 and Hebrews 11:3, with Heb 11:3 having 20 votes (first) and John 1:1 having 15 votes (second)
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].book_to, BookNumber(650));
    assert_eq!(refs[0].chapter_to, 11);
    assert_eq!(refs[0].verse_to_start, 3);
    assert_eq!(refs[0].votes, Some(20));

    assert_eq!(refs[1].book_to, BookNumber(500));
    assert_eq!(refs[1].chapter_to, 1);
    assert_eq!(refs[1].verse_to_start, 1);
    assert_eq!(refs[1].votes, Some(15));
}

#[test]
fn test_dictionary_and_lookup_engine() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("Strong.dictionary.SQLite3");

    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "
        CREATE TABLE info (name TEXT, value TEXT);
        INSERT INTO info VALUES ('type', 'strong lexicon');

        CREATE TABLE dictionary (
            topic TEXT,
            definition TEXT,
            short_definition TEXT,
            lexeme TEXT,
            transliteration TEXT,
            pronunciation TEXT
        );
        INSERT INTO dictionary VALUES ('H7225', '<b>Beginning</b>', 'beginning', 'רֵאשִׁית', 'reshith', 'ray-sheeth');
        INSERT INTO dictionary VALUES ('G25', '<b>Love</b>', 'to love', 'ἀγαπάω', 'agapao', 'ag-ap-ah-o');

        CREATE TABLE words (
            standard_form TEXT,
            variation TEXT
        );
        INSERT INTO words VALUES ('H7225', 'beginning|beginnings|head');
        ",
    )
    .unwrap();
    drop(conn);

    let reader = std::sync::Arc::new(DictionaryReader::open(&db_path, "Strong").unwrap());
    let mut lookup = DictionaryLookupEngine::new();
    lookup.add_dictionary(reader);

    // Look up by Strong number
    let match_h = lookup.lookup_strong("H7225").unwrap().unwrap();
    assert_eq!(match_h.topic, "H7225");
    assert_eq!(
        match_h.article.as_ref().unwrap().lexeme.as_deref(),
        Some("רֵאשִׁית")
    );

    // Look up by bare digits "7225"
    let match_bare = lookup.lookup_strong("7225").unwrap().unwrap();
    assert_eq!(match_bare.topic, "H7225");

    // Look up by word variation
    let word_matches = lookup.lookup_word("beginnings", None, None, None).unwrap();
    assert_eq!(word_matches.len(), 1);
    assert_eq!(word_matches[0].topic, "H7225");
}

#[test]
fn test_engine_composite_integration() {
    let dir = tempdir().unwrap();

    // 1. Bible
    let bbl_path = dir.path().join("ENG.SQLite3");
    let conn_bbl = Connection::open(&bbl_path).unwrap();
    conn_bbl.execute_batch(
        "
        CREATE TABLE info (name TEXT, value TEXT);
        INSERT INTO info VALUES ('description', 'English Bible');

        CREATE TABLE books (book_number NUMERIC, book_color TEXT, short_name TEXT, long_name TEXT, sorting_order NUMERIC);
        INSERT INTO books VALUES (10, '#ccccff', 'Gen', 'Genesis', 10);

        CREATE TABLE verses (book_number NUMERIC, chapter NUMERIC, verse NUMERIC, text TEXT);
        INSERT INTO verses VALUES (10, 1, 1, 'In the beginning <f>[1]</f> God created the heaven.');

        CREATE TABLE content_fragments (id TEXT, fragment TEXT);
        INSERT INTO content_fragments VALUES ('footnote_note', '<i>Note from translator</i>');
        ",
    ).unwrap();
    drop(conn_bbl);

    // 2. Commentary / Footnotes
    let comm_path = dir.path().join("ENG.commentaries.SQLite3");
    let conn_comm = Connection::open(&comm_path).unwrap();
    conn_comm.execute_batch(
        "
        CREATE TABLE info (name TEXT, value TEXT);
        INSERT INTO info VALUES ('is_footnotes', 'true');

        CREATE TABLE commentaries (book_number NUMERIC, chapter_number_from NUMERIC, verse_number_from NUMERIC, chapter_number_to NUMERIC, verse_number_to NUMERIC, is_preceding NUMERIC, marker TEXT, text TEXT);
        INSERT INTO commentaries VALUES (10, 1, 1, NULL, NULL, 0, '[1]', 'First verse footnote: <!-- INCLUDE(footnote_note) -->');
        ",
    ).unwrap();
    drop(conn_comm);

    // 3. Subheadings
    let sub_path = dir.path().join("ENG.subheadings.SQLite3");
    let conn_sub = Connection::open(&sub_path).unwrap();
    conn_sub.execute_batch(
        "
        CREATE TABLE info (name TEXT, value TEXT);
        CREATE TABLE subheadings (book_number NUMERIC, chapter NUMERIC, verse NUMERIC, order_if_several NUMERIC, subheading TEXT);
        INSERT INTO subheadings VALUES (10, 1, 1, 1, 'The Creation of Heaven and Earth');
        ",
    ).unwrap();
    drop(conn_sub);

    // Initialize MyBibleEngine and load directory
    let mut engine = MyBibleEngine::new();
    let loaded = engine.load_directory(dir.path()).unwrap();
    assert_eq!(loaded, 3);

    // Read composite verse
    let comp = engine.read_composite_verse(Some("ENG"), 10, 1, 1).unwrap();
    assert_eq!(
        comp.verse.text,
        "In the beginning <f>[1]</f> God created the heaven."
    );
    assert_eq!(comp.subheadings.len(), 1);
    assert_eq!(
        comp.subheadings[0].subheading,
        "The Creation of Heaven and Earth"
    );
    assert_eq!(comp.footnotes.len(), 1);
    assert!(comp.footnotes[0].text.contains("First verse footnote"));

    // Render HTML with cross-module fragment inclusion!
    let rendered_footnote = engine
        .render_html(&comp.footnotes[0].text, Some("ENG.commentaries"))
        .unwrap();
    assert_eq!(
        rendered_footnote,
        "First verse footnote: <i>Note from translator</i>"
    );
}
