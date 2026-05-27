//! MCP server with tool routing.

use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use crate::engine::{self, SharedStore};

// ── Input types ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateInput {
    pub title: Option<String>,
    /// "kdp:technical", "kdp:novel", or omit for blank
    pub format: Option<String>,
    /// Page width in inches
    pub page_width: Option<f64>,
    /// Page height in inches
    pub page_height: Option<f64>,
    /// Top margin in inches
    pub margin_top: Option<f64>,
    /// Bottom margin in inches
    pub margin_bottom: Option<f64>,
    /// Left margin in inches
    pub margin_left: Option<f64>,
    /// Right margin in inches
    pub margin_right: Option<f64>,
    /// Default font family
    pub default_font: Option<String>,
    /// Default font size in points
    pub default_size: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HandleInput { pub document_handle: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OpenInput { pub file_path: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SaveInput { pub document_handle: String, pub output_path: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InsertParaInput {
    pub document_handle: String,
    pub index: usize,
    pub text: String,
    /// "Heading1", "Heading2", "Heading3", "BodyText", "BodyTextIndent", "ChapterNum", "TitlePage", "Subtitle", "Author", "Copyright"
    pub style: Option<String>,
    pub page_break_before: Option<bool>,
    /// Font family (default "Garamond")
    pub font: Option<String>,
    /// Font size in points (default 11.0)
    pub font_size: Option<f64>,
    /// Line spacing multiplier (default 1.3)
    pub line_spacing: Option<f64>,
    /// Bold text
    pub bold: Option<bool>,
    /// Italic text
    pub italic: Option<bool>,
    /// Hex color e.g. "FF0000"
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CodeBlockInput {
    pub document_handle: String,
    pub index: usize,
    pub code: String,
    /// "rust", "python", "bash", "json", etc.
    pub language: Option<String>,
    /// Left/right margin in inches (default 0.3)
    pub margin: Option<f64>,
    /// Top/bottom padding in points (default 8)
    pub padding: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CalloutInput {
    pub document_handle: String,
    pub index: usize,
    /// "tip", "warning", or "note"
    pub callout_type: String,
    pub text: String,
    /// Left/right margin in inches (default 0.3)
    pub margin: Option<f64>,
    /// Top/bottom padding in points (default 8.0)
    pub padding: Option<f64>,
    /// Border width in eighths of a point (default 4)
    pub border_size: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TableInput {
    pub document_handle: String,
    pub index: usize,
    pub rows: usize,
    pub cols: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CellInput {
    pub document_handle: String,
    pub row: usize,
    pub col: usize,
    pub text: String,
    /// Which table (0-based index among all tables in document)
    pub table_index: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ImageInput {
    pub document_handle: String,
    pub image_path: String,
    /// Width in inches
    pub width: Option<f64>,
    /// Height in inches
    pub height: Option<f64>,
    /// Optional caption text below image
    pub caption: Option<String>,
    /// Whether caption is italic (default true)
    pub caption_italic: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TocInput {
    pub document_handle: String,
    pub index: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SceneBreakInput {
    pub document_handle: String,
    pub index: usize,
    /// "asterisks", "diamond", or "blank"
    pub style: Option<String>,
    /// Spacing above/below in points (default 18.0)
    pub spacing: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HeaderFooterInput {
    pub document_handle: String,
    pub header: Option<String>,
    pub footer: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportInput { pub document_handle: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DropCapInput {
    pub document_handle: String,
    pub index: usize,
    pub text: String,
    /// Size of the drop cap letter in points (default 48.0)
    pub size: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EpigraphInput {
    pub document_handle: String,
    pub index: usize,
    /// The quote text
    pub quote: String,
    /// Attribution (e.g. "— Author Name")
    pub attribution: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FigureCaptionInput {
    pub document_handle: String,
    pub index: usize,
    /// Caption text e.g. "Figure 1.1: System architecture"
    pub caption: String,
}

// ── Server ───────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DocxServer {
    store: SharedStore,
}

impl DocxServer {
    pub fn new() -> Self {
        Self { store: engine::new_store() }
    }
}

macro_rules! with_doc {
    ($store:expr, $handle:expr, $body:expr) => {{
        let mut store = $store.lock().await;
        match store.get_mut(&$handle) {
            Some(doc) => {
                let result = $body(doc);
                result
            }
            None => serde_json::json!({"error": "Document not found"}).to_string(),
        }
    }};
}

#[tool_router(server_handler)]
impl DocxServer {
    #[tool(description = "Create a new DOCX document. format: 'kdp:technical' for 6x9 tech book, 'kdp:novel' for 5.25x8 fiction, or omit for blank. Optional page/margin overrides.")]
    async fn create_document(&self, Parameters(input): Parameters<CreateInput>) -> String {
        let mut doc = rdocx::Document::new();
        match input.format.as_deref() {
            Some("kdp:technical" | "kdp") => engine::create_kdp_technical(&mut doc),
            Some("kdp:novel") => engine::create_kdp_novel(&mut doc),
            _ => {}
        }
        // Apply optional overrides
        if input.page_width.is_some() || input.page_height.is_some() {
            doc.set_page_size(
                rdocx::Length::inches(input.page_width.unwrap_or(6.0)),
                rdocx::Length::inches(input.page_height.unwrap_or(9.0)),
            );
        }
        if input.margin_top.is_some() || input.margin_bottom.is_some() || input.margin_left.is_some() || input.margin_right.is_some() {
            doc.set_margins(
                rdocx::Length::inches(input.margin_top.unwrap_or(0.75)),
                rdocx::Length::inches(input.margin_right.unwrap_or(0.75)),
                rdocx::Length::inches(input.margin_bottom.unwrap_or(0.75)),
                rdocx::Length::inches(input.margin_left.unwrap_or(0.875)),
            );
        }
        let mut store = self.store.lock().await;
        let handle = store.insert(doc);
        serde_json::json!({"handle": handle, "default_font": input.default_font.as_deref().unwrap_or("Garamond"), "default_size": input.default_size.unwrap_or(11.0)}).to_string()
    }

    #[tool(description = "Open an existing .docx file from disk")]
    async fn open_document(&self, Parameters(input): Parameters<OpenInput>) -> String {
        match rdocx::Document::open(&input.file_path) {
            Ok(doc) => {
                let mut store = self.store.lock().await;
                let handle = store.insert(doc);
                serde_json::json!({"handle": handle}).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Save document to disk as .docx")]
    async fn save_document(&self, Parameters(input): Parameters<SaveInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            match doc.save(&input.output_path) {
                Ok(_) => serde_json::json!({"saved": input.output_path}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        })
    }

    #[tool(description = "Close a document and free memory")]
    async fn close_document(&self, Parameters(input): Parameters<HandleInput>) -> String {
        let mut store = self.store.lock().await;
        if store.remove(&input.document_handle) {
            serde_json::json!({"closed": true}).to_string()
        } else {
            serde_json::json!({"error": "Not found"}).to_string()
        }
    }

    #[tool(description = "Get document info: paragraph count, table count, word count")]
    async fn describe_document(&self, Parameters(input): Parameters<HandleInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            serde_json::json!({
                "paragraphs": doc.paragraph_count(),
                "tables": doc.table_count(),
                "content_elements": doc.content_count(),
                "word_count": doc.word_count(),
            }).to_string()
        })
    }

    #[tool(description = "Insert a paragraph with optional style, font, size, spacing, bold, italic, color. Styles: Heading1, Heading2, Heading3, BodyText, BodyTextIndent, ChapterNum, TitlePage, Subtitle, Author, Copyright")]
    async fn insert_paragraph(&self, Parameters(input): Parameters<InsertParaInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let font = input.font.as_deref().unwrap_or("Garamond");
            let size = input.font_size.unwrap_or(11.0);
            let spacing = input.line_spacing.unwrap_or(1.3);

            let mut para = doc.insert_paragraph(input.index, "");

            if input.page_break_before.unwrap_or(false) {
                para = para.page_break_before(true);
            }

            let style = input.style.as_deref();
            match style {
                Some("Heading1") => {
                    para = para.alignment(rdocx::Alignment::Center)
                        .space_before(rdocx::Length::pt(24.0))
                        .space_after(rdocx::Length::pt(12.0))
                        .keep_with_next(true)
                        .outline_level(0);
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(input.font_size.unwrap_or(24.0)).bold(true);
                    if let Some(c) = &input.color { run.color(c); }
                }
                Some("Heading2") => {
                    para = para.space_before(rdocx::Length::pt(18.0))
                        .space_after(rdocx::Length::pt(6.0))
                        .keep_with_next(true)
                        .outline_level(1);
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(input.font_size.unwrap_or(14.0)).bold(true);
                    if let Some(c) = &input.color { run.color(c); }
                }
                Some("Heading3") => {
                    para = para.space_before(rdocx::Length::pt(12.0))
                        .space_after(rdocx::Length::pt(4.0))
                        .keep_with_next(true)
                        .outline_level(2);
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(input.font_size.unwrap_or(12.0)).bold(true);
                    if let Some(c) = &input.color { run.color(c); }
                }
                Some("BodyTextIndent") => {
                    para = para.first_line_indent(rdocx::Length::inches(0.3))
                        .line_spacing_multiple(spacing);
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(size);
                    if input.bold.unwrap_or(false) { run = run.bold(true); }
                    if input.italic.unwrap_or(false) { run = run.italic(true); }
                    if let Some(c) = &input.color { run.color(c); }
                }
                Some("ChapterNum") => {
                    para = para.alignment(rdocx::Alignment::Center)
                        .space_after(rdocx::Length::pt(6.0));
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(input.font_size.unwrap_or(12.0)).small_caps(true);
                    if let Some(c) = &input.color { run.color(c); }
                }
                Some("TitlePage") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(input.font_size.unwrap_or(28.0)).bold(true);
                    if let Some(c) = &input.color { run.color(c); }
                }
                Some("Subtitle") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(input.font_size.unwrap_or(14.0)).italic(true);
                    if let Some(c) = &input.color { run.color(c); }
                }
                Some("Author") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(input.font_size.unwrap_or(14.0));
                    if let Some(c) = &input.color { run.color(c); }
                }
                Some("Copyright") => {
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(input.font_size.unwrap_or(9.0));
                    if let Some(c) = &input.color { run.color(c); }
                }
                _ => {
                    para = para.line_spacing_multiple(spacing);
                    let mut run = para.add_run(&input.text);
                    run = run.font(font).size(size);
                    if input.bold.unwrap_or(false) { run = run.bold(true); }
                    if input.italic.unwrap_or(false) { run = run.italic(true); }
                    if let Some(c) = &input.color { run.color(c); }
                }
            }

            serde_json::json!({"index": input.index}).to_string()
        })
    }

    #[tool(description = "Insert a syntax-highlighted code block with gray background (Courier New 9pt). Supports 'rust' highlighting.")]
    async fn insert_code_block(&self, Parameters(input): Parameters<CodeBlockInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let lines = engine::insert_code_block(doc, input.index, &input.code, input.language.as_deref(), input.margin.unwrap_or(0.3), input.padding.unwrap_or(8.0));
            serde_json::json!({"lines_inserted": lines}).to_string()
        })
    }

    #[tool(description = "Insert a callout box with colored background and border (tip=green, warning=orange, note=blue)")]
    async fn insert_callout(&self, Parameters(input): Parameters<CalloutInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_callout(
                doc, input.index, &input.callout_type, &input.text,
                input.margin.unwrap_or(0.3),
                input.padding.unwrap_or(8.0),
                input.border_size.unwrap_or(4),
            );
            serde_json::json!({"inserted": true}).to_string()
        })
    }

    #[tool(description = "Insert a table at the given position")]
    async fn add_table(&self, Parameters(input): Parameters<TableInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            doc.insert_table(input.index, input.rows, input.cols)
                .borders(rdocx::BorderStyle::Single, 4, "CCCCCC");
            serde_json::json!({"index": input.index, "rows": input.rows, "cols": input.cols}).to_string()
        })
    }

    #[tool(description = "Set text in a table cell")]
    async fn set_table_cell(&self, Parameters(input): Parameters<CellInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let ti = input.table_index.unwrap_or(0);
            match doc.table_mut(ti) {
                Some(mut table) => {
                    match table.cell(input.row, input.col) {
                        Some(mut cell) => {
                            cell.set_text(&input.text);
                            serde_json::json!({"set": true, "row": input.row, "col": input.col}).to_string()
                        }
                        None => serde_json::json!({"error": format!("Cell ({},{}) not found", input.row, input.col)}).to_string(),
                    }
                }
                None => serde_json::json!({"error": "Table not found"}).to_string(),
            }
        })
    }

    #[tool(description = "Add an image from file path, with optional caption")]
    async fn add_image(&self, Parameters(input): Parameters<ImageInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let img_data = match std::fs::read(&input.image_path) {
                Ok(d) => d,
                Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
            };
            let ext = input.image_path.rsplit('.').next().unwrap_or("png");
            let filename = format!("image.{}", ext);
            let w = rdocx::Length::inches(input.width.unwrap_or(4.0));
            let h = rdocx::Length::inches(input.height.unwrap_or(3.0));
            doc.add_picture(&img_data, &filename, w, h);
            if let Some(caption) = &input.caption {
                let mut para = doc.add_paragraph("");
                para = para.alignment(rdocx::Alignment::Center);
                let mut run = para.add_run(caption);
                run = run.font("Garamond").size(10.0);
                if input.caption_italic.unwrap_or(true) {
                    run.italic(true);
                }
            }
            serde_json::json!({"added": true}).to_string()
        })
    }

    #[tool(description = "Insert a linked Table of Contents at the given position")]
    async fn add_toc(&self, Parameters(input): Parameters<TocInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            doc.insert_toc(input.index, 3);
            serde_json::json!({"index": input.index}).to_string()
        })
    }

    #[tool(description = "Insert a scene break (for novels). style: 'asterisks', 'diamond', or 'blank'. Optional spacing in points.")]
    async fn insert_scene_break(&self, Parameters(input): Parameters<SceneBreakInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_scene_break(doc, input.index, input.style.as_deref().unwrap_or("asterisks"), input.spacing.unwrap_or(18.0));
            serde_json::json!({"inserted": true}).to_string()
        })
    }

    #[tool(description = "Set header and/or footer text. Use {{PAGE}} for page numbers.")]
    async fn set_header_footer(&self, Parameters(input): Parameters<HeaderFooterInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            if let Some(h) = &input.header { doc.set_header(h); }
            if let Some(f) = &input.footer { doc.set_footer(f); }
            serde_json::json!({"set": true}).to_string()
        })
    }

    #[tool(description = "Export document as plain text")]
    async fn to_plain_text(&self, Parameters(input): Parameters<ExportInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let mut text = String::new();
            for para in doc.paragraphs() {
                text.push_str(&para.text());
                text.push('\n');
            }
            serde_json::json!({"text": text}).to_string()
        })
    }

    #[tool(description = "Insert a drop cap: first letter large, rest of paragraph normal size")]
    async fn insert_drop_cap(&self, Parameters(input): Parameters<DropCapInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_drop_cap(doc, input.index, &input.text, input.size.unwrap_or(48.0));
            serde_json::json!({"index": input.index}).to_string()
        })
    }

    #[tool(description = "Insert an epigraph: italic quote with optional attribution, indented right")]
    async fn insert_epigraph(&self, Parameters(input): Parameters<EpigraphInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_epigraph(doc, input.index, &input.quote, input.attribution.as_deref());
            serde_json::json!({"index": input.index}).to_string()
        })
    }

    #[tool(description = "Insert a centered italic figure caption (e.g. 'Figure 1.1: description')")]
    async fn insert_figure_caption(&self, Parameters(input): Parameters<FigureCaptionInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_figure_caption(doc, input.index, &input.caption);
            serde_json::json!({"index": input.index}).to_string()
        })
    }
}
