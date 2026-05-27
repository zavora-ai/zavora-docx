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
    doc.set_footer_page_number();
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
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
}

/// Create a KDP cookbook document (8×10, Georgia body, generous margins for photos).
pub fn create_kdp_cookbook(doc: &mut Document) {
    doc.set_page_size(Length::inches(8.0), Length::inches(10.0));
    doc.set_margins(
        Length::inches(0.75),
        Length::inches(0.75),
        Length::inches(0.75),
        Length::inches(1.0), // larger gutter for lay-flat binding
    );
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
}

/// Create a KDP children's book document (8.5×8.5 square, minimal margins).
pub fn create_kdp_children(doc: &mut Document) {
    doc.set_page_size(Length::inches(8.5), Length::inches(8.5));
    doc.set_margins(
        Length::inches(0.5),
        Length::inches(0.5),
        Length::inches(0.5),
        Length::inches(0.5),
    );
    // No page numbers for children's books
    doc.set_different_first_page(true);
}

// ── Cookbook tools ────────────────────────────────────────────────────────────

/// Insert a complete recipe layout.
pub fn insert_recipe(
    doc: &mut Document,
    index: usize,
    title: &str,
    subtitle: Option<&str>,
    prep_time: &str,
    cook_time: &str,
    servings: &str,
    ingredients: &[String],
    instructions: &[String],
    chef_tip: Option<&str>,
) -> usize {
    let mut pos = index;

    // Recipe title — bold, large, with bottom border
    let mut para = doc.insert_paragraph(pos, "");
    para = para.space_before(Length::pt(12.0))
        .space_after(Length::pt(4.0))
        .border_bottom(BorderStyle::Single, 8, "E67E22");
    para.add_run(title).font("Georgia").size(20.0).bold(true);
    pos += 1;

    // Subtitle (optional)
    if let Some(sub) = subtitle {
        let mut para = doc.insert_paragraph(pos, "");
        para = para.space_after(Length::pt(4.0));
        para.add_run(sub).font("Georgia").size(11.0).italic(true).color("666666");
        pos += 1;
    }

    // Prep info line
    let mut para = doc.insert_paragraph(pos, "");
    para = para.space_before(Length::pt(6.0)).space_after(Length::pt(12.0));
    para.add_run(&format!("Prep: {}  |  Cook: {}  |  Serves: {}", prep_time, cook_time, servings))
        .font("Georgia").size(9.0).color("888888");
    pos += 1;

    // INGREDIENTS header
    let mut para = doc.insert_paragraph(pos, "");
    para = para.space_before(Length::pt(8.0)).space_after(Length::pt(4.0));
    para.add_run("INGREDIENTS").font("Georgia").size(9.0).bold(true).all_caps(true).color("333333");
    pos += 1;

    // Ingredient list
    for item in ingredients {
        let mut para = doc.insert_paragraph(pos, "");
        para = para.indent_left(Length::inches(0.2))
            .space_before(Length::pt(1.0))
            .space_after(Length::pt(1.0));
        para.add_run(&format!("—  {}", item)).font("Georgia").size(10.0);
        pos += 1;
    }

    // INSTRUCTIONS header
    let mut para = doc.insert_paragraph(pos, "");
    para = para.space_before(Length::pt(12.0)).space_after(Length::pt(4.0));
    para.add_run("INSTRUCTIONS").font("Georgia").size(9.0).bold(true).all_caps(true).color("333333");
    pos += 1;

    // Numbered instructions
    for (i, step) in instructions.iter().enumerate() {
        let mut para = doc.insert_paragraph(pos, "");
        para = para.indent_left(Length::inches(0.3))
            .hanging_indent(Length::inches(0.3))
            .space_before(Length::pt(3.0))
            .space_after(Length::pt(3.0));
        para.add_run(&format!("{}.", i + 1)).font("Georgia").size(10.0).bold(true).color("E67E22");
        para.add_run(&format!("  {}", step)).font("Georgia").size(10.0);
        pos += 1;
    }

    // Chef's tip (optional)
    if let Some(tip) = chef_tip {
        let mut para = doc.insert_paragraph(pos, "");
        para = para.shading("FFF8F0")
            .border_all(BorderStyle::Single, 4, "E67E22")
            .indent_left(Length::inches(0.2))
            .indent_right(Length::inches(0.2))
            .space_before(Length::pt(12.0))
            .space_after(Length::pt(8.0));
        para.add_run("👨‍🍳 CHEF'S TIP: ").font("Georgia").size(9.0).bold(true).color("E67E22");
        para.add_run(tip).font("Georgia").size(9.0).italic(true);
        pos += 1;
    }

    pos - index
}

// ── Children's book tools ────────────────────────────────────────────────────

/// Insert a full-page spread: large image with text positioned below, above, or as overlay.
pub fn insert_spread(
    doc: &mut Document,
    index: usize,
    text: &str,
    text_position: &str,
    font_size: f64,
    page_break: bool,
) -> usize {
    let mut pos = index;

    if page_break && pos > 0 {
        let para = doc.insert_paragraph(pos, "");
        para.page_break_before(true);
        pos += 1;
    }

    match text_position {
        "top" => {
            // Text first, then space for image
            let mut para = doc.insert_paragraph(pos, "");
            para = para.alignment(Alignment::Center)
                .space_after(Length::pt(24.0));
            para.add_run(text).font("Century Schoolbook").size(font_size);
            pos += 1;
            // Placeholder for image area
            let mut para = doc.insert_paragraph(pos, "");
            para = para.alignment(Alignment::Center)
                .space_before(Length::pt(12.0));
            para.add_run("[illustration]").font("Century Schoolbook").size(10.0).italic(true).color("AAAAAA");
            pos += 1;
        }
        "overlay" => {
            // Image area with text overlaid (text in a shaded box)
            let mut para = doc.insert_paragraph(pos, "");
            para = para.alignment(Alignment::Center)
                .space_before(Length::pt(120.0))
                .space_after(Length::pt(120.0))
                .shading("F0F8FF");
            para.add_run(text).font("Century Schoolbook").size(font_size).bold(true);
            pos += 1;
        }
        _ => {
            // "bottom" (default) — image area first, text at bottom
            let mut para = doc.insert_paragraph(pos, "");
            para = para.alignment(Alignment::Center)
                .space_after(Length::pt(12.0));
            para.add_run("[illustration]").font("Century Schoolbook").size(10.0).italic(true).color("AAAAAA");
            pos += 1;
            let mut para = doc.insert_paragraph(pos, "");
            para = para.alignment(Alignment::Center)
                .space_before(Length::pt(24.0));
            para.add_run(text).font("Century Schoolbook").size(font_size);
            pos += 1;
        }
    }

    pos - index
}

/// Insert large emphasis text for children's books (sound effects, key words).
pub fn insert_big_text(doc: &mut Document, index: usize, text: &str, size: f64, bold: bool) {
    let mut para = doc.insert_paragraph(index, "");
    para = para.alignment(Alignment::Center)
        .space_before(Length::pt(12.0))
        .space_after(Length::pt(12.0));
    let mut run = para.add_run(text);
    run = run.font("Century Schoolbook").size(size);
    if bold { run.bold(true); }
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

pub fn insert_code_block(doc: &mut Document, index: usize, code: &str, language: Option<&str>, margin_inches: f64, padding_pt: f64, bg_color: &str, code_font: &str, code_size: f64) -> usize {
    let lines: Vec<&str> = code.lines().collect();
    let count = lines.len();

    for (i, line) in lines.iter().enumerate() {
        let mut para = doc.insert_paragraph(index + i, "");
        para = para.shading(bg_color)
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
                run = run.font(code_font).size(code_size);
                if let Some(color) = token_color(kind) {
                    run.color(color);
                }
            }
        } else {
            para.add_run(if line.is_empty() { " " } else { line })
                .font(code_font).size(code_size);
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
