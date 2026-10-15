"""
HTML Renderer for QTextBrowser
Generates clean HTML and CSS formatting for Scripture, Commentary, Dictionaries, and Search Results.
"""

import re
from typing import Optional

THEMES = {
    "light": {
        "bg": "#ffffff",
        "fg": "#2b2b2b",
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
        margin: 16px 24px;
    }}
    h1 {{
        color: {t['header']};
        font-size: {int(font_size * 1.5)}pt;
        border-bottom: 2px solid {t['border']};
        padding-bottom: 6px;
        margin-bottom: 16px;
    }}
    .subheading {{
        color: {t['subheading']};
        font-family: Georgia, serif;
        font-size: {int(font_size * 1.2)}pt;
        font-weight: bold;
        margin-top: 24px;
        margin-bottom: 10px;
        padding: 4px 8px;
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
    .footnote-link:hover {{
        text-decoration: underline;
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
    .hit-card {{
        background: {t['card_bg']};
        border: 1px solid {t['border']};
        border-radius: 6px;
        padding: 10px 14px;
        margin-bottom: 12px;
    }}
    .hit-title {{
        color: {t['header']};
        font-weight: bold;
        font-size: {int(font_size * 1.05)}pt;
        margin-bottom: 4px;
    }}
    .badge {{
        background: {t['subheading']};
        color: {t['bg']};
        padding: 2px 6px;
        border-radius: 4px;
        font-size: {max(8, font_size - 4)}pt;
        font-weight: bold;
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

    # Footnotes: <f>[1]</f> -> <a href="fn:1" class="footnote-link">[1]</a>
    def replace_fn(match):
        marker = match.group(1).strip("[]")
        return f'<a href="fn:{marker}" class="footnote-link">[{marker}]</a>'

    text = re.sub(r"<f>(.*?)</f>", replace_fn, text)

    # Strong's numbers: <S>26</S> -> <a href="dict:G26" class="strong-link">&lt;26&gt;</a>
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

        # Prepend any subheadings before this verse
        if v_num in subheadings_by_verse:
            for s_title in subheadings_by_verse[v_num]:
                verses_html.append(f'<div class="subheading">{s_title}</div>')

        # Build verse row
        v_anchor = f'<a name="v{v_num}"></a><a href="verse:{v_num}" class="verse-num">{v_num}</a>'
        verses_html.append(f'<div class="verse">{v_anchor} {formatted_text}</div>')

    content = "\n".join(verses_html)

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <h1>{ref} <span style="font-size: 0.6em; opacity: 0.7;">({bible})</span></h1>
        {content}
    </body>
    </html>
    """


def render_search_results_html(results: dict, theme: str = "light", font_size: int = 14) -> str:
    """Renders search results with highlights into HTML."""
    css = build_css(theme, font_size)
    query_text = results.get("query", {}).get("text", "")
    total = results.get("total_matches", 0)
    hits = results.get("hits", [])
    duration = results.get("duration_ms", 0)

    cards = []
    for hit in hits:
        ref = f"{hit.get('book_name', '')} {hit.get('chapter', 1)}:{hit.get('verse', 1)}"
        module = hit.get("bible_module", "")
        plain = hit.get("plain_text", "").strip()

        # Highlight matches using spans
        matches = hit.get("matches", [])
        if matches:
            # Sort spans
            sorted_spans = sorted(matches, key=lambda s: s.get("start", 0))
            # Merge
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
        <h2>Search Results for "{query_text}"</h2>
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
    definition = data.get("definition", "")

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

    return f"""<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8"/>
        <style>{css}</style>
    </head>
    <body>
        <h2>📚 {term} <span class="badge">{module}</span></h2>
        <div>{definition}</div>
    </body>
    </html>
    """
