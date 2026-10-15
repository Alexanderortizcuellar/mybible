use crate::cli::commands::{Cli, Commands, ConfigCommands, ModulesCommands};
use crate::cli::config::CliConfig;
use crate::engine::MyBibleEngine;
use crate::error::{MyBibleError, Result};
use crate::parsers::reference::parse_passage_reference;
use crate::repository::RepositoryClient;
use crate::search::{SearchMode, SearchQuery, SearchScope};
use crate::terminal::{render_subheading, render_to_terminal};
use colored::*;
use std::fs;
use std::path::PathBuf;

pub struct CliRunner {
    pub cli: Cli,
    pub config: CliConfig,
    pub data_dir: PathBuf,
}

impl CliRunner {
    pub fn new(cli: Cli) -> Result<Self> {
        let config = CliConfig::load();
        let data_dir = CliConfig::resolve_data_dir(cli.data_dir.as_deref())?;
        Ok(Self {
            cli,
            config,
            data_dir,
        })
    }

    pub fn run(mut self) -> Result<()> {
        let use_color = !self.cli.no_color;
        let command = self.cli.command.clone();

        match &command {
            Commands::Config(cfg_cmd) => self.handle_config(cfg_cmd, use_color),
            Commands::Modules(mod_cmd) => self.handle_modules(mod_cmd, use_color),
            Commands::Read {
                reference,
                bible,
                footnotes,
                xrefs,
                subheadings,
                no_subheadings,
            } => self.handle_read(
                reference,
                bible.as_deref(),
                *footnotes,
                *xrefs,
                subheadings.as_deref(),
                *no_subheadings,
                use_color,
            ),
            Commands::Books { bible } => self.handle_books(bible.as_deref(), use_color),
            Commands::Search {
                query,
                bible,
                all,
                book,
                testament,
                mode,
                limit,
                case_sensitive,
                match_accents,
            } => self.handle_search(
                query,
                bible.as_deref(),
                *all,
                book.as_deref(),
                testament.as_deref(),
                mode,
                *limit,
                *case_sensitive,
                *match_accents,
                use_color,
            ),
            Commands::Commentary { reference, module } => {
                self.handle_commentary(reference, module.as_deref(), use_color)
            }
            Commands::Dict {
                term,
                module,
                suggest,
            } => self.handle_dict(term, module.as_deref(), *suggest, use_color),
            Commands::Xref { reference, preview } => {
                self.handle_xref(reference, *preview, use_color)
            }
            Commands::Plan { plan, day } => self.handle_plan(plan, *day, use_color),
            Commands::Devotion { module, day } => {
                self.handle_devotion(module.as_deref(), *day, use_color)
            }
        }
    }

    fn init_engine(&self) -> Result<MyBibleEngine> {
        let mut engine = MyBibleEngine::new();
        engine.load_directory(&self.data_dir)?;

        // Apply saved defaults
        if let Some(def_b) = &self.config.default_bible {
            if engine.bibles.contains_key(def_b) {
                engine.active_bible = Some(def_b.clone());
            }
        }
        if let Some(def_c) = &self.config.default_commentary {
            if engine.commentaries.contains_key(def_c) {
                engine.active_commentary = Some(def_c.clone());
            }
        }
        if let Some(def_s) = &self.config.default_subheadings {
            if engine.subheadings.contains_key(def_s) {
                engine.active_subheadings = Some(def_s.clone());
            }
        } else if engine.active_subheadings.is_none() {
            engine.active_subheadings = engine.subheadings.keys().next().cloned();
        }

        Ok(engine)
    }

    // --- READ COMMAND ---
    #[allow(clippy::too_many_arguments)]
    fn handle_read(
        &self,
        reference_str: &str,
        bible_arg: Option<&str>,
        show_footnotes: bool,
        show_xrefs: bool,
        subheadings_arg: Option<&str>,
        no_subheadings: bool,
        use_color: bool,
    ) -> Result<()> {
        let mut engine = self.init_engine()?;
        if let Some(sub_override) = subheadings_arg {
            engine.active_subheadings = Some(sub_override.to_string());
        }

        let parsed = parse_passage_reference(reference_str)?;

        let bible_key = bible_arg
            .map(|s| s.to_string())
            .or_else(|| self.config.default_bible.clone())
            .or_else(|| engine.active_bible.clone())
            .or_else(|| engine.bibles.keys().next().cloned())
            .ok_or_else(|| {
                MyBibleError::InvalidFormat(
                    "No Bible modules installed. Run 'mybible modules download <abbrev>' to install one."
                        .to_string(),
                )
            })?;

        let bible = engine
            .bibles
            .get(&bible_key)
            .ok_or_else(|| MyBibleError::ModuleNotFound(bible_key.clone()))?;

        // Determine range of verses to read
        let (v_start, v_end) = match (parsed.verse_start, parsed.verse_end) {
            (Some(s), Some(e)) => (s, e),
            (Some(s), None) => (s, s),
            (None, _) => {
                let count = bible.verse_count(parsed.book, parsed.chapter)?;
                (1, count)
            }
        };

        if self.cli.json {
            #[derive(serde::Serialize)]
            struct JsonReadResponse {
                reference: String,
                book_number: u16,
                book_name: String,
                chapter: u32,
                verse_start: u32,
                verse_end: u32,
                bible: String,
                subheadings: Vec<JsonSubheading>,
                verses: Vec<JsonVerse>,
            }

            #[derive(serde::Serialize)]
            struct JsonSubheading {
                verse: u32,
                title: String,
            }

            #[derive(serde::Serialize)]
            struct JsonVerse {
                verse: u32,
                text: String,
                plain_text: String,
                footnotes: Vec<JsonFootnote>,
                cross_references: Vec<String>,
            }

            #[derive(serde::Serialize)]
            struct JsonFootnote {
                marker: Option<String>,
                text: String,
            }

            let mut json_subheadings = Vec::new();
            let mut json_verses = Vec::new();

            for v in v_start..=v_end {
                if !no_subheadings {
                    let mut subs = Vec::new();
                    if let Some(ref sub_name) = engine.active_subheadings {
                        if let Some(sub_reader) = engine.subheadings.get(sub_name) {
                            if let Ok(s) =
                                sub_reader.subheadings_for_verse(parsed.book, parsed.chapter, v)
                            {
                                subs = s;
                            }
                        }
                    }
                    if subs.is_empty() {
                        if let Ok(stories) = bible.stories_for_verse(parsed.book, parsed.chapter, v)
                        {
                            for s in stories {
                                subs.push(crate::models::subheading::SubheadingRecord {
                                    book_number: s.book_number,
                                    chapter: s.chapter,
                                    verse: s.verse,
                                    order_if_several: s.order_if_several,
                                    subheading: s.title,
                                });
                            }
                        }
                    }
                    for s in subs {
                        json_subheadings.push(JsonSubheading {
                            verse: v,
                            title: crate::terminal::render_subheading(&s.subheading, false),
                        });
                    }
                }

                if let Some(verse_rec) = bible.verse(parsed.book, parsed.chapter, v)? {
                    let mut fns = Vec::new();
                    if show_footnotes {
                        if let Some(comm_reader) = engine.get_active_commentary() {
                            if comm_reader.is_footnotes() {
                                if let Ok(notes) = comm_reader.commentaries_for_verse(
                                    parsed.book,
                                    parsed.chapter,
                                    v,
                                ) {
                                    for n in notes {
                                        fns.push(JsonFootnote {
                                            marker: n.marker,
                                            text: crate::terminal::render_plain_text(&n.text),
                                        });
                                    }
                                }
                            }
                        }
                    }

                    let mut xref_list = Vec::new();
                    if show_xrefs {
                        if let Some(xref_name) = &engine.active_cross_references {
                            if let Some(xref_reader) = engine.cross_references.get(xref_name) {
                                if let Ok(refs) =
                                    xref_reader.references_for_verse(parsed.book, parsed.chapter, v)
                                {
                                    for r in refs {
                                        xref_list.push(r.format_target());
                                    }
                                }
                            }
                        }
                    }

                    let ast = crate::parsers::verse_text::parse_verse_text(&verse_rec.text);
                    let plain = crate::parsers::verse_text::nodes_to_plain_text(&ast);

                    json_verses.push(JsonVerse {
                        verse: v,
                        text: verse_rec.text,
                        plain_text: plain,
                        footnotes: fns,
                        cross_references: xref_list,
                    });
                }
            }

            let resp = JsonReadResponse {
                reference: parsed.display_string(),
                book_number: parsed.book.0,
                book_name: parsed.book_name.clone(),
                chapter: parsed.chapter,
                verse_start: v_start,
                verse_end: v_end,
                bible: bible_key,
                subheadings: json_subheadings,
                verses: json_verses,
            };

            println!("{}", serde_json::to_string_pretty(&resp)?);
            return Ok(());
        }

        // Print terminal header
        let header = format!("📖 {} ({})", parsed.display_string(), bible_key);
        if use_color {
            println!("\n{}\n", header.bright_cyan().bold());
        } else {
            println!("\n{}\n", header);
        }

        for v in v_start..=v_end {
            // Check for subheadings before this verse
            if !no_subheadings {
                let mut subheadings = Vec::new();
                if let Some(ref sub_name) = engine.active_subheadings {
                    if let Some(sub_reader) = engine.subheadings.get(sub_name) {
                        if let Ok(subs) =
                            sub_reader.subheadings_for_verse(parsed.book, parsed.chapter, v)
                        {
                            subheadings = subs;
                        }
                    }
                }
                if subheadings.is_empty() {
                    if let Ok(stories) = bible.stories_for_verse(parsed.book, parsed.chapter, v) {
                        for s in stories {
                            subheadings.push(crate::models::subheading::SubheadingRecord {
                                book_number: s.book_number,
                                chapter: s.chapter,
                                verse: s.verse,
                                order_if_several: s.order_if_several,
                                subheading: s.title,
                            });
                        }
                    }
                }

                for s in subheadings {
                    let rendered = render_subheading(&s.subheading, use_color);
                    if use_color {
                        println!("\n  ── {} ──\n", rendered.bright_green().bold());
                    } else {
                        println!("\n  ── {} ──\n", rendered);
                    }
                }
            }

            if let Some(verse_rec) = bible.verse(parsed.book, parsed.chapter, v)? {
                // Verse number
                let v_label = format!("{:>3} ", v);
                let text_rendered = render_to_terminal(&verse_rec.text, use_color);

                if use_color {
                    println!("{} {}", v_label.bright_yellow().bold(), text_rendered);
                } else {
                    println!("{} {}", v_label, text_rendered);
                }

                // Inline Footnotes
                if show_footnotes {
                    if let Some(comm_reader) = engine.get_active_commentary() {
                        if comm_reader.is_footnotes() {
                            let notes = comm_reader.commentaries_for_verse(
                                parsed.book,
                                parsed.chapter,
                                v,
                            )?;
                            for n in notes {
                                let note_text = render_to_terminal(&n.text, use_color);
                                let prefix = format!(
                                    "      ↳ Footnote {}:",
                                    n.marker.as_deref().unwrap_or("")
                                );
                                if use_color {
                                    println!("{} {}", prefix.dimmed().yellow(), note_text.dimmed());
                                } else {
                                    println!("{} {}", prefix, note_text);
                                }
                            }
                        }
                    }
                }

                // Inline Cross References
                if show_xrefs {
                    if let Some(xref_name) = &engine.active_cross_references {
                        if let Some(xref_reader) = engine.cross_references.get(xref_name) {
                            let refs =
                                xref_reader.references_for_verse(parsed.book, parsed.chapter, v)?;
                            if !refs.is_empty() {
                                let ref_summaries: Vec<String> =
                                    refs.iter().take(5).map(|r| r.format_target()).collect();
                                let xref_line =
                                    format!("      ↳ Cross-refs: {}", ref_summaries.join(", "));
                                if use_color {
                                    println!("{}", xref_line.dimmed().cyan());
                                } else {
                                    println!("{}", xref_line);
                                }
                            }
                        }
                    }
                }
            }
        }

        println!();
        Ok(())
    }

    // --- BOOKS COMMAND ---
    fn handle_books(&self, bible_arg: Option<&str>, use_color: bool) -> Result<()> {
        let engine = self.init_engine()?;
        let bible_key = bible_arg
            .map(|s| s.to_string())
            .or_else(|| self.config.default_bible.clone())
            .or_else(|| engine.active_bible.clone())
            .or_else(|| engine.bibles.keys().next().cloned())
            .ok_or_else(|| MyBibleError::InvalidFormat("No Bible module loaded".to_string()))?;

        let bible = engine.get_bible(&bible_key).ok_or_else(|| {
            MyBibleError::InvalidFormat(format!("Bible '{}' not found", bible_key))
        })?;

        let books = bible.present_books()?;

        #[derive(serde::Serialize)]
        struct BookDto {
            book_number: u16,
            short_name: String,
            long_name: String,
            testament: String,
            chapter_count: u32,
        }

        let mut book_dtos = Vec::new();
        for b in &books {
            let canon = crate::books::find_book_by_number(b.book_number);
            let testament = canon
                .map(|c| format!("{:?}", c.testament))
                .unwrap_or_else(|| "Unknown".to_string());
            let ch_count = bible.chapter_count(b.book_number).unwrap_or(1);
            book_dtos.push(BookDto {
                book_number: b.book_number.0,
                short_name: b.short_name.clone(),
                long_name: b.display_title().to_string(),
                testament,
                chapter_count: ch_count,
            });
        }

        if self.cli.json {
            println!("{}", serde_json::to_string_pretty(&book_dtos)?);
            return Ok(());
        }

        let header = format!("📚 Books in {} ({} books):", bible_key, book_dtos.len());
        if use_color {
            println!("\n{}\n", header.bright_cyan().bold());
        } else {
            println!("\n{}\n", header);
        }

        for b in &book_dtos {
            if use_color {
                println!(
                    "  {:>3}: {:<16} ({:<5}) - {} chapters [{}]",
                    b.book_number,
                    b.long_name.bright_yellow(),
                    b.short_name,
                    b.chapter_count,
                    b.testament.dimmed()
                );
            } else {
                println!(
                    "  {:>3}: {:<16} ({:<5}) - {} chapters [{}]",
                    b.book_number, b.long_name, b.short_name, b.chapter_count, b.testament
                );
            }
        }
        println!();
        Ok(())
    }

    // --- SEARCH COMMAND ---
    #[allow(clippy::too_many_arguments)]
    fn handle_search(
        &self,
        query_str: &str,
        bible_arg: Option<&str>,
        search_all: bool,
        book_filter: Option<&str>,
        testament_filter: Option<&str>,
        mode_str: &str,
        limit: usize,
        case_sensitive: bool,
        match_accents: bool,
        use_color: bool,
    ) -> Result<()> {
        let engine = self.init_engine()?;

        // 1. Resolve search mode
        let mode = match mode_str.to_lowercase().as_str() {
            "phrase" | "exact" => SearchMode::Phrase,
            "any" | "or" => SearchMode::AnyWord,
            _ => SearchMode::AllWords,
        };

        // 2. Resolve scope
        let scope = if let Some(b_str) = book_filter {
            let (book_num, _) = crate::parsers::reference::resolve_book(b_str)?;
            SearchScope::Book(book_num)
        } else if let Some(t_str) = testament_filter {
            match t_str.to_lowercase().as_str() {
                "ot" | "old" => SearchScope::OldTestament,
                "nt" | "new" => SearchScope::NewTestament,
                other => {
                    return Err(MyBibleError::InvalidFormat(format!(
                        "Invalid testament '{}'. Expected 'ot' or 'nt'",
                        other
                    )));
                }
            }
        } else {
            SearchScope::All
        };

        // 3. Construct SearchQuery
        let mut query = SearchQuery::new(query_str)
            .with_mode(mode)
            .with_scope(scope)
            .with_case_sensitive(case_sensitive)
            .with_ignore_accents(!match_accents);

        if limit > 0 {
            query = query.with_limit(Some(limit));
        } else {
            query = query.with_limit(None);
        }

        // 4. Resolve target modules
        let modules_to_search: Vec<&str> = if search_all {
            vec!["all"]
        } else if let Some(b_arg) = bible_arg {
            b_arg
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![]
        };

        // 5. Execute search
        let results = engine.search(&query, &modules_to_search)?;

        if self.cli.json {
            println!("{}", serde_json::to_string_pretty(&results)?);
            return Ok(());
        }

        // 6. Handle no results
        if results.total_matches == 0 {
            let msg = format!("No verses matched query: \"{}\"", query.text);
            if use_color {
                println!("\n{}\n", msg.bright_yellow());
            } else {
                println!("\n{}\n", msg);
            }
            return Ok(());
        }

        // 7. Print summary header
        let modules_count = results.matches_by_module.len();
        let summary_str = if modules_count > 1 {
            let mut breakdown_items: Vec<_> = results.matches_by_module.iter().collect();
            breakdown_items.sort_by_key(|&(k, _)| k);
            let breakdown = breakdown_items
                .into_iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "🔍 Found {} matching verses for \"{}\" across {} Bible modules ({}) in {}ms:",
                results.total_matches, query.text, modules_count, breakdown, results.duration_ms
            )
        } else {
            let mod_name = results
                .matches_by_module
                .keys()
                .next()
                .map(|s| s.as_str())
                .unwrap_or("");
            format!(
                "🔍 Found {} matching verses for \"{}\" in {} in {}ms:",
                results.total_matches, query.text, mod_name, results.duration_ms
            )
        };

        if use_color {
            println!("\n{}\n", summary_str.bright_cyan().bold());
        } else {
            println!("\n{}\n", summary_str);
        }

        // 8. Print matched verses
        let is_multi = modules_count > 1;
        for hit in &results.hits {
            let ref_str = if is_multi {
                format!("📖 {} [{}]", hit.format_reference(), hit.bible_module)
            } else {
                format!("📖 {} ({})", hit.format_reference(), hit.bible_module)
            };

            if use_color {
                println!("  {}", ref_str.bright_yellow().bold());
            } else {
                println!("  {}", ref_str);
            }

            let rendered = hit.render_highlighted(use_color);
            for line in rendered.lines() {
                println!("     {}", line.trim_end());
            }
            println!();
        }

        // 9. Truncation notice if limit reached
        if let Some(_limit_val) = query.limit {
            if results.total_matches > results.hits.len() {
                let note = format!(
                    "ℹ️  Showing first {} of {} matching verses. Use --limit 0 to display all results.",
                    results.hits.len(),
                    results.total_matches
                );
                if use_color {
                    println!("  {}\n", note.dimmed().cyan());
                } else {
                    println!("  {}\n", note);
                }
            }
        }

        Ok(())
    }

    // --- COMMENTARY COMMAND ---
    fn handle_commentary(
        &self,
        reference_str: &str,
        module_arg: Option<&str>,
        use_color: bool,
    ) -> Result<()> {
        let engine = self.init_engine()?;
        let parsed = parse_passage_reference(reference_str)?;

        let comm_key = module_arg
            .map(|s| s.to_string())
            .or_else(|| self.config.default_commentary.clone())
            .or_else(|| engine.active_commentary.clone())
            .or_else(|| engine.commentaries.keys().next().cloned())
            .ok_or_else(|| {
                MyBibleError::InvalidFormat(
                    "No commentary modules installed. Run 'mybible modules download <abbrev>' to install one."
                        .to_string(),
                )
            })?;

        let comm = engine
            .commentaries
            .get(&comm_key)
            .ok_or_else(|| MyBibleError::ModuleNotFound(comm_key.clone()))?;

        let notes = match parsed.verse_start {
            Some(v) => comm.commentaries_for_verse(parsed.book, parsed.chapter, v)?,
            None => comm.commentaries_for_chapter(parsed.book, parsed.chapter)?,
        };

        if self.cli.json {
            #[derive(serde::Serialize)]
            struct CommItemDto {
                chapter_from: u32,
                verse_from: u32,
                chapter_to: u32,
                verse_to: u32,
                text: String,
                plain_text: String,
            }

            #[derive(serde::Serialize)]
            struct CommJson {
                reference: String,
                module: String,
                notes: Vec<CommItemDto>,
            }

            let dtos = notes
                .iter()
                .map(|n| CommItemDto {
                    chapter_from: n.chapter_number_from,
                    verse_from: n.verse_number_from,
                    chapter_to: n.chapter_number_to.unwrap_or(n.chapter_number_from),
                    verse_to: n.verse_number_to.unwrap_or(n.verse_number_from),
                    text: n.text.clone(),
                    plain_text: crate::terminal::render_plain_text(&n.text),
                })
                .collect();

            let resp = CommJson {
                reference: parsed.display_string(),
                module: comm_key,
                notes: dtos,
            };
            println!("{}", serde_json::to_string_pretty(&resp)?);
            return Ok(());
        }

        let header = format!(
            "📝 Commentary on {} ({})",
            parsed.display_string(),
            comm_key
        );

        if use_color {
            println!("\n{}\n", header.bright_magenta().bold());
        } else {
            println!("\n{}\n", header);
        }

        if notes.is_empty() {
            println!("No commentary notes found for this passage.");
            return Ok(());
        }

        for (idx, note) in notes.iter().enumerate() {
            let range_str = format!(
                "Note {} [Chapter {}:{} - {}:{}]",
                idx + 1,
                note.chapter_number_from,
                note.verse_number_from,
                note.chapter_number_to.unwrap_or(note.chapter_number_from),
                note.verse_number_to.unwrap_or(note.verse_number_from)
            );

            if use_color {
                println!("--- {} ---", range_str.bright_cyan().bold());
            } else {
                println!("--- {} ---", range_str);
            }

            let rendered = render_to_terminal(&note.text, use_color);
            println!("{}\n", rendered);
        }

        Ok(())
    }

    // --- DICTIONARY COMMAND ---
    fn handle_dict(
        &self,
        term: &str,
        module_arg: Option<&str>,
        suggest: bool,
        use_color: bool,
    ) -> Result<()> {
        let engine = self.init_engine()?;

        let dict_key = module_arg
            .map(|s| s.to_string())
            .or_else(|| self.config.default_dictionary.clone())
            .or_else(|| engine.dictionaries.keys().next().cloned())
            .ok_or_else(|| {
                MyBibleError::InvalidFormat(
                    "No dictionary modules installed. Run 'mybible modules download <abbrev>' to install one."
                        .to_string(),
                )
            })?;

        let dict = engine
            .dictionaries
            .get(&dict_key)
            .ok_or_else(|| MyBibleError::ModuleNotFound(dict_key.clone()))?;

        if self.cli.json {
            #[derive(serde::Serialize)]
            struct DictJson {
                term: String,
                module: String,
                found: bool,
                topic: Option<String>,
                definition: Option<String>,
                plain_definition: Option<String>,
                suggestions: Vec<String>,
            }

            if let Some(topic_rec) = dict.get_topic(term)? {
                let resp = DictJson {
                    term: term.to_string(),
                    module: dict_key,
                    found: true,
                    topic: Some(topic_rec.topic),
                    definition: Some(topic_rec.definition.clone()),
                    plain_definition: Some(crate::terminal::render_plain_text(
                        &topic_rec.definition,
                    )),
                    suggestions: Vec::new(),
                };
                println!("{}", serde_json::to_string_pretty(&resp)?);
                return Ok(());
            }

            let prefix = if term.len() >= 3 { &term[..3] } else { term };
            let suggestions = if suggest {
                dict.search_topics_prefix(prefix, 10).unwrap_or_default()
            } else {
                Vec::new()
            };

            let resp = DictJson {
                term: term.to_string(),
                module: dict_key,
                found: false,
                topic: None,
                definition: None,
                plain_definition: None,
                suggestions,
            };
            println!("{}", serde_json::to_string_pretty(&resp)?);
            return Ok(());
        }

        // 1. Try exact lookup
        if let Some(topic_rec) = dict.get_topic(term)? {
            let header = format!("📚 {} — Dictionary ({})", topic_rec.topic, dict_key);
            if use_color {
                println!("\n{}\n", header.bright_green().bold());
            } else {
                println!("\n{}\n", header);
            }

            let rendered = render_to_terminal(&topic_rec.definition, use_color);
            println!("{}\n", rendered);
            return Ok(());
        }

        // 2. Not found: suggest topics with same prefix
        println!("Topic '{}' not found in dictionary '{}'.", term, dict_key);

        if suggest {
            let prefix = if term.len() >= 3 { &term[..3] } else { term };
            let suggestions = dict.search_topics_prefix(prefix, 10)?;
            if !suggestions.is_empty() {
                println!("\nSimilar topics starting with '{}':", prefix);
                for s in suggestions {
                    if use_color {
                        println!("  • {}", s.bright_yellow());
                    } else {
                        println!("  • {}", s);
                    }
                }
            }
        }

        Ok(())
    }

    // --- CROSS REFERENCES COMMAND ---
    fn handle_xref(&self, reference_str: &str, preview_text: bool, use_color: bool) -> Result<()> {
        let engine = self.init_engine()?;
        let parsed = parse_passage_reference(reference_str)?;
        let verse = parsed.verse_start.unwrap_or(1);

        let xref_key = engine
            .active_cross_references
            .clone()
            .or_else(|| engine.cross_references.keys().next().cloned())
            .ok_or_else(|| {
                MyBibleError::InvalidFormat(
                    "No cross-reference modules installed (e.g. OBX). Run 'mybible modules download OBX' to install."
                        .to_string(),
                )
            })?;

        let xref_reader = engine
            .cross_references
            .get(&xref_key)
            .ok_or_else(|| MyBibleError::ModuleNotFound(xref_key.clone()))?;

        let refs = xref_reader.references_for_verse(parsed.book, parsed.chapter, verse)?;

        if self.cli.json {
            #[derive(serde::Serialize)]
            struct XrefItemDto {
                target: String,
                book: u16,
                chapter: u32,
                verse: u32,
                votes: Option<i32>,
                preview: Option<String>,
            }

            #[derive(serde::Serialize)]
            struct XrefJson {
                reference: String,
                module: String,
                references: Vec<XrefItemDto>,
            }

            let active_bible = engine.get_active_bible();
            let mut dtos = Vec::new();
            for r in &refs {
                let mut preview = None;
                if let Some(b) = &active_bible {
                    if let Ok(Some(verse_rec)) = b.verse(r.book_to, r.chapter_to, r.verse_to_start)
                    {
                        let plain = crate::parsers::verse_text::nodes_to_plain_text(
                            &crate::parsers::verse_text::parse_verse_text(&verse_rec.text),
                        );
                        preview = Some(plain.trim().replace('\n', " "));
                    }
                }
                dtos.push(XrefItemDto {
                    target: r.format_target(),
                    book: r.book_to.0,
                    chapter: r.chapter_to,
                    verse: r.verse_to_start,
                    votes: r.votes,
                    preview,
                });
            }

            let resp = XrefJson {
                reference: format!("{} {}:{}", parsed.book_name, parsed.chapter, verse),
                module: xref_key,
                references: dtos,
            };
            println!("{}", serde_json::to_string_pretty(&resp)?);
            return Ok(());
        }

        let header = format!(
            "🔗 Cross References for {} {}:{} ({})",
            parsed.book_name, parsed.chapter, verse, xref_key
        );
        if use_color {
            println!("\n{}\n", header.bright_blue().bold());
        } else {
            println!("\n{}\n", header);
        }

        if refs.is_empty() {
            println!("No cross-references found for this verse.");
            return Ok(());
        }

        let active_bible = engine.get_active_bible();

        for (idx, r) in refs.iter().enumerate() {
            let votes_str = match r.votes {
                Some(v) => format!(" ({} votes)", v),
                None => String::new(),
            };

            let target_str = format!("{:>2}. {}{}", idx + 1, r.format_target(), votes_str);
            if use_color {
                print!("{}", target_str.bright_cyan().bold());
            } else {
                print!("{}", target_str);
            }

            if preview_text {
                if let Some(b) = &active_bible {
                    if let Ok(Some(verse_rec)) = b.verse(r.book_to, r.chapter_to, r.verse_to_start)
                    {
                        let plain = crate::parsers::verse_text::nodes_to_plain_text(
                            &crate::parsers::verse_text::parse_verse_text(&verse_rec.text),
                        );
                        let clean = plain.trim().replace('\n', " ");
                        let preview_snippet = if clean.len() > 80 {
                            format!("{}...", &clean[..80])
                        } else {
                            clean
                        };
                        if use_color {
                            println!(" — {}", preview_snippet.dimmed());
                        } else {
                            println!(" — {}", preview_snippet);
                        }
                        continue;
                    }
                }
            }
            println!();
        }

        println!();
        Ok(())
    }

    // --- READING PLAN COMMAND ---
    fn handle_plan(&self, plan_abbrev: &str, day: u32, use_color: bool) -> Result<()> {
        let engine = self.init_engine()?;

        let plan_reader = engine
            .reading_plans
            .get(plan_abbrev)
            .ok_or_else(|| MyBibleError::ModuleNotFound(plan_abbrev.to_string()))?;

        let max_days = plan_reader.max_days()?;
        let header = format!(
            "📅 Reading Plan: {} (Day {} of {})",
            plan_abbrev, day, max_days
        );

        if use_color {
            println!("\n{}\n", header.bright_yellow().bold());
        } else {
            println!("\n{}\n", header);
        }

        let items = plan_reader.items_for_day(day)?;
        if items.is_empty() {
            println!("No reading items scheduled for day {}.", day);
            return Ok(());
        }

        for (idx, item) in items.iter().enumerate() {
            let book_name = crate::books::find_book_by_number(item.book_number)
                .map(|b| b.short_name_eng)
                .unwrap_or("Book");
            let range = match (item.start_verse, item.end_verse) {
                (Some(sv), Some(ev)) if item.start_chapter == item.end_chapter => {
                    format!("{}:{}-{}", item.start_chapter, sv, ev)
                }
                (Some(sv), Some(ev)) => format!(
                    "{}:{} - {}:{}",
                    item.start_chapter, sv, item.end_chapter, ev
                ),
                (Some(sv), None) => format!("{}:{}", item.start_chapter, sv),
                (None, _) => format!("Chapter {}", item.start_chapter),
            };
            let time_str = if item.evening { " (Evening)" } else { "" };
            if use_color {
                println!(
                    "  {}. {} {}{}",
                    idx + 1,
                    book_name.bright_cyan().bold(),
                    range,
                    time_str.dimmed()
                );
            } else {
                println!("  {}. {} {}{}", idx + 1, book_name, range, time_str);
            }
        }
        println!();
        Ok(())
    }

    // --- DEVOTION COMMAND ---
    fn handle_devotion(&self, module_arg: Option<&str>, day: u32, use_color: bool) -> Result<()> {
        let engine = self.init_engine()?;

        let dev_key = module_arg
            .map(|s| s.to_string())
            .or_else(|| engine.devotions.keys().next().cloned())
            .ok_or_else(|| {
                MyBibleError::InvalidFormat(
                    "No daily devotional modules installed. Run 'mybible modules download <abbrev>' to install one."
                        .to_string(),
                )
            })?;

        let dev_reader = engine
            .devotions
            .get(&dev_key)
            .ok_or_else(|| MyBibleError::ModuleNotFound(dev_key.clone()))?;

        if let Some(devotion_rec) = dev_reader.devotion_for_day(day)? {
            let header = format!("🌅 Daily Devotional — Day {} ({})", day, dev_key);

            if use_color {
                println!("\n{}\n", header.bright_magenta().bold());
            } else {
                println!("\n{}\n", header);
            }

            let rendered = render_to_terminal(&devotion_rec.devotion, use_color);
            println!("{}\n", rendered);
        } else {
            println!("No devotion found for day {} in module {}.", day, dev_key);
        }

        Ok(())
    }

    // --- MODULES COMMANDS ---
    fn handle_modules(&self, cmd: &ModulesCommands, use_color: bool) -> Result<()> {
        match cmd {
            ModulesCommands::List => {
                let engine = self.init_engine()?;

                if self.cli.json {
                    let mut b_list: Vec<_> = engine.bibles.keys().cloned().collect();
                    b_list.sort();
                    let mut c_list: Vec<_> = engine.commentaries.keys().cloned().collect();
                    c_list.sort();
                    let mut d_list: Vec<_> = engine.dictionaries.keys().cloned().collect();
                    d_list.sort();
                    let mut x_list: Vec<_> = engine.cross_references.keys().cloned().collect();
                    x_list.sort();
                    let mut s_list: Vec<_> = engine.subheadings.keys().cloned().collect();
                    s_list.sort();
                    let mut p_list: Vec<_> = engine.reading_plans.keys().cloned().collect();
                    p_list.sort();
                    let mut dev_list: Vec<_> = engine.devotions.keys().cloned().collect();
                    dev_list.sort();

                    #[derive(serde::Serialize)]
                    struct ModulesJson {
                        data_dir: String,
                        total_modules: usize,
                        bibles: Vec<String>,
                        commentaries: Vec<String>,
                        dictionaries: Vec<String>,
                        cross_references: Vec<String>,
                        subheadings: Vec<String>,
                        reading_plans: Vec<String>,
                        devotions: Vec<String>,
                    }

                    let resp = ModulesJson {
                        data_dir: self.data_dir.display().to_string(),
                        total_modules: engine.total_modules(),
                        bibles: b_list,
                        commentaries: c_list,
                        dictionaries: d_list,
                        cross_references: x_list,
                        subheadings: s_list,
                        reading_plans: p_list,
                        devotions: dev_list,
                    };
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                    return Ok(());
                }

                println!("\n📁 Active Data Directory: {}", self.data_dir.display());
                println!("==================================================");

                print_module_group("Bibles:", &engine.bibles, use_color);
                print_module_group("Commentaries & Footnotes:", &engine.commentaries, use_color);
                print_module_group("Dictionaries:", &engine.dictionaries, use_color);
                print_module_group("Cross References:", &engine.cross_references, use_color);
                print_module_group("Subheadings:", &engine.subheadings, use_color);
                print_module_group("Reading Plans:", &engine.reading_plans, use_color);
                print_module_group("Devotionals:", &engine.devotions, use_color);

                println!("\nTotal modules loaded: {}\n", engine.total_modules());
                Ok(())
            }

            ModulesCommands::Search { query, language } => {
                let client = RepositoryClient::new();
                let registry = client.fetch_official_registry()?;

                let mut results: Vec<_> = registry.search(query);
                if let Some(lang) = language {
                    results.retain(|m| m.language.as_deref() == Some(lang));
                }

                if self.cli.json {
                    #[derive(serde::Serialize)]
                    struct RepoItemDto {
                        abbreviation: String,
                        module_type: String,
                        language: Option<String>,
                        size_str: Option<String>,
                        description: String,
                    }

                    let dtos: Vec<_> = results
                        .iter()
                        .map(|m| RepoItemDto {
                            abbreviation: m.abbreviation.clone(),
                            module_type: format!("{:?}", m.module_type()),
                            language: m.language.clone(),
                            size_str: m.size_str.clone(),
                            description: m.description.clone(),
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&dtos)?);
                    return Ok(());
                }

                println!("Fetching official module registry from repository...");

                println!("\nFound {} modules matching '{}':\n", results.len(), query);
                println!(
                    "{:<16} {:<8} {:<12} {:<8} DESCRIPTION",
                    "ABBREVIATION", "TYPE", "LANGUAGE", "SIZE"
                );
                println!("{}", "-".repeat(80));

                for m in results.iter().take(40) {
                    let type_str = format!("{:?}", m.module_type());
                    let lang_str = m.language.as_deref().unwrap_or("-");
                    let size_str = m.size_str.as_deref().unwrap_or("-");
                    let desc = if m.description.len() > 40 {
                        format!("{}...", &m.description[..37])
                    } else {
                        m.description.clone()
                    };

                    if use_color {
                        println!(
                            "{:<16} {:<8} {:<12} {:<8} {}",
                            m.abbreviation.bright_yellow().bold(),
                            type_str.bright_cyan(),
                            lang_str,
                            size_str.dimmed(),
                            desc
                        );
                    } else {
                        println!(
                            "{:<16} {:<8} {:<12} {:<8} {}",
                            m.abbreviation, type_str, lang_str, size_str, desc
                        );
                    }
                }

                if results.len() > 40 {
                    println!(
                        "\n... and {} more modules. Refine search query for more specific results.",
                        results.len() - 40
                    );
                }
                println!();
                Ok(())
            }

            ModulesCommands::Download { abbreviation } => {
                let client = RepositoryClient::new();
                println!("Connecting to repository...");
                let registry = client.fetch_official_registry()?;

                let entry = registry
                    .find_by_abbrev(abbreviation, None)
                    .ok_or_else(|| {
                        MyBibleError::InvalidFormat(format!(
                            "Module '{}' not found in online catalog. Try 'mybible modules search <name>'.",
                            abbreviation
                        ))
                    })?;

                println!(
                    "Downloading '{}' ({}) - {} ...",
                    entry.abbreviation,
                    entry.size_str.as_deref().unwrap_or("?"),
                    entry.description
                );

                let extracted_path = client.download_module(entry, &registry, &self.data_dir)?;
                println!(
                    "{}",
                    format!(
                        "✓ Successfully installed '{}' to {}",
                        entry.abbreviation,
                        extracted_path.display()
                    )
                    .bright_green()
                    .bold()
                );
                Ok(())
            }

            ModulesCommands::Delete { abbreviation } => {
                let mut found = false;
                for entry in fs::read_dir(&self.data_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        if file_name.starts_with(abbreviation) {
                            fs::remove_file(&path)?;
                            println!("✓ Deleted module file: {}", path.display());
                            found = true;
                        }
                    }
                }
                if !found {
                    println!(
                        "Module '{}' not found in {}",
                        abbreviation,
                        self.data_dir.display()
                    );
                }
                Ok(())
            }

            ModulesCommands::Info { abbreviation } => {
                let engine = self.init_engine()?;
                println!("\nMetadata for module '{}':", abbreviation);
                println!("----------------------------------------");

                if let Some(b) = engine.bibles.get(abbreviation) {
                    println!("Type: Bible");
                    println!("Description: {:?}", b.info().description());
                    println!("Language: {:?}", b.info().language());
                    println!("Total Books: {}", b.books()?.len());
                } else if let Some(c) = engine.commentaries.get(abbreviation) {
                    println!("Type: Commentary");
                    println!("Description: {:?}", c.info().description());
                    println!("Is Footnotes: {}", c.is_footnotes());
                } else if let Some(d) = engine.dictionaries.get(abbreviation) {
                    println!("Type: Dictionary");
                    println!("Description: {:?}", d.info().description());
                    println!("Dictionary Type: {:?}", d.dictionary_type());
                } else {
                    println!("Module '{}' is not loaded or not found.", abbreviation);
                }
                println!();
                Ok(())
            }
        }
    }

    // --- CONFIG COMMANDS ---
    fn handle_config(&mut self, cmd: &ConfigCommands, _use_color: bool) -> Result<()> {
        match cmd {
            ConfigCommands::Get => {
                println!("\n⚙️ MyBible CLI Configuration");
                println!("================================");
                println!("Data Directory:      {}", self.data_dir.display());
                println!(
                    "Config File:         {}",
                    CliConfig::config_file_path().unwrap_or_default().display()
                );
                println!(
                    "Default Bible:       {}",
                    self.config.default_bible.as_deref().unwrap_or("<none>")
                );
                println!(
                    "Default Commentary:  {}",
                    self.config
                        .default_commentary
                        .as_deref()
                        .unwrap_or("<none>")
                );
                println!(
                    "Default Dictionary:  {}",
                    self.config
                        .default_dictionary
                        .as_deref()
                        .unwrap_or("<none>")
                );
                println!(
                    "Default Subheadings: {}",
                    self.config
                        .default_subheadings
                        .as_deref()
                        .unwrap_or("<none>")
                );
                println!();
                Ok(())
            }

            ConfigCommands::SetDir { path } => {
                self.config.data_dir = Some(path.display().to_string());
                self.config.save()?;
                println!("✓ Updated default data directory to: {}", path.display());
                Ok(())
            }

            ConfigCommands::SetBible { abbreviation } => {
                self.config.default_bible = Some(abbreviation.clone());
                self.config.save()?;
                println!("✓ Default Bible set to '{}'", abbreviation);
                Ok(())
            }

            ConfigCommands::SetCommentary { abbreviation } => {
                self.config.default_commentary = Some(abbreviation.clone());
                self.config.save()?;
                println!("✓ Default Commentary set to '{}'", abbreviation);
                Ok(())
            }

            ConfigCommands::SetDictionary { abbreviation } => {
                self.config.default_dictionary = Some(abbreviation.clone());
                self.config.save()?;
                println!("✓ Default Dictionary set to '{}'", abbreviation);
                Ok(())
            }

            ConfigCommands::SetSubheadings { abbreviation } => {
                self.config.default_subheadings = Some(abbreviation.clone());
                self.config.save()?;
                println!("✓ Default Subheadings set to '{}'", abbreviation);
                Ok(())
            }
        }
    }
}

fn print_module_group<V>(
    title: &str,
    items: &std::collections::HashMap<String, V>,
    use_color: bool,
) {
    if items.is_empty() {
        return;
    }
    if use_color {
        println!("\n{}", title.bright_cyan().bold());
    } else {
        println!("\n{}", title);
    }
    let mut names: Vec<_> = items.keys().collect();
    names.sort();
    for k in names {
        println!("  • {}", k);
    }
}
