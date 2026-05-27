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
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CodeBlockInput {
    pub document_handle: String,
    pub index: usize,
    pub code: String,
    /// "rust", "python", "bash", "json", etc.
    pub language: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CalloutInput {
    pub document_handle: String,
    pub index: usize,
    /// "tip", "warning", or "note"
    pub callout_type: String,
    pub text: String,
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
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HeaderFooterInput {
    pub document_handle: String,
    pub header: Option<String>,
    pub footer: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportInput { pub document_handle: String }

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
    #[tool(description = "Create a new DOCX document. format: 'kdp:technical' for 6x9 tech book, 'kdp:novel' for 5.25x8 fiction, or omit for blank.")]
    async fn create_document(&self, Parameters(input): Parameters<CreateInput>) -> String {
        let mut doc = rdocx::Document::new();
        match input.format.as_deref() {
            Some("kdp:technical" | "kdp") => engine::create_kdp_technical(&mut doc),
            Some("kdp:novel") => engine::create_kdp_novel(&mut doc),
            _ => {}
        }
        let mut store = self.store.lock().await;
        let handle = store.insert(doc);
        serde_json::json!({"handle": handle}).to_string()
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

    #[tool(description = "Insert a paragraph with optional style and page break. Styles: Heading1, Heading2, Heading3, BodyText, BodyTextIndent, ChapterNum, TitlePage, Subtitle, Author, Copyright")]
    async fn insert_paragraph(&self, Parameters(input): Parameters<InsertParaInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let mut para = doc.insert_paragraph(input.index, "");

            if input.page_break_before.unwrap_or(false) {
                para = para.page_break_before(true);
            }

            // Apply style-specific formatting
            let style = input.style.as_deref();
            match style {
                Some("Heading1") => {
                    para = para.alignment(rdocx::Alignment::Center)
                        .space_before(rdocx::Length::pt(24.0))
                        .space_after(rdocx::Length::pt(12.0))
                        .keep_with_next(true)
                        .outline_level(0);
                    para.add_run(&input.text).font("Garamond").size(24.0).bold(true);
                }
                Some("Heading2") => {
                    para = para.space_before(rdocx::Length::pt(18.0))
                        .space_after(rdocx::Length::pt(6.0))
                        .keep_with_next(true)
                        .outline_level(1);
                    para.add_run(&input.text).font("Garamond").size(14.0).bold(true);
                }
                Some("Heading3") => {
                    para = para.space_before(rdocx::Length::pt(12.0))
                        .space_after(rdocx::Length::pt(4.0))
                        .keep_with_next(true)
                        .outline_level(2);
                    para.add_run(&input.text).font("Garamond").size(12.0).bold(true);
                }
                Some("BodyTextIndent") => {
                    para = para.first_line_indent(rdocx::Length::inches(0.3))
                        .line_spacing_multiple(1.3);
                    para.add_run(&input.text).font("Garamond").size(11.0);
                }
                Some("ChapterNum") => {
                    para = para.alignment(rdocx::Alignment::Center)
                        .space_after(rdocx::Length::pt(6.0));
                    para.add_run(&input.text).font("Garamond").size(12.0).small_caps(true);
                }
                Some("TitlePage") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    para.add_run(&input.text).font("Garamond").size(28.0).bold(true);
                }
                Some("Subtitle") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    para.add_run(&input.text).font("Garamond").size(14.0).italic(true);
                }
                Some("Author") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    para.add_run(&input.text).font("Garamond").size(14.0);
                }
                Some("Copyright") => {
                    para.add_run(&input.text).font("Garamond").size(9.0);
                }
                _ => {
                    // BodyText (default)
                    para = para.line_spacing_multiple(1.3);
                    para.add_run(&input.text).font("Garamond").size(11.0);
                }
            }

            serde_json::json!({"index": input.index}).to_string()
        })
    }

    #[tool(description = "Insert a syntax-highlighted code block with gray background (Courier New 9pt). Supports 'rust' highlighting.")]
    async fn insert_code_block(&self, Parameters(input): Parameters<CodeBlockInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let lines = engine::insert_code_block(doc, input.index, &input.code, input.language.as_deref());
            serde_json::json!({"lines_inserted": lines}).to_string()
        })
    }

    #[tool(description = "Insert a callout box with colored background and border (tip=green, warning=orange, note=blue)")]
    async fn insert_callout(&self, Parameters(input): Parameters<CalloutInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_callout(doc, input.index, &input.callout_type, &input.text);
            serde_json::json!({"inserted": true}).to_string()
        })
    }

    #[tool(description = "Insert a table at the given position")]
    async fn add_table(&self, Parameters(input): Parameters<TableInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            doc.insert_table(input.index, input.rows, input.cols);
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

    #[tool(description = "Add an image from file path")]
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

    #[tool(description = "Insert a scene break (for novels). style: 'asterisks', 'diamond', or 'blank'")]
    async fn insert_scene_break(&self, Parameters(input): Parameters<SceneBreakInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_scene_break(doc, input.index, input.style.as_deref().unwrap_or("asterisks"));
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
}
