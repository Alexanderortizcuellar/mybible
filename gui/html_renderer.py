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
        background-color: {t['bg']};
        color: {t['fg']};
        font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, 'Roboto', 'Oxygen', sans-serif;
        font-size: {font_size}pt;
        line-height: 1.65;
        margin: 14px 20px;
    }}
    h1 {{
        color: {t['header']};
        font-size: {int(font_size * 1.45)}pt;
        border-bottom: 2px solid {t['border']};
        padding-bottom: 6px;
        margin-bottom: 16px;
    }}
    h2, h3, h4 {{
        color: {t['header']};
        margin-top: 14px;
        margin-bottom: 8px;
    }}
    .subheading {{
        color: {t['subheading']};
        font-family: Georgia, serif;
        font-size: {int(font_size * 1.15)}pt;
        font-weight: bold;
        margin-top: 20px;
        margin-bottom: 8px;
        padding: 4px 10px;
        border-left: 4px solid {t['subheading']};
        background: {t['card_bg']};
    }}
    .verse {{
        margin-bottom: 8px;
    }}
    .verse-num {{
        color: {t['verse_num']};
        font-weight: bold;
        font-size: {max(9, font_size - 3)}pt;
        vertical-align: super;
        margin-right: 6px;
        text-decoration: none;
    }}
    .jesus {{
        color: {t['jesus']};
    }}
    .footnote-link {{
        color: {t['footnote']};
        text-decoration: none;
        font-size: {max(8, font_size - 4)}pt;
        font-weight: bold;
        vertical-align: super;
        padding: 0 2px;
    }}
    .strong-link {{
        color: {t['strong']};
        text-decoration: none;
        font-size: {max(8, font_size - 4)}pt;
        padding: 0 2px;
    }}
    .xref-link {{
        color: {t['link']};
        text-decoration: underline;
    }}
    .verse-hover {{
        color: {t['link']};
        text-decoration: underline;
        font-weight: 500;
    }}
    .hit-card, .comm-card, .compare-card {{
        background: {t['card_bg']};
        border: 1px solid {t['border']};
        border-radius: 6px;
        padding: 10px 14px;
        margin-bottom: 12px;
    }}
    .hit-title, .card-title {{
        color: {t['header']};
        font-weight: bold;
        font-size: {int(font_size * 1.05)}pt;
        margin-bottom: 4px;
    }}
    .badge {{
        background: {t['subheading']};
        color: #ffffff;
        padding: 2px 7px;
        border-radius: 4px;
        font-size: {max(8, font_size - 4)}pt;
        font-weight: bold;
        margin-left: 6px;
    }}
    .trans-badge {{
        background: {t['border']};
        color: {t['fg']};
        padding: 2px 6px;
        border-radius: 4px;
        font-size: {max(8, font_size - 4)}pt;
        font-weight: bold;
        margin-right: 6px;
    }}
    .match-highlight {{
        background-color: {t['highlight']};
        font-weight: bold;
        padding: 1px 2px;
        border-radius: 2px;
    }}
    """


def clean_verse_tags_to_html(raw_text: str) -> str:
    """Translates MyBible internal tags into clean HTML."""
    text = raw_text

    # Paragraph breaks
    text = text.replace("<pb/>", "<br/><br/>")
    text = text.replace("<p/>", "<br/><br/>")
    text = text.replace("<br/>", "<br/>")

    # Words of Jesus
    text = re.sub(r"<J>(.*?)</J>", r'<span class="jesus">\1</span>', text, flags=re.DOTALL)

    # Footnotes
    def replace_fn(match):
        marker = match.group(1).strip("[]")
        return f'<a href="fn:{marker}" class="footnote-link">[{marker}]</a>'

    text = re.sub(r"<f>(.*?)</f>", replace_fn, text)

    # Strong's numbers
    text = re.sub(r"<S>([0-9]+)</S>", r'<a href="dict:\1" class="strong-link">&lt;\1&gt;</a>', text)
    text = re.sub(r"<S>([GH][0-9]+)</S>", r'<a href="dict:\1" class="strong-link">&lt;\1&gt;</a>', text)

    # Morphology codes <m>...</m>
    text = re.sub(r"<m>.*?</m>", "", text)

    # Notes <n>...</n>
    text = re.sub(r"<n>(.*?)</n>", r"<em>(\1)</em>", text)

    # Indents <t>...</t>
    text = re.sub(r"<t>(.*?)</t>", r'<div style="margin-left: 20px;">\1</div>', text)

    # Strip any stray XML tags
    text = re.sub(r"</?(?:e|h|b|i)[^>]*>", "", text)

    return text


def linkify_scripture_references(text: str) -> str:
    """Detects Scripture passage references in commentary or text and wraps them in hoverable links."""
    def repl(m):
        raw_ref = m.group(0)
        clean = raw_ref.strip("()[]., ")
        return f'<a href="goto:{clean}" class="verse-hover" title="Hover for preview / click to jump">{raw_ref}</a>'

    return VERSE_REF_REGEX.sub(repl, text)


def render_chapter_html(data: dict, theme: str = "light", font_size: int = 15) -> str:
    """Renders a full chapter with subheadings and interactive verses into HTML."""
    css = build_css(theme, font_size)
    ref = data.get("reference", "")
    bible = data.get("bible", "")

    subheadings_by_verse: dict[int, list[str]] = {}
    for sub in data.get("subheadings", []):
        v = sub.get("verse", 1)
        subheadings_by_verse.setdefault(v, []).append(sub.get("title", ""))

    verses_html = []
    for v_item in data.get("verses", []):
        v_num = v_item.get("verse", 1)
        raw_text = v_item.get("text", "")
        formatted_text = clean_verse_tags_to_html(raw_text)

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
        <h1>📖 {ref} <span class="badge">{bible}</span></h1>
        {body_content}
    </body>
    </html>
    """


def render_commentary_html(data: dict, theme: str = "light", font_size: int = 14) -> str:
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

    cards = []
    for n in notes:
        v_from = n.get("verse_from", 1)
        v_to = n.get("verse_to", v_from)
        v_badge = f"{v_from}" if v_from == v_to else f"{v_from}–{v_to}"
        raw_text = n.get("text", "")
        formatted_text = clean_verse_tags_to_html(raw_text)
        linked_text = linkify_scripture_references(formatted_text)

        cards.append(f"""
        <div class="comm-card">
            <div class="card-title">
                <span style="color: #2b6cb0;">Verse {v_badge}</span>
            </div>
            <div>{linked_text}</div>
        </div>
        """)

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <h3>📝 {ref} <span class="badge">{module}</span></h3>
        {"\n".join(cards)}
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
    for v_item in verses:
        v_num = v_item.get("verse", 1)
        trans_rows = []
        for t in v_item.get("translations", []):
            b_name = t.get("bible", "")
            raw_text = t.get("text", "")
            cleaned = clean_verse_tags_to_html(raw_text)
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
        return f"""<!DOCTYPE html><html><head><style>{css}</style></head><body>
        <h2>Topic '{term}' not found in {module}</h2>
        {sug_html}
        </body></html>"""

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
