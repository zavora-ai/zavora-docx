//! Style access and manipulation for documents.

use zavora_docx_oxml::properties::{CT_PPr, CT_RPr};
use zavora_docx_oxml::styles::{CT_Style, CT_Styles, StyleType};

/// An immutable reference to a style definition.
pub struct Style<'a> {
    pub(crate) inner: &'a CT_Style,
}

impl<'a> Style<'a> {
    /// The style ID (used to reference this style).
    pub fn style_id(&self) -> &str {
        &self.inner.style_id
    }

    /// The display name of the style.
    pub fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }

    /// The style ID this style is based on.
    pub fn based_on(&self) -> Option<&str> {
        self.inner.based_on.as_deref()
    }

    /// Whether this is the default style for its type.
    pub fn is_default(&self) -> bool {
        self.inner.is_default
    }
}

/// Builder for creating a new paragraph style.
pub struct StyleBuilder {
    style: CT_Style,
}

impl StyleBuilder {
    /// Create a new paragraph style builder.
    pub fn paragraph(style_id: &str, name: &str) -> Self {
        StyleBuilder {
            style: CT_Style {
                style_id: style_id.to_string(),
                style_type: StyleType::Paragraph,
                name: Some(name.to_string()),
                based_on: None,
                next_style: None,
                is_default: false,
                ppr: None,
                rpr: None,
                extra_xml: None,
            },
        }
    }

    /// Create a new character style builder.
    pub fn character(style_id: &str, name: &str) -> Self {
        StyleBuilder {
            style: CT_Style {
                style_id: style_id.to_string(),
                style_type: StyleType::Character,
                name: Some(name.to_string()),
                based_on: None,
                next_style: None,
                is_default: false,
                ppr: None,
                rpr: None,
                extra_xml: None,
            },
        }
    }

    /// Set the parent style this one inherits from.
    pub fn based_on(mut self, style_id: &str) -> Self {
        self.style.based_on = Some(style_id.to_string());
        self
    }

    /// Set the next style (applied to the following paragraph after pressing Enter).
    pub fn next_style(mut self, style_id: &str) -> Self {
        self.style.next_style = Some(style_id.to_string());
        self
    }

    // ── Typed fluent setters (no need to touch internal oxml types) ──────────

    fn ppr(&mut self) -> &mut CT_PPr {
        self.style.ppr.get_or_insert_with(CT_PPr::default)
    }
    fn rpr(&mut self) -> &mut CT_RPr {
        self.style.rpr.get_or_insert_with(CT_RPr::default)
    }

    /// Text alignment: "left", "right", "center", "both" (justified).
    pub fn align(mut self, jc: &str) -> Self {
        use zavora_docx_oxml::shared::ST_Jc;
        self.ppr().jc = Some(match jc {
            "center" => ST_Jc::Center,
            "right" => ST_Jc::Right,
            "both" | "justify" | "justified" => ST_Jc::Both,
            _ => ST_Jc::Left,
        });
        self
    }

    /// First-line indent.
    pub fn first_line_indent(mut self, len: crate::Length) -> Self {
        self.ppr().ind_first_line = Some(len.as_twips());
        self
    }

    /// Left indent.
    pub fn left_indent(mut self, len: crate::Length) -> Self {
        self.ppr().ind_left = Some(len.as_twips());
        self
    }

    /// Line spacing as a multiple of single spacing (1.0 = single, 1.5, 2.0…).
    pub fn line_spacing(mut self, multiple: f64) -> Self {
        let ppr = self.ppr();
        ppr.line_spacing = Some(zavora_docx_oxml::units::Twips((multiple * 240.0).round() as i32));
        ppr.line_rule = Some("auto".to_string());
        self
    }

    /// Space before/after the paragraph.
    pub fn spacing(mut self, before: crate::Length, after: crate::Length) -> Self {
        let ppr = self.ppr();
        ppr.space_before = Some(before.as_twips());
        ppr.space_after = Some(after.as_twips());
        self
    }

    /// Keep this paragraph with the next (no page break between).
    pub fn keep_with_next(mut self, val: bool) -> Self {
        self.ppr().keep_next = Some(val);
        self
    }

    /// Widow/orphan control (avoid a lone line at top/bottom of a page).
    pub fn widow_control(mut self, val: bool) -> Self {
        self.ppr().widow_control = Some(val);
        self
    }

    /// Outline level (0 = Heading1 … 8). Set so the TOC and nav pane detect it.
    pub fn outline_level(mut self, lvl: u32) -> Self {
        self.ppr().outline_lvl = Some(lvl);
        self
    }

    /// Page-break before this paragraph.
    pub fn page_break_before(mut self, val: bool) -> Self {
        self.ppr().page_break_before = Some(val);
        self
    }

    /// Font family (sets ascii + hAnsi). Clears any theme-font reference.
    pub fn font(mut self, family: &str) -> Self {
        let rpr = self.rpr();
        rpr.font_ascii = Some(family.to_string());
        rpr.font_hansi = Some(family.to_string());
        rpr.font_ascii_theme = None;
        rpr.font_hansi_theme = None;
        self
    }

    /// Font size in points.
    pub fn size(mut self, pt: f64) -> Self {
        let hp = zavora_docx_oxml::units::HalfPoint((pt * 2.0).round() as u32);
        let rpr = self.rpr();
        rpr.sz = Some(hp);
        rpr.sz_cs = Some(hp);
        self
    }

    /// Text color as a hex string ("000000").
    pub fn color(mut self, hex: &str) -> Self {
        self.rpr().color = Some(hex.to_string());
        self
    }

    /// Bold.
    pub fn bold(mut self, val: bool) -> Self {
        let rpr = self.rpr();
        rpr.bold = Some(val);
        rpr.bold_cs = Some(val);
        self
    }

    /// Italic.
    pub fn italic(mut self, val: bool) -> Self {
        let rpr = self.rpr();
        rpr.italic = Some(val);
        rpr.italic_cs = Some(val);
        self
    }

    /// Small caps.
    pub fn small_caps(mut self, val: bool) -> Self {
        self.rpr().small_caps = Some(val);
        self
    }

    /// Letter spacing (character spacing) in points.
    pub fn letter_spacing(mut self, pt: f64) -> Self {
        self.rpr().spacing = Some(zavora_docx_oxml::units::Twips((pt * 20.0).round() as i32));
        self
    }

    /// Set paragraph properties for this style.
    pub fn paragraph_properties(mut self, ppr: CT_PPr) -> Self {
        self.style.ppr = Some(ppr);
        self
    }

    /// Set run properties for this style.
    pub fn run_properties(mut self, rpr: CT_RPr) -> Self {
        self.style.rpr = Some(rpr);
        self
    }

    /// Build the style (consumed by Document::add_style).
    pub(crate) fn build(self) -> CT_Style {
        self.style
    }
}

/// Resolve the effective paragraph properties by walking the style inheritance chain.
pub fn resolve_paragraph_properties(style_id: Option<&str>, styles: &CT_Styles) -> CT_PPr {
    let mut effective = CT_PPr::default();

    // Start from docDefaults
    if let Some(ref defaults) = styles.doc_defaults
        && let Some(ref ppr) = defaults.ppr
    {
        effective.merge_from(ppr);
    }

    // Walk the basedOn chain
    if let Some(sid) = style_id {
        let chain = collect_style_chain(sid, styles);
        // Apply from most-base to most-derived
        for style in chain.iter().rev() {
            if let Some(ref ppr) = style.ppr {
                effective.merge_from(ppr);
            }
        }
    } else {
        // Apply the default paragraph style
        if let Some(default_style) = styles.get_default(StyleType::Paragraph)
            && let Some(ref ppr) = default_style.ppr
        {
            effective.merge_from(ppr);
        }
    }

    effective
}

/// Resolve the effective run properties by walking the style inheritance chain.
pub fn resolve_run_properties(
    para_style_id: Option<&str>,
    run_style_id: Option<&str>,
    styles: &CT_Styles,
) -> CT_RPr {
    let mut effective = CT_RPr::default();

    // Start from docDefaults
    if let Some(ref defaults) = styles.doc_defaults
        && let Some(ref rpr) = defaults.rpr
    {
        effective.merge_from(rpr);
    }

    // Apply paragraph style's rpr
    let para_sid = para_style_id.or_else(|| {
        styles
            .get_default(StyleType::Paragraph)
            .map(|s| s.style_id.as_str())
    });
    if let Some(sid) = para_sid {
        let chain = collect_style_chain(sid, styles);
        for style in chain.iter().rev() {
            if let Some(ref rpr) = style.rpr {
                effective.merge_from(rpr);
            }
        }
    }

    // Apply character style's rpr
    if let Some(sid) = run_style_id {
        let chain = collect_style_chain(sid, styles);
        for style in chain.iter().rev() {
            if let Some(ref rpr) = style.rpr {
                effective.merge_from(rpr);
            }
        }
    }

    effective
}

/// Collect the chain of styles from the given style up through basedOn ancestors.
fn collect_style_chain<'a>(style_id: &str, styles: &'a CT_Styles) -> Vec<&'a CT_Style> {
    let mut chain = Vec::new();
    let mut current_id = Some(style_id.to_string());
    let mut seen = std::collections::HashSet::new();

    while let Some(ref sid) = current_id {
        if !seen.insert(sid.clone()) {
            break; // Prevent cycles
        }
        if let Some(style) = styles.get_by_id(sid) {
            chain.push(style);
            current_id = style.based_on.clone();
        } else {
            break;
        }
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use zavora_docx_oxml::units::{HalfPoint, Twips};

    fn test_styles() -> CT_Styles {
        let mut styles = CT_Styles::new_default();

        // Add a custom style based on Heading1 (unique id avoids colliding with
        // the shipped Heading2 in new_default).
        styles.styles.push(CT_Style {
            style_id: "MyHeading2".to_string(),
            style_type: StyleType::Paragraph,
            name: Some("my heading 2".to_string()),
            based_on: Some("Heading1".to_string()),
            next_style: Some("Normal".to_string()),
            is_default: false,
            ppr: Some(CT_PPr {
                space_before: Some(Twips(40)), // Override Heading1's 240
                ..Default::default()
            }),
            rpr: Some(CT_RPr {
                sz: Some(HalfPoint(26)), // Override Heading1's 32
                color: Some("2E74B5".to_string()),
                ..Default::default()
            }),
            extra_xml: None,
        });

        styles
    }

    #[test]
    fn resolve_normal_paragraph() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Normal"), &styles);
        // Should have docDefaults' spacing
        assert_eq!(ppr.space_after, Some(Twips(160)));
    }

    #[test]
    fn resolve_heading1() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Heading1"), &styles);
        // keepNext from Heading1
        assert_eq!(ppr.keep_next, Some(true));
        // spaceBefore from Heading1 (overrides docDefaults which has none)
        assert_eq!(ppr.space_before, Some(Twips(240)));
        // spaceAfter from Heading1 overrides docDefaults
        assert_eq!(ppr.space_after, Some(Twips(0)));
    }

    #[test]
    fn resolve_heading2_inherits_heading1() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("MyHeading2"), &styles);
        // keepNext inherited from Heading1
        assert_eq!(ppr.keep_next, Some(true));
        // spaceBefore overridden by MyHeading2
        assert_eq!(ppr.space_before, Some(Twips(40)));
    }

    #[test]
    fn resolve_heading2_rpr() {
        let styles = test_styles();
        let rpr = resolve_run_properties(Some("MyHeading2"), None, &styles);
        // Font inherited from Heading1's major theme font
        assert_eq!(rpr.font_ascii_theme, Some("majorHAnsi".to_string()));
        // Size overridden by MyHeading2 (not Heading1's 32)
        assert_eq!(rpr.sz, Some(HalfPoint(26)));
        // Bold inherited from Heading1
        assert_eq!(rpr.bold, Some(true));
        // Color from MyHeading2
        assert_eq!(rpr.color, Some("2E74B5".to_string()));
    }

    #[test]
    fn resolve_default_when_no_style() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(None, &styles);
        // Should get docDefaults
        assert_eq!(ppr.space_after, Some(Twips(160)));
    }
}
