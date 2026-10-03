#!/usr/bin/env python3
import sys
import io
import re
import html

sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding='utf-8', errors='replace')
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

raw_text = sys.stdin.read()

CSS = """
<style>
.markdown-body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
    font-size: 14px;
    line-height: 1.6;
    word-wrap: break-word;
    color: #24292e;
}
.markdown-body>*:first-child { margin-top: 0 !important; }
.markdown-body>*:last-child { margin-bottom: 0 !important; }
.markdown-body a { color: #0366d6; text-decoration: none; }
.markdown-body a:hover { text-decoration: underline; }
.markdown-body h1, .markdown-body h2, .markdown-body h3, .markdown-body h4, .markdown-body h5, .markdown-body h6 {
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
    line-height: 1.25;
    color: #24292e;
}
.markdown-body h1 { font-size: 2em; padding-bottom: 0.3em; border-bottom: 1px solid #eaecef; }
.markdown-body h2 { font-size: 1.5em; padding-bottom: 0.3em; border-bottom: 1px solid #eaecef; }
.markdown-body h3 { font-size: 1.25em; }
.markdown-body h4 { font-size: 1em; }
.markdown-body p, .markdown-body blockquote, .markdown-body ul, .markdown-body ol, .markdown-body dl, .markdown-body table, .markdown-body pre {
    margin-top: 0;
    margin-bottom: 16px;
}
.markdown-body blockquote {
    padding: 0 1em;
    color: #6a737d;
    border-left: 0.25em solid #dfe2e5;
}
.markdown-body ul, .markdown-body ol {
    padding-left: 2em;
}
.markdown-body table {
    border-spacing: 0;
    border-collapse: collapse;
    display: block;
    width: 100%;
    overflow: auto;
}
.markdown-body table th, .markdown-body table td {
    padding: 6px 13px;
    border: 1px solid #dfe2e5;
}
.markdown-body table tr:nth-child(2n) {
    background-color: #f6f8fa;
}
.markdown-body code {
    padding: 0.2em 0.4em;
    margin: 0;
    font-size: 85%;
    background-color: rgba(27,31,35,0.05);
    border-radius: 3px;
    font-family: SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace;
}
.markdown-body pre {
    padding: 16px;
    overflow: auto;
    font-size: 85%;
    line-height: 1.45;
    background-color: #f6f8fa;
    border-radius: 3px;
}
.markdown-body pre code {
    display: inline;
    padding: 0;
    margin: 0;
    overflow: visible;
    line-height: inherit;
    word-wrap: normal;
    background-color: transparent;
    border: 0;
}
.markdown-body hr {
    height: 0.25em;
    padding: 0;
    margin: 24px 0;
    background-color: #e1e4e8;
    border: 0;
}
.markdown-body img {
    max-width: 100%;
    box-sizing: content-box;
}
</style>
"""

def render():
    sys.stdout.write(CSS)
    sys.stdout.write("<div class='markdown-body'>")
    
    rendered = False
    try:
        import markdown
        try:
            from pygments.formatters import HtmlFormatter
            sys.stdout.write("<style>" + HtmlFormatter(style='pastie').get_style_defs('.highlight') + "</style>")
        except Exception:
            pass
        html_out = markdown.markdown(
            raw_text,
            output_format="html5",
            extensions=[
                "markdown.extensions.fenced_code",
                "markdown.extensions.tables",
                "markdown.extensions.sane_lists",
            ]
        )
        sys.stdout.write(html_out)
        rendered = True
    except Exception:
        pass

    if not rendered:
        lines = raw_text.splitlines()
        in_code = False
        in_p = False
        i = 0
        while i < len(lines):
            line = lines[i]
            if line.startswith("```"):
                if in_code:
                    sys.stdout.write("</code></pre>\n")
                    in_code = False
                else:
                    if in_p: sys.stdout.write("</p>\n"); in_p = False
                    sys.stdout.write("<pre><code>")
                    in_code = True
                i += 1
                continue
            if in_code:
                sys.stdout.write(html.escape(line) + "\n")
                i += 1
                continue
            
            # Check underline headers (like in cgit README)
            if i + 1 < len(lines) and lines[i+1].strip():
                next_line = lines[i+1].strip()
                if set(next_line) == {'='} and len(next_line) >= 3:
                    if in_p: sys.stdout.write("</p>\n"); in_p = False
                    sys.stdout.write(f"<h1>{html.escape(line.strip())}</h1>\n")
                    i += 2
                    continue
                elif set(next_line) == {'-'} and len(next_line) >= 3:
                    if in_p: sys.stdout.write("</p>\n"); in_p = False
                    sys.stdout.write(f"<h2>{html.escape(line.strip())}</h2>\n")
                    i += 2
                    continue

            if line.startswith("# "):
                if in_p: sys.stdout.write("</p>\n"); in_p = False
                sys.stdout.write(f"<h1>{html.escape(line[2:])}</h1>\n")
            elif line.startswith("## "):
                if in_p: sys.stdout.write("</p>\n"); in_p = False
                sys.stdout.write(f"<h2>{html.escape(line[3:])}</h2>\n")
            elif line.startswith("### "):
                if in_p: sys.stdout.write("</p>\n"); in_p = False
                sys.stdout.write(f"<h3>{html.escape(line[4:])}</h3>\n")
            elif not line.strip():
                if in_p: sys.stdout.write("</p>\n"); in_p = False
            else:
                esc = html.escape(line)
                esc = re.sub(r'`([^`]+)`', r'<code>\1</code>', esc)
                esc = re.sub(r'\*\*([^*]+)\*\*', r'<strong>\1</strong>', esc)
                if not in_p:
                    sys.stdout.write("<p>")
                    in_p = True
                else:
                    sys.stdout.write("<br/>")
                sys.stdout.write(esc)
            i += 1
        if in_code: sys.stdout.write("</code></pre>\n")
        if in_p: sys.stdout.write("</p>\n")
    
    sys.stdout.write("</div>\n")

if __name__ == '__main__':
    render()
