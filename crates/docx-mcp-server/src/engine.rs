//! Core document engine using rdocx.

use rdocx::{Document, Length, BorderStyle, Alignment};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// In-memory document store.
pub struct Store {
    docs: HashMap<String, Document>,
}

impl Store {
    pub fn new() -> Self {
        Self { docs: HashMap::new() }
    }

    pub fn insert(&mut self, doc: Document) -> String {
        let handle = Uuid::new_v4().to_string();
        self.docs.insert(handle.clone(), doc);
        handle
    }

    pub fn get_mut(&mut self, handle: &str) -> Option<&mut Document> {
        self.docs.get_mut(handle)
    }

    pub fn remove(&mut self, handle: &str) -> bool {
        self.docs.remove(handle).is_some()
    }
}

pub type SharedStore = Arc<Mutex<Store>>;

pub fn new_store() -> SharedStore {
    Arc::new(Mutex::new(Store::new()))
}

// ── KDP Templates ────────────────────────────────────────────────────────────

pub fn create_kdp_technical(doc: &mut Document) {
    doc.set_page_size(Length::inches(6.0), Length::inches(9.0));
    doc.set_margins(
        Length::inches(0.75),
        Length::inches(0.75),
        Length::inches(0.75),
        Length::inches(0.875),
    );
    doc.set_footer("{{PAGE}}");
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
}

pub fn create_kdp_novel(doc: &mut Document) {
    doc.set_page_size(Length::inches(5.25), Length::inches(8.0));
    doc.set_margins(
        Length::inches(0.75),
        Length::inches(0.625),
        Length::inches(0.75),
        Length::inches(0.875),
    );
    doc.set_footer("{{PAGE}}");
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
}

// ── Syntax highlighting ──────────────────────────────────────────────────────

struct SyntaxColors;
impl SyntaxColors {
    const KEYWORD: &str = "0000FF";
    const STRING: &str = "A31515";
    const COMMENT: &str = "008000";
    const FUNCTION: &str = "795E26";
    const TYPE: &str = "267F99";
    const NUMBER: &str = "098658";
    const MACRO: &str = "AF00DB";
}

#[allow(dead_code)]
enum TokenKind { Keyword, String, Comment, Function, Type, Number, Macro, Plain }

fn tokenize_rust(line: &str) -> Vec<(TokenKind, String)> {
    let mut tokens = Vec::new();
    let keywords = ["use", "fn", "let", "mut", "pub", "struct", "impl", "async", "await",
        "match", "if", "else", "for", "in", "return", "self", "Self", "crate", "mod",
        "true", "false", "Ok", "Err", "Some", "None", "where", "trait", "enum", "type"];

    if line.trim_start().starts_with("//") {
        tokens.push((TokenKind::Comment, line.to_string()));
        return tokens;
    }

    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            if !current.is_empty() {
                tokens.push((classify_word(&current, &keywords), current.clone()));
                current.clear();
            }
            let mut s = String::new();
            s.push(chars.next().unwrap());
            while let Some(&c) = chars.peek() {
                s.push(chars.next().unwrap());
                if c == '"' && !s.ends_with("\\\"") { break; }
            }
            tokens.push((TokenKind::String, s));
        } else if ch == '#' {
            if !current.is_empty() {
                tokens.push((classify_word(&current, &keywords), current.clone()));
                current.clear();
            }
            let mut s = String::new();
            while let Some(&c) = chars.peek() {
                if c == ']' { s.push(chars.next().unwrap()); break; }
                s.push(chars.next().unwrap());
            }
            tokens.push((TokenKind::Macro, s));
        } else if ch.is_alphanumeric() || ch == '_' {
            current.push(chars.next().unwrap());
        } else {
            if !current.is_empty() {
                tokens.push((classify_word(&current, &keywords), current.clone()));
                current.clear();
            }
            tokens.push((TokenKind::Plain, chars.next().unwrap().to_string()));
        }
    }
    if !current.is_empty() {
        tokens.push((classify_word(&current, &keywords), current));
    }
    tokens
}

fn classify_word(word: &str, keywords: &[&str]) -> TokenKind {
    if keywords.contains(&word) { TokenKind::Keyword }
    else if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) { TokenKind::Type }
    else if word.chars().all(|c| c.is_ascii_digit() || c == '.') { TokenKind::Number }
    else { TokenKind::Plain }
}

fn token_color(kind: &TokenKind) -> Option<&'static str> {
    match kind {
        TokenKind::Keyword => Some(SyntaxColors::KEYWORD),
        TokenKind::String => Some(SyntaxColors::STRING),
        TokenKind::Comment => Some(SyntaxColors::COMMENT),
        TokenKind::Function => Some(SyntaxColors::FUNCTION),
        TokenKind::Type => Some(SyntaxColors::TYPE),
        TokenKind::Number => Some(SyntaxColors::NUMBER),
        TokenKind::Macro => Some(SyntaxColors::MACRO),
        TokenKind::Plain => None,
    }
}

// ── Tool implementations ─────────────────────────────────────────────────────

pub fn insert_code_block(doc: &mut Document, index: usize, code: &str, language: Option<&str>, margin_inches: f64, padding_pt: f64) -> usize {
    let lines: Vec<&str> = code.lines().collect();
    let count = lines.len();

    for (i, line) in lines.iter().enumerate() {
        let mut para = doc.insert_paragraph(index + i, "");
        para = para.shading("F5F5F5")
            .indent_left(Length::inches(margin_inches))
            .indent_right(Length::inches(margin_inches))
            .line_spacing_multiple(1.0)
            .space_before(Length::pt(0.0))
            .space_after(Length::pt(0.0))
            .keep_together(true);

        if i == 0 {
            para = para.space_before(Length::pt(padding_pt));
        }
        if i == count - 1 {
            para = para.space_after(Length::pt(padding_pt));
        }

        if matches!(language, Some("rust" | "rs")) {
            let tokens = tokenize_rust(line);
            for (kind, text) in &tokens {
                let mut run = para.add_run(text);
                run = run.font("Courier New").size(9.0);
                if let Some(color) = token_color(kind) {
                    run.color(color);
                }
            }
        } else {
            para.add_run(if line.is_empty() { " " } else { line })
                .font("Courier New").size(9.0);
        }
    }
    count
}

pub fn insert_callout(doc: &mut Document, index: usize, callout_type: &str, text: &str, margin: f64, padding: f64, border_size: u32) {
    let (prefix, border_color, bg_color) = match callout_type {
        "warning" => ("⚠ WARNING: ", "ED7D31", "FFF2CC"),
        "note" => ("📝 NOTE: ", "4472C4", "D9E2F3"),
        _ => ("💡 TIP: ", "70AD47", "E2EFDA"),
    };

    let mut para = doc.insert_paragraph(index, "");
    para = para
        .shading(bg_color)
        .border_all(BorderStyle::Single, border_size, border_color)
        .indent_left(Length::inches(margin))
        .indent_right(Length::inches(margin))
        .space_before(Length::pt(padding))
        .space_after(Length::pt(padding));

    para.add_run(prefix).font("Garamond").size(10.0).bold(true);
    para.add_run(text).font("Garamond").size(10.0);
}

pub fn insert_scene_break(doc: &mut Document, index: usize, style: &str, spacing: f64) {
    let symbol = match style {
        "diamond" => "◆",
        "blank" => "",
        _ => "* * *",
    };
    let mut para = doc.insert_paragraph(index, "");
    para = para
        .alignment(Alignment::Center)
        .space_before(Length::pt(spacing))
        .space_after(Length::pt(spacing));
    if !symbol.is_empty() {
        para.add_run(symbol).font("Garamond").size(11.0);
    }
}

pub fn insert_drop_cap(doc: &mut Document, index: usize, text: &str, size: f64) {
    let first = &text[..text.chars().next().map(|c| c.len_utf8()).unwrap_or(0)];
    let rest = &text[first.len()..];

    let mut para = doc.insert_paragraph(index, "");
    para = para.line_spacing_multiple(1.3);
    para.add_run(first).font("Garamond").size(size);
    para.add_run(rest).font("Garamond").size(11.0);
}

pub fn insert_epigraph(doc: &mut Document, index: usize, quote: &str, attribution: Option<&str>) {
    let mut para = doc.insert_paragraph(index, "");
    para = para
        .indent_left(Length::inches(1.5))
        .indent_right(Length::inches(0.5))
        .space_after(Length::pt(4.0));
    para.add_run(quote).font("Garamond").size(10.0).italic(true);

    if let Some(attr) = attribution {
        let mut p2 = doc.insert_paragraph(index + 1, "");
        p2 = p2
            .indent_left(Length::inches(1.5))
            .indent_right(Length::inches(0.5))
            .space_after(Length::pt(12.0));
        p2.add_run(attr).font("Garamond").size(10.0);
    }
}

pub fn insert_figure_caption(doc: &mut Document, index: usize, caption: &str) {
    let mut para = doc.insert_paragraph(index, "");
    para = para
        .alignment(Alignment::Center)
        .space_before(Length::pt(4.0))
        .space_after(Length::pt(8.0));
    para.add_run(caption).font("Garamond").size(10.0).italic(true);
}
