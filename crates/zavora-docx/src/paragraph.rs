//! Paragraph — a block-level container for runs of text.

use zavora_docx_oxml::borders::{CT_BorderEdge, CT_PBdr, CT_TabStop, CT_Tabs};
use zavora_docx_oxml::document::CT_SectPr;
use zavora_docx_oxml::properties::{CT_PPr, CT_Shd};
use zavora_docx_oxml::shared::{
    ST_Border, ST_Jc, ST_PageOrientation, ST_SectionType, ST_TabJc, ST_TabLeader,
};
use zavora_docx_oxml::text::{CT_P, CT_R};
use zavora_docx_oxml::units::Twips;

use crate::Length;
use crate::run::{Run, RunRef};

/// Paragraph alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justify,
}

impl Alignment {
    fn to_st_jc(self) -> ST_Jc {
        match self {
            Alignment::Left => ST_Jc::Left,
            Alignment::Center => ST_Jc::Center,
            Alignment::Right => ST_Jc::Right,
            Alignment::Justify => ST_Jc::Both,
        }
    }

    fn from_st_jc(jc: ST_Jc) -> Self {
        match jc {
            ST_Jc::Center => Alignment::Center,
            ST_Jc::Right | ST_Jc::End => Alignment::Right,
            ST_Jc::Both | ST_Jc::Distribute => Alignment::Justify,
            _ => Alignment::Left,
        }
    }
}

/// Border style for paragraph borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Single,
    Thick,
    Double,
    Dotted,
    Dashed,
    DotDash,
    Wave,
}

impl BorderStyle {
    /// Convert to the OXML ST_Border type (public for table module).
    pub(crate) fn to_st_border(self) -> ST_Border {
        self.to_st()
    }

    fn to_st(self) -> ST_Border {
        match self {
            Self::None => ST_Border::None,
            Self::Single => ST_Border::Single,
            Self::Thick => ST_Border::Thick,
            Self::Double => ST_Border::Double,
            Self::Dotted => ST_Border::Dotted,
            Self::Dashed => ST_Border::Dashed,
            Self::DotDash => ST_Border::DotDash,
            Self::Wave => ST_Border::Wave,
        }
    }
}

/// Tab stop alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAlignment {
    Left,
    Center,
    Right,
    Decimal,
}

impl TabAlignment {
    fn to_st(self) -> ST_TabJc {
        match self {
            Self::Left => ST_TabJc::Left,
            Self::Center => ST_TabJc::Center,
            Self::Right => ST_TabJc::Right,
            Self::Decimal => ST_TabJc::Decimal,
        }
    }
}

/// Tab leader character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabLeader {
    None,
    Dot,
    Hyphen,
    Underscore,
}

impl TabLeader {
    fn to_st(self) -> ST_TabLeader {
        match self {
            Self::None => ST_TabLeader::None,
            Self::Dot => ST_TabLeader::Dot,
            Self::Hyphen => ST_TabLeader::Hyphen,
            Self::Underscore => ST_TabLeader::Underscore,
        }
    }
}

/// A mutable reference to a paragraph in a document.
pub struct Paragraph<'a> {
    pub(crate) inner: &'a mut CT_P,
}

impl<'a> Paragraph<'a> {
    /// Get the combined text of all runs.
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Add a general Word field (complex field) with the given instruction and
    /// optional cached result text — e.g. `DATE \@ "yyyy-MM-dd"`, `REF bookmark`,
    /// `SEQ Figure \* ARABIC`, `STYLEREF "Heading 1"`. Word recomputes it on open
    /// when update fields is enabled.
    pub fn add_field(&mut self, instruction: &str, cached: Option<&str>) {
        let result = cached.unwrap_or("");
        let xml = format!(
            concat!(
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="begin"/></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:instrText xml:space="preserve"> {} </w:instrText></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="separate"/></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:t xml:space="preserve">{}</w:t></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            instruction, result
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), xml.into_bytes()));
    }

    /// Add a run with the given text and return a mutable reference for chaining.
    pub fn add_run(&mut self, text: &str) -> Run<'_> {
        self.inner.runs.push(CT_R::new(text));
        Run {
            inner: self.inner.runs.last_mut().unwrap(),
        }
    }

    /// Add a tab character (a real `<w:tab/>`) that advances to the next tab
    /// stop. Use with `add_tab_stop` to right-align trailing text (e.g. dates).
    pub fn add_tab(&mut self) -> &mut Self {
        use zavora_docx_oxml::text::RunContent;
        let mut r = CT_R::new("");
        r.content = vec![RunContent::Tab];
        self.inner.runs.push(r);
        self
    }

    /// Add a hyperlink run. The `rel_id` should be obtained from `Document::add_hyperlink_rel()`.
    /// For internal links, pass `None` for rel_id and `Some(bookmark)` for anchor.
    pub fn add_hyperlink_run(
        &mut self,
        text: &str,
        rel_id: Option<&str>,
        anchor: Option<&str>,
    ) -> Run<'_> {
        let run_start = self.inner.runs.len();
        self.inner.runs.push(CT_R::new(text));
        let run_end = self.inner.runs.len();
        self.inner
            .hyperlinks
            .push(zavora_docx_oxml::text::HyperlinkSpan {
                rel_id: rel_id.map(|s| s.to_string()),
                anchor: anchor.map(|s| s.to_string()),
                run_start,
                run_end,
            });
        Run {
            inner: self.inner.runs.last_mut().unwrap(),
        }
    }

    /// Add a bookmark at this paragraph. The bookmark wraps all content in the paragraph.
    /// Use `id` as a unique integer and `name` as the bookmark name for cross-references.

    /// Make this paragraph a drop cap (large first letter spanning multiple lines).
    /// `lines` is how many lines the drop cap spans (typically 2-4).
    pub fn drop_cap(mut self, lines: u32) -> Self {
        self.ensure_ppr().drop_cap_lines = Some(lines);
        self
    }

    pub fn bookmark(&mut self, id: u32, name: &str) {
        let bm = zavora_docx_oxml::bookmark::CT_Bookmark::new(id, name);
        // bookmarkStart before all runs, bookmarkEnd after all runs
        self.inner.extra_xml.push((0, bm.start_xml()));
        let pos = self.inner.runs.len();
        self.inner.extra_xml.push((pos, bm.end_xml()));
    }

    /// Insert a cross-reference (`REF` field) to a bookmark by name, showing
    /// the referenced text and updating on field refresh.
    pub fn cross_reference(&mut self, name: &str) {
        let bm = zavora_docx_oxml::bookmark::CT_Bookmark::new(0, name);
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), bm.cross_reference_xml()));
    }

    /// Mark the start of a comment range. Place before the commented text runs.
    pub fn comment_start(&mut self, id: u32) {
        let xml = format!(
            r#"<w:commentRangeStart w:id="{}" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#,
            id
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), xml.into_bytes()));
    }

    /// Mark the end of a comment range and insert the comment reference. Place after the commented text runs.
    pub fn comment_end(&mut self, id: u32) {
        let end_xml = format!(
            r#"<w:commentRangeEnd w:id="{}" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#,
            id
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), end_xml.into_bytes()));
        // Add a run with commentReference
        let ref_xml = format!(
            r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="{}"/></w:r>"#,
            id
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), ref_xml.into_bytes()));
    }

    /// Insert only a comment reference run (no range) — used to anchor a reply
    /// comment at its parent's location so Word displays it in the thread.
    pub fn comment_reference(&mut self, id: u32) {
        let ref_xml = format!(
            r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="{}"/></w:r>"#,
            id
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), ref_xml.into_bytes()));
    }

    /// Add a tracked insertion (text shown as added in review mode).
    pub fn add_tracked_insert(&mut self, text: &str, author: &str) {
        let xml = format!(
            r#"<w:ins w:id="{}" w:author="{}" w:date="2026-01-01T00:00:00Z" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:rPr><w:color w:val="FF0000"/></w:rPr><w:t>{}</w:t></w:r></w:ins>"#,
            self.inner.runs.len() + 100,
            author,
            text
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), xml.into_bytes()));
    }

    /// Add a tracked deletion (text shown as strikethrough in review mode).
    pub fn add_tracked_delete(&mut self, text: &str, author: &str) {
        let xml = format!(
            r#"<w:del w:id="{}" w:author="{}" w:date="2026-01-01T00:00:00Z" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:rPr><w:strike/><w:color w:val="FF0000"/></w:rPr><w:delText>{}</w:delText></w:r></w:del>"#,
            self.inner.runs.len() + 200,
            author,
            text
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), xml.into_bytes()));
    }

    /// Add a text input form field.
    pub fn add_text_field(&mut self, name: &str, default_value: &str) {
        let xml = format!(
            concat!(
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="begin"><w:ffData>"#,
                r#"<w:name w:val="{}"/><w:enabled/><w:calcOnExit w:val="0"/>"#,
                r#"<w:textInput><w:default w:val="{}"/></w:textInput>"#,
                r#"</w:ffData></w:fldChar></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:instrText> FORMTEXT </w:instrText></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="separate"/></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:t>{}</w:t></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            name, default_value, default_value
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), xml.into_bytes()));
    }

    /// Add a checkbox form field.
    pub fn add_checkbox(&mut self, name: &str, checked: bool) {
        let check_val = if checked { "1" } else { "0" };
        let xml = format!(
            concat!(
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="begin"><w:ffData>"#,
                r#"<w:name w:val="{}"/><w:enabled/><w:calcOnExit w:val="0"/>"#,
                r#"<w:checkBox><w:sizeAuto/><w:default w:val="{}"/></w:checkBox>"#,
                r#"</w:ffData></w:fldChar></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:instrText> FORMCHECKBOX </w:instrText></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="separate"/></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            name, check_val
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), xml.into_bytes()));
    }

    /// Add a dropdown form field.
    pub fn add_dropdown(&mut self, name: &str, options: &[&str], selected: usize) {
        let mut entries = String::new();
        for opt in options {
            entries.push_str(&format!(r#"<w:listEntry w:val="{}"/>"#, opt));
        }
        let xml = format!(
            concat!(
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="begin"><w:ffData>"#,
                r#"<w:name w:val="{}"/><w:enabled/><w:calcOnExit w:val="0"/>"#,
                r#"<w:ddList><w:result w:val="{}"/>{}</w:ddList>"#,
                r#"</w:ffData></w:fldChar></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:instrText> FORMDROPDOWN </w:instrText></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="separate"/></w:r>"#,
                r#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            name, selected, entries
        );
        self.inner
            .extra_xml
            .push((self.inner.runs.len(), xml.into_bytes()));
    }

    /// Get an iterator over immutable run references.
    pub fn runs(&self) -> impl Iterator<Item = RunRef<'_>> {
        self.inner.runs.iter().map(|r| RunRef { inner: r })
    }

    /// Set the paragraph alignment.
    pub fn alignment(mut self, align: Alignment) -> Self {
        self.ensure_ppr().jc = Some(align.to_st_jc());
        self
    }

    /// Set the paragraph style by ID.
    pub fn style(mut self, style_id: &str) -> Self {
        self.ensure_ppr().style_id = Some(style_id.to_string());
        self
    }

    /// Set space before the paragraph.
    pub fn space_before(mut self, length: Length) -> Self {
        self.ensure_ppr().space_before = Some(length.as_twips());
        self
    }

    /// Set space after the paragraph.
    pub fn space_after(mut self, length: Length) -> Self {
        self.ensure_ppr().space_after = Some(length.as_twips());
        self
    }

    /// Set left indentation.
    pub fn indent_left(mut self, length: Length) -> Self {
        self.ensure_ppr().ind_left = Some(length.as_twips());
        self
    }

    /// Set right indentation.
    pub fn indent_right(mut self, length: Length) -> Self {
        self.ensure_ppr().ind_right = Some(length.as_twips());
        self
    }

    /// Set first line indent.
    pub fn first_line_indent(mut self, length: Length) -> Self {
        self.ensure_ppr().ind_first_line = Some(length.as_twips());
        self
    }

    /// Set hanging indent.
    pub fn hanging_indent(mut self, length: Length) -> Self {
        self.ensure_ppr().ind_hanging = Some(length.as_twips());
        self
    }

    /// Set keep with next paragraph.
    pub fn keep_with_next(mut self, val: bool) -> Self {
        self.ensure_ppr().keep_next = Some(val);
        self
    }

    /// Set keep lines together.
    pub fn keep_together(mut self, val: bool) -> Self {
        self.ensure_ppr().keep_lines = Some(val);
        self
    }

    /// Set page break before.
    pub fn page_break_before(mut self, val: bool) -> Self {
        self.ensure_ppr().page_break_before = Some(val);
        self
    }

    /// Set widow/orphan control.
    pub fn widow_control(mut self, val: bool) -> Self {
        self.ensure_ppr().widow_control = Some(val);
        self
    }

    /// Set line spacing in points with "exact" rule.
    pub fn line_spacing(mut self, pt: f64) -> Self {
        let ppr = self.ensure_ppr();
        ppr.line_spacing = Some(Twips::from_pt(pt));
        ppr.line_rule = Some("exact".to_string());
        self
    }

    /// Set line spacing with a multiplier (1.0 = single, 1.5, 2.0 = double, etc.).
    pub fn line_spacing_multiple(mut self, multiple: f64) -> Self {
        let ppr = self.ensure_ppr();
        // In "auto" mode, line spacing is in 240ths of a line (240 = single)
        ppr.line_spacing = Some(Twips((multiple * 240.0) as i32));
        ppr.line_rule = Some("auto".to_string());
        self
    }

    /// Set a background/shading fill color (hex string, e.g. "FFFF00").
    pub fn shading(mut self, fill_color: &str) -> Self {
        self.ensure_ppr().shading = Some(CT_Shd {
            val: "clear".to_string(),
            color: Some("auto".to_string()),
            fill: Some(fill_color.to_string()),
        });
        self
    }

    /// Add a border to all sides.
    pub fn border_all(mut self, style: BorderStyle, size_eighths_pt: u32, color: &str) -> Self {
        let edge = CT_BorderEdge {
            val: style.to_st(),
            sz: Some(size_eighths_pt),
            space: Some(1),
            color: Some(color.to_string()),
        };
        self.ensure_ppr().borders = Some(CT_PBdr {
            top: Some(edge.clone()),
            bottom: Some(edge.clone()),
            left: Some(edge.clone()),
            right: Some(edge),
            between: None,
            bar: None,
        });
        self
    }

    /// Add a bottom border only.
    pub fn border_bottom(mut self, style: BorderStyle, size_eighths_pt: u32, color: &str) -> Self {
        let edge = CT_BorderEdge {
            val: style.to_st(),
            sz: Some(size_eighths_pt),
            space: Some(1),
            color: Some(color.to_string()),
        };
        let borders = self
            .ensure_ppr()
            .borders
            .get_or_insert_with(CT_PBdr::default);
        borders.bottom = Some(edge);
        self
    }

    /// Add a tab stop.
    pub fn add_tab_stop(mut self, alignment: TabAlignment, position: Length) -> Self {
        let tabs = self
            .ensure_ppr()
            .tabs
            .get_or_insert_with(|| CT_Tabs { tabs: Vec::new() });
        tabs.tabs.push(CT_TabStop {
            val: alignment.to_st(),
            pos: position.as_twips(),
            leader: None,
        });
        self
    }

    /// Add a tab stop with a leader character.
    pub fn add_tab_stop_with_leader(
        mut self,
        alignment: TabAlignment,
        position: Length,
        leader: TabLeader,
    ) -> Self {
        let tabs = self
            .ensure_ppr()
            .tabs
            .get_or_insert_with(|| CT_Tabs { tabs: Vec::new() });
        tabs.tabs.push(CT_TabStop {
            val: alignment.to_st(),
            pos: position.as_twips(),
            leader: Some(leader.to_st()),
        });
        self
    }

    /// Set outline level (0–8, used for TOC generation).
    pub fn outline_level(mut self, level: u32) -> Self {
        self.ensure_ppr().outline_lvl = Some(level);
        self
    }

    /// Add a section break after this paragraph.
    ///
    /// This creates a `<w:sectPr>` inside the paragraph's properties,
    /// ending the current section at this paragraph.
    pub fn section_break(mut self, break_type: SectionBreak) -> Self {
        let sect = self.ensure_sect_pr();
        sect.section_type = Some(break_type.to_st());
        self
    }

    /// Set the section ending at this paragraph to landscape orientation.
    ///
    /// Sets page dimensions to 11" x 8.5" (US Letter landscape).
    /// Must be combined with `section_break()` to create a section break.
    pub fn section_landscape(mut self) -> Self {
        let sect = self.ensure_sect_pr();
        sect.orientation = Some(ST_PageOrientation::Landscape);
        sect.page_width = Some(Twips(15840)); // 11"
        sect.page_height = Some(Twips(12240)); // 8.5"
        self
    }

    /// Set the section ending at this paragraph to portrait orientation.
    ///
    /// Sets page dimensions to 8.5" x 11" (US Letter portrait).
    /// Must be combined with `section_break()` to create a section break.
    pub fn section_portrait(mut self) -> Self {
        let sect = self.ensure_sect_pr();
        sect.orientation = Some(ST_PageOrientation::Portrait);
        sect.page_width = Some(Twips(12240)); // 8.5"
        sect.page_height = Some(Twips(15840)); // 11"
        self
    }

    /// Set custom page dimensions for the section ending at this paragraph.
    pub fn section_page_size(mut self, width: crate::Length, height: crate::Length) -> Self {
        let sect = self.ensure_sect_pr();
        sect.page_width = Some(width.as_twips());
        sect.page_height = Some(height.as_twips());
        self
    }

    fn ensure_ppr(&mut self) -> &mut CT_PPr {
        self.inner.properties.get_or_insert_with(CT_PPr::default)
    }

    fn ensure_sect_pr(&mut self) -> &mut CT_SectPr {
        let ppr = self.ensure_ppr();
        ppr.sect_pr.get_or_insert_with(CT_SectPr::default_letter)
    }
}

/// Section break type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionBreak {
    /// Start a new section on the next page.
    NextPage,
    /// Start a new section on the same page (continuous).
    Continuous,
    /// Start a new section on the next even-numbered page.
    EvenPage,
    /// Start a new section on the next odd-numbered page.
    OddPage,
}

impl SectionBreak {
    fn to_st(self) -> ST_SectionType {
        match self {
            SectionBreak::NextPage => ST_SectionType::NextPage,
            SectionBreak::Continuous => ST_SectionType::Continuous,
            SectionBreak::EvenPage => ST_SectionType::EvenPage,
            SectionBreak::OddPage => ST_SectionType::OddPage,
        }
    }
}

/// An immutable reference to a paragraph.
pub struct ParagraphRef<'a> {
    pub(crate) inner: &'a CT_P,
}

impl<'a> ParagraphRef<'a> {
    /// Get the combined text of all runs.
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Get the paragraph style ID, if set.
    pub fn style_id(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.style_id.as_deref())
    }

    /// Get the alignment, if set.
    pub fn alignment(&self) -> Option<Alignment> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.jc)
            .map(Alignment::from_st_jc)
    }

    /// Get an iterator over immutable run references.
    pub fn runs(&self) -> impl Iterator<Item = RunRef<'_>> {
        self.inner.runs.iter().map(|r| RunRef { inner: r })
    }

    /// Check if paragraph has borders.
    pub fn has_borders(&self) -> bool {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.borders.as_ref())
            .map(|b| !b.is_empty())
            .unwrap_or(false)
    }

    /// Get the number of tab stops defined.
    pub fn tab_stop_count(&self) -> usize {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.tabs.as_ref())
            .map(|t| t.tabs.len())
            .unwrap_or(0)
    }

    /// Get the shading fill color, if set.
    pub fn shading_fill(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.shading.as_ref())
            .and_then(|shd| shd.fill.as_deref())
    }
}
