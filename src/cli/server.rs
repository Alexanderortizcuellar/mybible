use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::MyBibleEngine;
use crate::error::Result;
use crate::parsers::reference::parse_passage_reference;
use crate::repository::{ModuleRegistry, RepositoryClient};
use crate::search::types::{SearchMode, SearchQuery, SearchResults, SearchScope};

#[derive(Deserialize, Debug)]
struct Request {
    pub id: Option<u64>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Serialize)]
struct Response<'a, T: Serialize> {
    pub id: Option<u64>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'a str>,
}

pub struct InteractiveServer {
    data_dir: PathBuf,
    engine: MyBibleEngine,
    last_search: Option<SearchResults>,
    registry_cache: Option<ModuleRegistry>,
}

impl InteractiveServer {
    pub fn new(data_dir: PathBuf) -> Self {
        let mut engine = MyBibleEngine::new();
        let _ = engine.load_directory(&data_dir);

        // Auto select first available active modules if not explicitly set
        if engine.active_bible.is_none() {
            engine.active_bible = engine.bibles.keys().next().cloned();
        }
        if engine.active_commentary.is_none() {
            engine.active_commentary = engine.commentaries.keys().next().cloned();
        }
        if engine.active_cross_references.is_none() {
            engine.active_cross_references = engine.cross_references.keys().next().cloned();
        }
        if engine.active_subheadings.is_none() {
            engine.active_subheadings = engine.subheadings.keys().next().cloned();
        }

        Self {
            data_dir,
            engine,
            last_search: None,
            registry_cache: None,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        for line_res in stdin.lock().lines() {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => break, // EOF or stream closed
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let req: Request = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    let err_msg = format!("Malformed JSON request: {}", e);
                    let resp: Response<()> = Response {
                        id: None,
                        success: false,
                        data: None,
                        error: Some(&err_msg),
                    };
                    let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                    let _ = stdout.flush();
                    continue;
                }
            };

            let res_json = self.dispatch(&req);
            let _ = writeln!(stdout, "{}", res_json);
            let _ = stdout.flush();
        }

        Ok(())
    }

    fn dispatch(&mut self, req: &Request) -> String {
        let id = req.id;
        match req.method.as_str() {
            "ping" => self.resp_ok(id, serde_json::json!({"pong": true})),
            "status" => self.handle_status(id),
            "books" => self.handle_books(id, &req.params),
            "read" => self.handle_read(id, &req.params),
            "compare" => self.handle_compare(id, &req.params),
            "preview" => self.handle_preview(id, &req.params),
            "commentary" => self.handle_commentary(id, &req.params),
            "dict" => self.handle_dict(id, &req.params),
            "xref" => self.handle_xref(id, &req.params),
            "search" => self.handle_search(id, &req.params),
            "last_search" => self.handle_last_search(id),
            "catalog" => self.handle_catalog(id, &req.params),
            "download" => self.handle_download(id, &req.params),
            "reload" => self.handle_reload(id),
            unknown => self.resp_err(id, &format!("Unknown method '{}'", unknown)),
        }
    }

    fn resp_ok<T: Serialize>(&self, id: Option<u64>, data: T) -> String {
        let resp = Response {
            id,
            success: true,
            data: Some(data),
            error: None,
        };
        serde_json::to_string(&resp)
            .unwrap_or_else(|e| format!(r#"{{"id":{:?},"success":false,"error":"{}"}}"#, id, e))
    }

    fn resp_err(&self, id: Option<u64>, err: &str) -> String {
        let resp: Response<()> = Response {
            id,
            success: false,
            data: None,
            error: Some(err),
        };
        serde_json::to_string(&resp).unwrap_or_else(|_| {
            format!(
                r#"{{"id":{:?},"success":false,"error":"JSON serialization failure"}}"#,
                id
            )
        })
    }

    // --- RPC Handlers ---

    fn handle_status(&self, id: Option<u64>) -> String {
        let mut b_list: Vec<_> = self.engine.bibles.keys().cloned().collect();
        b_list.sort();
        let mut c_list: Vec<_> = self.engine.commentaries.keys().cloned().collect();
        c_list.sort();
        let mut d_list: Vec<_> = self.engine.dictionaries.keys().cloned().collect();
        d_list.sort();
        let mut x_list: Vec<_> = self.engine.cross_references.keys().cloned().collect();
        x_list.sort();
        let mut s_list: Vec<_> = self.engine.subheadings.keys().cloned().collect();
        s_list.sort();
        let mut p_list: Vec<_> = self.engine.reading_plans.keys().cloned().collect();
        p_list.sort();
        let mut dev_list: Vec<_> = self.engine.devotions.keys().cloned().collect();
        dev_list.sort();

        let val = serde_json::json!({
            "data_dir": self.data_dir.display().to_string(),
            "total_modules": self.engine.total_modules(),
            "bibles": b_list,
            "commentaries": c_list,
            "dictionaries": d_list,
            "cross_references": x_list,
            "subheadings": s_list,
            "reading_plans": p_list,
            "devotions": dev_list,
        });
        self.resp_ok(id, val)
    }

    fn handle_books(&self, id: Option<u64>, params: &Value) -> String {
        let bible_key = params
            .get("bible")
            .and_then(|v| v.as_str())
            .or(self.engine.active_bible.as_deref())
            .or_else(|| self.engine.bibles.keys().next().map(|s| s.as_str()));

        let b_key = match bible_key {
            Some(k) => k,
            None => return self.resp_err(id, "No Bible modules loaded"),
        };

        let bible = match self.engine.bibles.get(b_key) {
            Some(b) => b,
            None => return self.resp_err(id, &format!("Bible module '{}' not found", b_key)),
        };

        let books = match bible.books() {
            Ok(b) => b,
            Err(e) => return self.resp_err(id, &e.to_string()),
        };

        let mut book_dtos = Vec::new();
        for b in books {
            let max_ch = bible.chapter_count(b.book_number).unwrap_or(1);
            let is_ot = b.book_number.0 < 400;
            book_dtos.push(serde_json::json!({
                "book_number": b.book_number.0,
                "short_name": b.short_name,
                "long_name": b.long_name,
                "testament": if is_ot { "ot" } else { "nt" },
                "chapter_count": max_ch,
            }));
        }

        self.resp_ok(
            id,
            serde_json::json!({
                "bible": b_key,
                "books": book_dtos,
            }),
        )
    }

    fn handle_read(&self, id: Option<u64>, params: &Value) -> String {
        let ref_str = match params.get("ref").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => return self.resp_err(id, "Missing 'ref' parameter"),
        };

        let parsed = match parse_passage_reference(ref_str) {
            Ok(p) => p,
            Err(e) => return self.resp_err(id, &e.to_string()),
        };

        let bible_key = params
            .get("bible")
            .and_then(|v| v.as_str())
            .or(self.engine.active_bible.as_deref())
            .or_else(|| self.engine.bibles.keys().next().map(|s| s.as_str()));

        let b_key = match bible_key {
            Some(k) => k,
            None => return self.resp_err(id, "No Bible modules loaded"),
        };

        let bible = match self.engine.bibles.get(b_key) {
            Some(b) => b,
            None => return self.resp_err(id, &format!("Bible module '{}' not found", b_key)),
        };

        let v_start = parsed.verse_start.unwrap_or(1);
        let v_end = parsed.verse_end.unwrap_or_else(|| {
            bible
                .verse_count(parsed.book, parsed.chapter)
                .unwrap_or(v_start)
        });

        let show_subheadings = params
            .get("subheadings")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let show_footnotes = params
            .get("footnotes")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let show_xrefs = params
            .get("xrefs")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let sub_override = params.get("subheadings_module").and_then(|v| v.as_str());

        let mut json_subheadings = Vec::new();
        let mut json_verses = Vec::new();

        for v in v_start..=v_end {
            if show_subheadings {
                let mut subs = Vec::new();
                let sub_key = sub_override.or(self.engine.active_subheadings.as_deref());
                if let Some(s_key) = sub_key {
                    if let Some(sub_reader) = self.engine.subheadings.get(s_key) {
                        if let Ok(records) =
                            sub_reader.subheadings_for_verse(parsed.book, parsed.chapter, v)
                        {
                            subs = records;
                        }
                    }
                }
                if subs.is_empty() {
                    if let Ok(stories) = bible.stories_for_verse(parsed.book, parsed.chapter, v) {
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
                    json_subheadings.push(serde_json::json!({
                        "verse": v,
                        "title": crate::terminal::render_subheading(&s.subheading, false),
                    }));
                }
            }

            if let Ok(Some(verse_rec)) = bible.verse(parsed.book, parsed.chapter, v) {
                let mut fns = Vec::new();
                if show_footnotes {
                    if let Some(comm_reader) = self.engine.get_active_commentary() {
                        if comm_reader.is_footnotes() {
                            if let Ok(notes) =
                                comm_reader.commentaries_for_verse(parsed.book, parsed.chapter, v)
                            {
                                for n in notes {
                                    fns.push(serde_json::json!({
                                        "marker": n.marker,
                                        "text": crate::terminal::render_plain_text(&n.text),
                                    }));
                                }
                            }
                        }
                    }
                }

                let mut xref_list = Vec::new();
                if show_xrefs {
                    if let Some(xref_name) = &self.engine.active_cross_references {
                        if let Some(xref_reader) = self.engine.cross_references.get(xref_name) {
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

                json_verses.push(serde_json::json!({
                    "verse": v,
                    "text": verse_rec.text,
                    "plain_text": plain,
                    "footnotes": fns,
                    "cross_references": xref_list,
                }));
            }
        }

        let resp = serde_json::json!({
            "reference": parsed.display_string(),
            "book_number": parsed.book.0,
            "book_name": parsed.book_name,
            "chapter": parsed.chapter,
            "verse_start": v_start,
            "verse_end": v_end,
            "bible": b_key,
            "subheadings": json_subheadings,
            "verses": json_verses,
        });

        self.resp_ok(id, resp)
    }

    fn handle_compare(&self, id: Option<u64>, params: &Value) -> String {
        let ref_str = match params.get("ref").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => return self.resp_err(id, "Missing 'ref' parameter"),
        };

        let parsed = match parse_passage_reference(ref_str) {
            Ok(p) => p,
            Err(e) => return self.resp_err(id, &e.to_string()),
        };

        let bibles_req: Vec<String> = params
            .get("bibles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| self.engine.bibles.keys().cloned().collect());

        if bibles_req.is_empty() {
            return self.resp_err(id, "No Bible modules specified for comparison");
        }

        // Determine verse range
        let mut max_v = 1;
        for b_name in &bibles_req {
            if let Some(b) = self.engine.bibles.get(b_name) {
                if let Ok(m) = b.verse_count(parsed.book, parsed.chapter) {
                    if m > max_v {
                        max_v = m;
                    }
                }
            }
        }

        let v_start = parsed.verse_start.unwrap_or(1);
        let v_end = parsed.verse_end.unwrap_or(max_v);

        let mut verses_dto = Vec::new();

        for v in v_start..=v_end {
            let mut trans_dto = Vec::new();
            for b_name in &bibles_req {
                if let Some(b) = self.engine.bibles.get(b_name) {
                    if let Ok(Some(rec)) = b.verse(parsed.book, parsed.chapter, v) {
                        let ast = crate::parsers::verse_text::parse_verse_text(&rec.text);
                        let plain = crate::parsers::verse_text::nodes_to_plain_text(&ast);
                        trans_dto.push(serde_json::json!({
                            "bible": b_name,
                            "text": rec.text,
                            "plain_text": plain.trim(),
                        }));
                    }
                }
            }
            if !trans_dto.is_empty() {
                verses_dto.push(serde_json::json!({
                    "verse": v,
                    "translations": trans_dto,
                }));
            }
        }

        let resp = serde_json::json!({
            "reference": parsed.display_string(),
            "book_number": parsed.book.0,
            "book_name": parsed.book_name,
            "chapter": parsed.chapter,
            "bibles": bibles_req,
            "verses": verses_dto,
        });

        self.resp_ok(id, resp)
    }

    fn handle_preview(&self, id: Option<u64>, params: &Value) -> String {
        let ref_str = match params.get("ref").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => return self.resp_err(id, "Missing 'ref' parameter"),
        };

        let parsed = match parse_passage_reference(ref_str) {
            Ok(p) => p,
            Err(e) => return self.resp_err(id, &e.to_string()),
        };

        let bible_key = params
            .get("bible")
            .and_then(|v| v.as_str())
            .or(self.engine.active_bible.as_deref())
            .or_else(|| self.engine.bibles.keys().next().map(|s| s.as_str()));

        let b_key = match bible_key {
            Some(k) => k,
            None => return self.resp_err(id, "No Bible modules loaded"),
        };

        let bible = match self.engine.bibles.get(b_key) {
            Some(b) => b,
            None => return self.resp_err(id, &format!("Bible module '{}' not found", b_key)),
        };

        let v_start = parsed.verse_start.unwrap_or(1);
        let v_end = parsed.verse_end.unwrap_or(v_start);

        let mut lines = Vec::new();
        for v in v_start..=v_end {
            if let Ok(Some(rec)) = bible.verse(parsed.book, parsed.chapter, v) {
                let ast = crate::parsers::verse_text::parse_verse_text(&rec.text);
                let plain = crate::parsers::verse_text::nodes_to_plain_text(&ast);
                if v_start != v_end {
                    lines.push(format!("{}. {}", v, plain.trim()));
                } else {
                    lines.push(plain.trim().to_string());
                }
            }
        }

        let resp = serde_json::json!({
            "reference": parsed.display_string(),
            "bible": b_key,
            "text": lines.join(" "),
        });

        self.resp_ok(id, resp)
    }

    fn handle_commentary(&self, id: Option<u64>, params: &Value) -> String {
        let ref_str = match params.get("ref").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => return self.resp_err(id, "Missing 'ref' parameter"),
        };

        let parsed = match parse_passage_reference(ref_str) {
            Ok(p) => p,
            Err(e) => return self.resp_err(id, &e.to_string()),
        };

        let comm_key = params
            .get("module")
            .and_then(|v| v.as_str())
            .or(self.engine.active_commentary.as_deref())
            .or_else(|| self.engine.commentaries.keys().next().map(|s| s.as_str()));

        let c_key = match comm_key {
            Some(k) => k,
            None => return self.resp_err(id, "No commentary modules loaded"),
        };

        let comm_reader = match self.engine.commentaries.get(c_key) {
            Some(c) => c,
            None => return self.resp_err(id, &format!("Commentary module '{}' not found", c_key)),
        };

        let notes = if let Some(v) = parsed.verse_start {
            comm_reader
                .commentaries_for_verse(parsed.book, parsed.chapter, v)
                .unwrap_or_default()
        } else {
            comm_reader
                .commentaries_for_chapter(parsed.book, parsed.chapter)
                .unwrap_or_default()
        };

        let mut note_dtos = Vec::new();
        for n in notes {
            note_dtos.push(serde_json::json!({
                "chapter_from": n.chapter_number_from,
                "verse_from": n.verse_number_from,
                "chapter_to": n.chapter_number_to.unwrap_or(n.chapter_number_from),
                "verse_to": n.verse_number_to.unwrap_or(n.verse_number_from),
                "text": n.text,
                "plain_text": crate::terminal::render_plain_text(&n.text),
            }));
        }

        let resp = serde_json::json!({
            "reference": parsed.display_string(),
            "module": c_key,
            "notes": note_dtos,
        });

        self.resp_ok(id, resp)
    }

    fn handle_dict(&self, id: Option<u64>, params: &Value) -> String {
        let term = match params.get("term").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return self.resp_err(id, "Missing 'term' parameter"),
        };

        let dict_key = params
            .get("module")
            .and_then(|v| v.as_str())
            .or_else(|| self.engine.dictionaries.keys().next().map(|s| s.as_str()));

        let d_key = match dict_key {
            Some(k) => k,
            None => return self.resp_err(id, "No dictionary modules loaded"),
        };

        let dict = match self.engine.dictionaries.get(d_key) {
            Some(d) => d,
            None => return self.resp_err(id, &format!("Dictionary module '{}' not found", d_key)),
        };

        let entry = dict.get_topic(term).unwrap_or(None);
        let suggestions = if entry.is_none() {
            let prefix = if term.len() >= 3 { &term[..3] } else { term };
            dict.search_topics_prefix(prefix, 10).unwrap_or_default()
        } else {
            vec![]
        };

        let resp = serde_json::json!({
            "term": term,
            "module": d_key,
            "found": entry.is_some(),
            "topic": entry.as_ref().map(|e| &e.topic),
            "text": entry.as_ref().map(|e| &e.definition),
            "plain_text": entry.as_ref().map(|e| crate::terminal::render_plain_text(&e.definition)),
            "suggestions": suggestions,
        });

        self.resp_ok(id, resp)
    }

    fn handle_xref(&self, id: Option<u64>, params: &Value) -> String {
        let ref_str = match params.get("ref").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => return self.resp_err(id, "Missing 'ref' parameter"),
        };

        let parsed = match parse_passage_reference(ref_str) {
            Ok(p) => p,
            Err(e) => return self.resp_err(id, &e.to_string()),
        };

        let verse = parsed.verse_start.unwrap_or(1);
        let xref_key = self
            .engine
            .active_cross_references
            .clone()
            .or_else(|| self.engine.cross_references.keys().next().cloned());

        let x_key = match xref_key {
            Some(k) => k,
            None => return self.resp_err(id, "No cross-reference modules loaded"),
        };

        let xref_reader = match self.engine.cross_references.get(&x_key) {
            Some(x) => x,
            None => {
                return self.resp_err(id, &format!("Cross-reference module '{}' not found", x_key))
            }
        };

        let refs = xref_reader
            .references_for_verse(parsed.book, parsed.chapter, verse)
            .unwrap_or_default();

        let active_bible = self.engine.get_active_bible();
        let mut ref_dtos = Vec::new();
        for r in &refs {
            let mut preview = None;
            if let Some(b) = &active_bible {
                if let Ok(Some(verse_rec)) = b.verse(r.book_to, r.chapter_to, r.verse_to_start) {
                    let plain = crate::parsers::verse_text::nodes_to_plain_text(
                        &crate::parsers::verse_text::parse_verse_text(&verse_rec.text),
                    );
                    preview = Some(plain.trim().replace('\n', " "));
                }
            }
            ref_dtos.push(serde_json::json!({
                "target": r.format_target(),
                "book": r.book_to.0,
                "chapter": r.chapter_to,
                "verse": r.verse_to_start,
                "votes": r.votes,
                "preview": preview,
            }));
        }

        let resp = serde_json::json!({
            "reference": format!("{} {}:{}", parsed.book_name, parsed.chapter, verse),
            "module": x_key,
            "references": ref_dtos,
        });

        self.resp_ok(id, resp)
    }

    fn handle_search(&mut self, id: Option<u64>, params: &Value) -> String {
        let query_text = match params.get("query").and_then(|v| v.as_str()) {
            Some(q) => q.trim(),
            None => return self.resp_err(id, "Missing 'query' parameter"),
        };

        if query_text.is_empty() {
            return self.resp_err(id, "Query cannot be empty");
        }

        let mode_str = params.get("mode").and_then(|v| v.as_str()).unwrap_or("all");
        let mode = match mode_str.to_lowercase().as_str() {
            "phrase" | "exact" => SearchMode::Phrase,
            "any" | "or" => SearchMode::AnyWord,
            _ => SearchMode::AllWords,
        };

        let scope_str = params.get("testament").and_then(|v| v.as_str());
        let scope = match scope_str {
            Some("ot") => SearchScope::OldTestament,
            Some("nt") => SearchScope::NewTestament,
            _ => SearchScope::All,
        };

        let case_sensitive = params
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let ignore_accents = !params
            .get("match_accents")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|u| u as usize)
            .or(Some(100));

        let query = SearchQuery {
            text: query_text.to_string(),
            mode,
            scope,
            case_sensitive,
            ignore_accents,
            limit,
        };

        let modules_to_search: Vec<String> =
            if let Some(arr) = params.get("bibles").and_then(|v| v.as_array()) {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            } else if let Some(single) = params.get("bible").and_then(|v| v.as_str()) {
                vec![single.to_string()]
            } else if params
                .get("all_bibles")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                self.engine.bibles.keys().cloned().collect()
            } else if let Some(ref active) = self.engine.active_bible {
                vec![active.clone()]
            } else {
                self.engine.bibles.keys().take(1).cloned().collect()
            };

        let mod_refs: Vec<&str> = modules_to_search.iter().map(|s| s.as_str()).collect();
        match self.engine.search(&query, &mod_refs) {
            Ok(results) => {
                self.last_search = Some(results.clone());
                self.resp_ok(id, results)
            }
            Err(e) => self.resp_err(id, &e.to_string()),
        }
    }

    fn handle_last_search(&self, id: Option<u64>) -> String {
        match &self.last_search {
            Some(res) => self.resp_ok(id, res),
            None => self.resp_ok(id, serde_json::json!(null)),
        }
    }

    fn handle_catalog(&mut self, id: Option<u64>, params: &Value) -> String {
        if self.registry_cache.is_none() {
            let client = RepositoryClient::new();
            match client.fetch_official_registry() {
                Ok(reg) => self.registry_cache = Some(reg),
                Err(e) => return self.resp_err(id, &format!("Failed to fetch catalog: {}", e)),
            }
        }

        let reg = self.registry_cache.as_ref().unwrap();

        // 1. Compute language list with counts
        let mut lang_counts: HashMap<String, usize> = HashMap::new();
        for m in &reg.modules {
            let lang_code = m.language.as_deref().unwrap_or("unknown").to_lowercase();
            *lang_counts.entry(lang_code).or_insert(0) += 1;
        }

        let mut languages: Vec<_> = lang_counts
            .into_iter()
            .map(|(code, count)| {
                let name = language_code_to_name(&code);
                serde_json::json!({
                    "code": code,
                    "name": name,
                    "count": count,
                })
            })
            .collect();

        languages.sort_by(|a, b| {
            let count_a = a["count"].as_u64().unwrap_or(0);
            let count_b = b["count"].as_u64().unwrap_or(0);
            count_b.cmp(&count_a)
        });

        // 2. Filter modules
        let query_filter = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let lang_filter = params.get("language").and_then(|v| v.as_str());
        let type_filter = params.get("mod_type").and_then(|v| v.as_str());

        let q_lower = query_filter.trim().to_lowercase();
        let l_lower = lang_filter.map(|s| s.trim().to_lowercase());
        let t_lower = type_filter.map(|s| s.trim().to_lowercase());

        let mut matches = Vec::new();

        for m in &reg.modules {
            // Language match
            if let Some(ref req_lang) = l_lower {
                if !req_lang.is_empty() && req_lang != "all" {
                    let m_lang = m.language.as_deref().unwrap_or("").to_lowercase();
                    if !m_lang.starts_with(req_lang) {
                        continue;
                    }
                }
            }

            // Type match
            let m_type_str = format!("{:?}", m.module_type()).to_lowercase();
            if let Some(ref req_type) = t_lower {
                if !req_type.is_empty() && req_type != "all" && !m_type_str.starts_with(req_type) {
                    continue;
                }
            }

            // Query match
            if !q_lower.is_empty() {
                let hit = m.abbreviation.to_lowercase().contains(&q_lower)
                    || m.description.to_lowercase().contains(&q_lower)
                    || m.file_stem.to_lowercase().contains(&q_lower);
                if !hit {
                    continue;
                }
            }

            // Check if installed
            let is_installed = self.is_module_installed(&m.abbreviation);

            matches.push(serde_json::json!({
                "abbreviation": m.abbreviation,
                "file_stem": m.file_stem,
                "module_type": format!("{:?}", m.module_type()),
                "language": m.language,
                "size_str": m.size_str,
                "description": m.description,
                "is_installed": is_installed,
            }));

            // Avoid serializing tens of thousands at once if unfiltered
            if matches.len() >= 1000 {
                break;
            }
        }

        let resp = serde_json::json!({
            "total_matches": matches.len(),
            "languages": languages,
            "modules": matches,
        });

        self.resp_ok(id, resp)
    }

    fn is_module_installed(&self, abbr: &str) -> bool {
        self.engine.bibles.contains_key(abbr)
            || self.engine.commentaries.contains_key(abbr)
            || self.engine.dictionaries.contains_key(abbr)
            || self.engine.cross_references.contains_key(abbr)
            || self.engine.subheadings.contains_key(abbr)
            || self.engine.reading_plans.contains_key(abbr)
            || self.engine.devotions.contains_key(abbr)
    }

    fn handle_download(&mut self, id: Option<u64>, params: &Value) -> String {
        let abbr = match params.get("abbr").and_then(|v| v.as_str()) {
            Some(a) => a.trim(),
            None => return self.resp_err(id, "Missing 'abbr' parameter"),
        };

        if self.registry_cache.is_none() {
            let client = RepositoryClient::new();
            match client.fetch_official_registry() {
                Ok(reg) => self.registry_cache = Some(reg),
                Err(e) => return self.resp_err(id, &format!("Failed to fetch catalog: {}", e)),
            }
        }

        let reg = self.registry_cache.as_ref().unwrap();
        let entry = match reg.find_by_abbrev(abbr, None) {
            Some(e) => e.clone(),
            None => return self.resp_err(id, &format!("Module '{}' not found in catalog", abbr)),
        };

        let client = RepositoryClient::new();
        match self
            .engine
            .download_and_load(&client, &entry, reg, &self.data_dir)
        {
            Ok(mtype) => self.resp_ok(
                id,
                serde_json::json!({
                    "success": true,
                    "abbreviation": abbr,
                    "module_type": format!("{:?}", mtype),
                    "message": format!("Successfully downloaded and loaded module '{}'", abbr),
                }),
            ),
            Err(e) => self.resp_err(id, &format!("Download failed for '{}': {}", abbr, e)),
        }
    }

    fn handle_reload(&mut self, id: Option<u64>) -> String {
        self.engine = MyBibleEngine::new();
        let count = self.engine.load_directory(&self.data_dir).unwrap_or(0);
        if self.engine.active_bible.is_none() {
            self.engine.active_bible = self.engine.bibles.keys().next().cloned();
        }
        if self.engine.active_commentary.is_none() {
            self.engine.active_commentary = self.engine.commentaries.keys().next().cloned();
        }
        if self.engine.active_cross_references.is_none() {
            self.engine.active_cross_references =
                self.engine.cross_references.keys().next().cloned();
        }
        if self.engine.active_subheadings.is_none() {
            self.engine.active_subheadings = self.engine.subheadings.keys().next().cloned();
        }

        self.resp_ok(
            id,
            serde_json::json!({
                "reloaded_modules": count,
                "total_modules": self.engine.total_modules(),
            }),
        )
    }
}

fn language_code_to_name(code: &str) -> &'static str {
    match code.to_lowercase().as_str() {
        "en" => "English",
        "es" => "Spanish (Español)",
        "ru" => "Russian (Русский)",
        "uk" => "Ukrainian (Українська)",
        "de" => "German (Deutsch)",
        "fr" => "French (Français)",
        "pt" => "Portuguese (Português)",
        "it" => "Italian (Italiano)",
        "he" | "hbo" => "Hebrew (עברית)",
        "grc" | "el" => "Greek (Ελληνικά)",
        "la" => "Latin (Latina)",
        "zh" => "Chinese (中文)",
        "ko" => "Korean (한국어)",
        "ja" => "Japanese (日本語)",
        "ar" => "Arabic (العربية)",
        "ro" => "Romanian (Română)",
        "pl" => "Polish (Polski)",
        "nl" => "Dutch (Nederlands)",
        "sv" => "Swedish (Svenska)",
        "no" => "Norwegian (Norsk)",
        "da" => "Danish (Dansk)",
        "fi" => "Finnish (Suomi)",
        "cs" => "Czech (Čeština)",
        "hu" => "Hungarian (Magyar)",
        _ => "Other / Multi",
    }
}
