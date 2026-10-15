"""HTML Renderer for QTextBrowser
Generates clean HTML and CSS formatting for Scripture, Commentary, Dictionaries,
Parallel Bible Comparisons, and Search Results.
"""

import html
import re
from typing import Optional

THEMES = {
    "light": {
        "bg": "#ffffff",
        "fg": "#2d3748",
        "header": "#1a365d",
        "subheading": "#2b6cb0",
        "verse_num": "#3182ce",
        "jesus": "#c53030",
        "footnote": "#d69e2e",
        "strong": "#805ad5",
        "link": "#2b6cb0",
        "card_bg": "#f7fafc",
        "border": "#e2e8f0",
        "highlight": "#fefcbf",
    },
    "sepia": {
        "bg": "#fbf0d9",
        "fg": "#433422",
        "header": "#5c3d1e",
        "subheading": "#8c5b2a",
        "verse_num": "#a06c3a",
        "jesus": "#9c2727",
        "footnote": "#b7791f",
        "strong": "#6b46c1",
        "link": "#8c5b2a",
        "card_bg": "#f3e5c8",
        "border": "#e2d2b4",
        "highlight": "#fed7aa",
    },
    "dark": {
        "bg": "#1e1e2e",
        "fg": "#cdd6f4",
        "header": "#89b4fa",
        "subheading": "#a6e3a1",
        "verse_num": "#f9e2af",
        "jesus": "#f38ba8",
        "footnote": "#fab387",
        "strong": "#cba6f7",
        "link": "#89dceb",
        "card_bg": "#181825",
        "border": "#313244",
        "highlight": "#585b70",
    },
}

BIBLE_BOOK_NAMES_PATTERN = (
    r"(?:[1-3]\s*)?(?:"
    r"Génesis|Éxodo|Levítico|Números|Deuteronomio|Josué|Jueces|Rut|Samuel|Reyes|Crónicas|"
    r"Esdras|Nehemías|Ester|Job|Salmo[s]?|Proverbios|Eclesiastés|Cantares|Isaías|Jeremías|"
    r"Lamentaciones|Ezequiel|Daniel|Oseas|Joel|Amós|Abdías|Jonás|Miqueas|Nahúm|Habacuc|"
    r"Sofonías|Hageo|Zacarías|Malaquías|Mateo|Marcos|Lucas|Juan|Hechos|Romanos|Corintios|"
    r"Gálatas|Efesios|Filipenses|Colosenses|Tesalonicenses|Timoteo|Tito|Filemón|Hebreos|"
    r"Santiago|Pedro|Judas|Apocalipsis|"
    r"Genesis|Exodus|Leviticus|Numbers|Deuteronomy|Joshua|Judges|Ruth|Kings|Chronicles|"
    r"Ezra|Nehemiah|Esther|Psalms?|Proverbs|Ecclesiastes|Song|Isaiah|Jeremiah|Lamentations|"
    r"Ezekiel|Hosea|Amos|Obadiah|Jonah|Micah|Nahum|Habakkuk|Zephaniah|Haggai|Zechariah|"
    r"Malachi|Matthew|Mark|Luke|John|Acts|Romans|Corinthians|Galatians|Ephesians|Philippians|"
    r"Colossians|Thessalonians|Timothy|Titus|Philemon|Hebrews|James|Peter|Jude|Revelation|"
    r"Gen|Exod?|Lev|Num|Deut?|Josh?|Judg?|Ruth|Sam|Kgs|Chron?|Neh|Esth?|Job|Psa?|Prov?|"
    r"Eccl?|Isa|Jer|Lam|Ezek?|Dan|Hos|Joel|Amos|Obad?|Jon|Mic|Nah|Hab|Zeph?|Hag|Zech?|"
    r"Mal|Matt?|Mrk|Luk|Jhn|Act|Rom|Cor|Gal|Eph|Phil?|Col|Thess?|Tim|Tit|Phlm|Heb|Jas|Pet|Rev"
    r")\.?\s+\d+[:\.]\d+(?:[-–]\d+)?"
)

VERSE_REF_REGEX = re.compile(BIBLE_BOOK_NAMES_PATTERN, re.IGNORECASE)


def build_css(theme_name: str = "light", font_size: int = 15) -> str:
    """Constructs CSS stylesheet corresponding to theme and font size."""
    t = THEMES.get(theme_name, THEMES["light"])
    return f"""
    body {{
        background-color: {t["bg"]};
        color: {t["fg"]};
        font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, 'Roboto', 'Oxygen', sans-serif;
        font-size: {font_size}pt;
        line-height: 1.65;
        margin: 14px 20px;
    }}
    h1 {{
        color: {t["header"]};
        font-size: {int(font_size * 1.45)}pt;
        border-bottom: 2px solid {t["border"]};
        padding-bottom: 6px;
        margin-bottom: 16px;
    }}
    h2, h3, h4 {{
        color: {t["header"]};
        margin-top: 14px;
        margin-bottom: 8px;
    }}
    .subheading {{
        color: {t["subheading"]};
        font-family: Georgia, serif;
        font-size: {int(font_size * 1.15)}pt;
        font-weight: bold;
        margin-top: 20px;
        margin-bottom: 8px;
        padding: 4px 10px;
        border-left: 4px solid {t["subheading"]};
        background: {t["card_bg"]};
    }}
    .verse {{
        margin-bottom: 8px;
    }}
    .verse-num {{
        color: {t["verse_num"]};
        font-weight: bold;
        font-size: {max(9, font_size - 3)}pt;
        vertical-align: super;
        margin-right: 6px;
        text-decoration: none;
    }}
    .jesus {{
        color: {t["jesus"]};
    }}
    .footnote-link {{
        color: {t["footnote"]};
        text-decoration: none;
        font-size: {max(8, font_size - 4)}pt;
        font-weight: bold;
        vertical-align: super;
        padding: 0 2px;
    }}
    .strong-link {{
        color: {t["strong"]};
        text-decoration: none;
        font-size: {max(8, font_size - 4)}pt;
        padding: 0 2px;
    }}
    a {{
        color: {t["link"]};
        text-decoration: underline;
    }}
    .xref-link {{
        color: {t["link"]};
        text-decoration: underline;
    }}
    .verse-hover {{
        color: {t["link"]};
        text-decoration: underline;
        font-weight: 500;
    }}
    .hit-card, .compare-card {{
        background: {t["card_bg"]};
        border: 1px solid {t["border"]};
        border-radius: 6px;
        padding: 10px 14px;
        margin-bottom: 12px;
    }}
    .comm-entry {{
        background: transparent;
        border-bottom: 1px solid {t["border"]};
        padding: 8px 2px 14px 2px;
        margin-bottom: 10px;
    }}
    .comm-entry:last-child {{
        border-bottom: none;
    }}
    .comm-title {{
        color: {t["subheading"]};
        font-weight: bold;
        font-size: {int(font_size * 1.05)}pt;
        margin-bottom: 6px;
    }}
    .comm-note-item {{
        margin-bottom: 8px;
        line-height: 1.65;
    }}
    .comm-note-item:last-child {{
        margin-bottom: 0;
    }}
    .comm-text {{
        line-height: 1.65;
    }}
    .hit-title, .card-title {{
        color: {t["header"]};
        font-weight: bold;
        font-size: {int(font_size * 1.05)}pt;
        margin-bottom: 4px;
    }}
    .badge {{
        background: transparent;
        color: {t["header"]};
        border-bottom: 2px solid {t["border"]};
        font-size: {max(9, font_size - 3)}pt;
        font-weight: 600;
        margin-left: 8px;
        padding-bottom: 1px;
        opacity: 0.85;
    }}
    .trans-badge {{
        background: {t["border"]};
        color: {t["fg"]};
        padding: 2px 6px;
        border-radius: 4px;
        font-size: {max(8, font_size - 4)}pt;
        font-weight: bold;
        margin-right: 6px;
    }}
    .match-highlight {{
        background-color: {t["highlight"]};
        font-weight: bold;
        padding: 1px 2px;
        border-radius: 2px;
    }}
    """


def clean_verse_tags_to_html(raw_text: str, book_number: Optional[int] = None) -> str:
    """Translates MyBible internal tags into clean HTML."""
    text = raw_text

    # Paragraph breaks
    text = text.replace("<pb/>", "<br/><br/>")
    text = text.replace("<p/>", "<br/><br/>")
    text = text.replace("<br/>", "<br/>")

    # Words of Jesus
    text = re.sub(
        r"<J>(.*?)</J>", r'<span class="jesus">\1</span>', text, flags=re.DOTALL
    )

    # Footnotes
    def replace_fn(match):
        marker = match.group(1).strip("[]")
        return f'<a href="fn:{marker}" class="footnote-link">[{marker}]</a>'

    text = re.sub(r"<f>(.*?)</f>", replace_fn, text)

    # Strong's numbers: if book_number is known, NT (>= 400) is Greek (G), OT (< 400) is Hebrew (H)
    prefix = "G" if (book_number is None or book_number >= 400) else "H"

    def replace_bare_strong(m):
        num = m.group(1)
        full_code = f"{prefix}{num}"
        return f'<a href="dict:{full_code}" class="strong-link" title="Lookup Strong {full_code}">&lt;{num}&gt;</a>'

    def replace_prefixed_strong(m):
        code = m.group(1).upper()
        return f'<a href="dict:{code}" class="strong-link" title="Lookup Strong {code}">&lt;{code}&gt;</a>'

    text = re.sub(r"<S>([0-9]+)</S>", replace_bare_strong, text)
    text = re.sub(r"<S>([GHgh][0-9]+)</S>", replace_prefixed_strong, text)

    # Morphology codes <m>...</m>
    text = re.sub(r"<m>.*?</m>", "", text)

    # Notes <n>...</n>
    text = re.sub(r"<n>(.*?)</n>", r"<em>(\1)</em>", text)

    # Indents <t>...</t>
    text = re.sub(r"<t>(.*?)</t>", r'<div style="margin-left: 20px;">\1</div>', text)

    # Convert embedded Bible links <a href="B:..."> to goto: links with verse-hover styling
    text = re.sub(
        r"""href\s*=\s*['"]?[Bb]:([^'">\s]+(?:[^'">]*))['"]?""",
        r'href="goto:\1" class="verse-hover"',
        text,
        flags=re.IGNORECASE,
    )

    # Strip any stray XML tags
    text = re.sub(r"</?(?:e|h|b|i)[^>]*>", "", text)

    return text


def linkify_scripture_references(text: str) -> str:
    """Detects Scripture passage references in commentary or text and wraps them in hoverable links."""
    # Normalize any embedded B: links first
    text = re.sub(
        r"""href\s*=\s*['"]?[Bb]:([^'">\s]+(?:[^'">]*))['"]?""",
        r'href="goto:\1" class="verse-hover"',
        text,
        flags=re.IGNORECASE,
    )

    # Protect existing <a>...</a> tags so regex doesn't nest anchors
    parts = re.split(r"(<a\s+[^>]*>.*?</a>)", text, flags=re.DOTALL | re.IGNORECASE)
    out = []
    for part in parts:
        if part.lower().startswith("<a") and part.lower().endswith("</a>"):
            # Ensure verse-hover class is present
            if 'class="' not in part and "class='" not in part:
                part = re.sub(
                    r"(<a\s+)",
                    r'\1class="verse-hover" ',
                    part,
                    count=1,
                    flags=re.IGNORECASE,
                )
            out.append(part)
        else:

            def repl(m):
                raw_ref = m.group(0)
                clean = raw_ref.strip("()[]., ")
                return f'<a href="goto:{clean}" class="verse-hover" title="Hover for preview / click to jump">{raw_ref}</a>'

            out.append(VERSE_REF_REGEX.sub(repl, part))

    return "".join(out)


def render_chapter_html(data: dict, theme: str = "light", font_size: int = 15) -> str:
    """Renders a full chapter with subheadings and interactive verses into HTML."""
    css = build_css(theme, font_size)
    ref = data.get("reference", "")
    bible = data.get("bible", "")
    book_number = data.get("book_number")

    subheadings_by_verse: dict[int, list[str]] = {}
    for sub in data.get("subheadings", []):
        v = sub.get("verse", 1)
        subheadings_by_verse.setdefault(v, []).append(sub.get("title", ""))

    verses_html = []
    for v_item in data.get("verses", []):
        v_num = v_item.get("verse", 1)
        raw_text = v_item.get("text", "")
        formatted_text = clean_verse_tags_to_html(raw_text, book_number=book_number)

        # Prepend any subheadings for this verse
        if v_num in subheadings_by_verse:
            for s in subheadings_by_verse[v_num]:
                verses_html.append(f'<div class="subheading">{s}</div>')

        verses_html.append(f"""
        <div class="verse" id="v{v_num}">
            <a href="verse:{v_num}" class="verse-num">{v_num}</a>
            <span>{formatted_text}</span>
        </div>
        """)

    body_content = "\n".join(verses_html)

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <h1>📖 {ref} <span class="badge">({bible})</span></h1>
        {body_content}
    </body>
    </html>
    """


def render_commentary_html(
    data: dict, theme: str = "light", font_size: int = 14
) -> str:
    """Renders commentary notes with interactive hoverable Scripture references."""
    css = build_css(theme, font_size)
    ref = data.get("reference", "")
    module = data.get("module", "")
    notes = data.get("notes", [])

    if not notes:
        return f"""<!DOCTYPE html><html><head><style>{css}</style></head><body>
        <h3>📝 {ref} — {module}</h3>
        <p style="opacity: 0.7;">No commentary notes found for this passage.</p>
        </body></html>"""

    # Group consecutive notes sharing the same verse range
    grouped: list[tuple[int, int, list[str]]] = []
    for n in notes:
        v_from = n.get("verse_from", 1)
        v_to = n.get("verse_to", v_from)
        raw_text = n.get("text", "")
        formatted_text = clean_verse_tags_to_html(raw_text)
        linked_text = linkify_scripture_references(formatted_text)

        if grouped and grouped[-1][0] == v_from and grouped[-1][1] == v_to:
            grouped[-1][2].append(linked_text)
        else:
            grouped.append((v_from, v_to, [linked_text]))

    sections = []
    for v_from, v_to, note_texts in grouped:
        if v_from == 0:
            if v_to == 0:
                v_title = (
                    "Book Introduction"
                    if data.get("chapter", 1) == 0
                    else "Chapter Introduction"
                )
            else:
                v_title = f"Introduction – Verse {v_to}"
        elif v_from == v_to:
            v_title = f"Verse {v_from}"
        else:
            v_title = f"Verses {v_from}–{v_to}"

        if len(note_texts) == 1:
            body_html = f'<div class="comm-text">{note_texts[0]}</div>'
        else:
            items_html = [f'<div class="comm-note-item">{t}</div>' for t in note_texts]
            body_html = "\n".join(items_html)

        sections.append(f"""
        <div class="comm-entry">
            <div class="comm-title">{v_title}</div>
            {body_html}
        </div>
        """)

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <h3>📝 {ref} <span class="badge">({module})</span></h3>
        {"\n".join(sections)}
    </body>
    </html>
    """


def render_compare_html(data: dict, theme: str = "light", font_size: int = 14) -> str:
    """Renders aligned multi-translation comparison for a chapter or verse."""
    css = build_css(theme, font_size)
    ref = data.get("reference", "")
    bibles = data.get("bibles", [])
    verses = data.get("verses", [])

    if not verses:
        return f"""<!DOCTYPE html><html><head><style>{css}</style></head><body>
        <h3>⚖️ Bible Comparison — {ref}</h3>
        <p style="opacity: 0.7;">No comparison verses available.</p>
        </body></html>"""

    cards = []
    book_number = data.get("book_number")
    for v_item in verses:
        v_num = v_item.get("verse", 1)
        trans_rows = []
        for t in v_item.get("translations", []):
            b_name = t.get("bible", "")
            raw_text = t.get("text", "")
            cleaned = clean_verse_tags_to_html(raw_text, book_number=book_number)
            trans_rows.append(f"""
            <div style="margin-bottom: 6px; padding: 4px 0; border-bottom: 1px dashed rgba(128,128,128,0.2);">
                <span class="trans-badge">{b_name}</span>
                <span>{cleaned}</span>
            </div>
            """)

        cards.append(f"""
        <div class="compare-card">
            <div class="card-title">
                <a href="verse:{v_num}" class="verse-num" style="font-size: 1.1em;">Verse {v_num}</a>
            </div>
            {"\n".join(trans_rows)}
        </div>
        """)

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <h3>⚖️ Compare Translations — {ref}</h3>
        <p style="opacity: 0.8; font-size: 0.9em; margin-bottom: 14px;">Comparing: <b>{", ".join(bibles)}</b></p>
        {"\n".join(cards)}
    </body>
    </html>
    """


def render_search_results_html(
    data: dict, query_text: str, theme: str = "light", font_size: int = 14
) -> str:
    """Renders full-text search results with matched term highlights into HTML."""
    css = build_css(theme, font_size)
    total = data.get("total_matches", 0)
    duration = data.get("duration_ms", 0)
    hits = data.get("hits", [])

    cards = []
    for hit in hits:
        module = hit.get("bible_module", "")
        ref = hit.get("reference", hit.get("format_reference", ""))
        if not ref:
            ref = f"{hit.get('book_name', '')} {hit.get('chapter', 1)}:{hit.get('verse', 1)}"

        plain = hit.get("plain_text", "")
        matches = hit.get("matches", [])

        if matches:
            sorted_spans = sorted(matches, key=lambda x: x.get("start", 0))
            merged = []
            for s in sorted_spans:
                st, en = s.get("start", 0), s.get("end", 0)
                if merged and st <= merged[-1][1]:
                    merged[-1] = (merged[-1][0], max(merged[-1][1], en))
                else:
                    merged.append((st, en))

            highlighted = ""
            cur = 0
            for st, en in merged:
                highlighted += plain[cur:st]
                highlighted += f'<span class="match-highlight">{plain[st:en]}</span>'
                cur = en
            highlighted += plain[cur:]
        else:
            highlighted = plain

        jump_url = f"goto:{hit.get('book_name', '')}/{hit.get('chapter', 1)}/{hit.get('verse', 1)}"
        cards.append(f"""
        <div class="hit-card">
            <div class="hit-title">
                <a href="{jump_url}" style="text-decoration: none; color: inherit;">📖 {ref}</a>
                <span class="badge">{module}</span>
            </div>
            <div>{highlighted}</div>
        </div>
        """)

    content = "\n".join(cards) if cards else "<p>No matching verses found.</p>"

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <h2>Search Results for "{html.escape(query_text)}"</h2>
        <p style="opacity: 0.8; font-size: 0.9em;">Found {total} matching verses in {duration}ms (showing {len(hits)})</p>
        {content}
    </body>
    </html>
    """


def render_dict_html(data: dict, theme: str = "light", font_size: int = 14) -> str:
    """Renders dictionary entry into HTML."""
    css = build_css(theme, font_size)
    term = data.get("term", "")
    module = data.get("module", "")
    text = data.get("text", "") or data.get("definition", "")

    if not data.get("found", False):
        suggestions = data.get("suggestions", [])
        sug_html = ""
        if suggestions:
            items = "".join(f'<li><a href="dict:{s}">{s}</a></li>' for s in suggestions)
            sug_html = f"<p>Did you mean:</p><ul>{items}</ul>"

        is_strong_query = data.get("is_strong_query", False) or bool(
            re.match(r"^[GHgh]?\d+$", term)
        )
        tip_html = ""
        if is_strong_query:
            tip_html = """
            <div style="margin-top: 16px; padding: 12px; background: rgba(128,128,128,0.1); border-left: 4px solid #805ad5; border-radius: 4px;">
                <b>💡 Looking for a Strong's concordance entry?</b><br/>
                <span style="opacity: 0.85; font-size: 0.95em;">
                Make sure you have a Strong's Lexicon dictionary installed (e.g. <b>Strong-ES</b> for Spanish or <b>Strong</b> for English).
                You can select it from the <b>Dictionary:</b> dropdown above, or install it via <b>File → Manage Modules</b>.
                </span>
            </div>
            """

        return f"""<!DOCTYPE html><html><head><style>{css}</style></head><body>
        <h2>Topic '{term}' not found in {module or "any dictionary"}</h2>
        {sug_html}
        {tip_html}
        </body></html>"""

    # Replace color tokens from MyBible dictionary modules
    text = text.replace("%COLOR_PURPLE%", "#805ad5")
    text = text.replace("%COLOR_BLUE%", "#2b6cb0")
    text = text.replace("%COLOR_RED%", "#c53030")
    text = text.replace("%COLOR_GRAY%", "#718096")

    # Replace internal MyBible Strong links <a href='S:G1074'> with dict: links
    text = re.sub(r"""href=['"]S:([^'"]+)['"]""", r'href="dict:\1"', text)
    # Replace internal MyBible Scripture links <a href='B:...'> with goto: links
    text = re.sub(r"""href=['"]B:([^'"]+)['"]""", r'href="goto:\1"', text)

    # Linkify scriptures in definition
    linked_text = linkify_scripture_references(text)

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <h2>📚 {term} <span class="badge">{module}</span></h2>
        <div>{linked_text}</div>
    </body>
    </html>
    """


def render_devotion_html(data: dict, theme: str = "light", font_size: int = 14) -> str:
    """Renders daily devotional article with clean typography and interactive scripture references."""
    css = build_css(theme, font_size)
    mod = data.get("module", "")
    day = data.get("day", 1)
    max_days = data.get("max_days", 365)
    raw_text = data.get("text")

    if not raw_text:
        return f"""<!DOCTYPE html><html><head><style>{css}</style></head><body>
        <h3>🌅 Daily Devotional — Day {day} ({mod})</h3>
        <p style="opacity: 0.7;">No devotional article found for Day {day}.</p>
        </body></html>"""

    formatted_text = clean_verse_tags_to_html(raw_text)
    linked_text = linkify_scripture_references(formatted_text)

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <div style="margin-bottom: 12px; border-bottom: 2px solid rgba(128,128,128,0.25); padding-bottom: 8px;">
            <h2 style="margin: 0; display: inline-block;">🌅 Day {day} of {max_days}</h2>
            <span class="badge" style="font-size: 0.85em; margin-left: 10px;">{mod}</span>
        </div>
        <div style="line-height: 1.7; font-size: {font_size}pt;">
            {linked_text}
        </div>
    </body>
    </html>
    """
