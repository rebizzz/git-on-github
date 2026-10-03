use std::io::{self, Read, Write};
use pulldown_cmark::{html, Options, Parser};

const CSS: &str = r#"<style>
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
    height: auto;
    box-sizing: border-box;
    display: inline-block;
    vertical-align: middle;
}
.markdown-body img.curl-logo, .markdown-body img[src*="curl-logo"] {
    max-width: 240px;
    height: auto;
    display: block;
    margin: 12px 0;
}
</style>
"#;

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    // Replace curl logo with local asset
    let processed = input.replace(
        "https://curl.se/logo/curl-logo.svg",
        "/git-on-github/cgit-css/curl-logo.svg",
    );

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&processed, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(CSS.as_bytes());
    let _ = stdout.write_all(b"<div class='markdown-body'>");
    let _ = stdout.write_all(html_output.as_bytes());
    let _ = stdout.write_all(b"</div>\n");
}
