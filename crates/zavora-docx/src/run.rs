//! Run — a contiguous stretch of text with uniform formatting.

use zavora_docx_oxml::properties::{CT_RPr, CT_Shd};
use zavora_docx_oxml::shared::ST_Underline;
use zavora_docx_oxml::text::{CT_R, CT_Text, RunContent};
use zavora_docx_oxml::units::{HalfPoint, Twips};

use crate::Length;

/// Underline style for runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderlineStyle {
    None,
    Single,
    Double,
    Thick,
    Dotted,
    Dash,
    Wave,
    Words,
}

impl UnderlineStyle {
    fn to_st(self) -> ST_Underline {
        match self {
            Self::None => ST_Underline::None,
            Self::Single => ST_Underline::Single,
            Self::Double => ST_Underline::Double,
            Self::Thick => ST_Underline::Thick,
            Self::Dotted => ST_Underline::Dotted,
            Self::Dash => ST_Underline::Dash,
            Self::Wave => ST_Underline::Wave,
            Self::Words => ST_Underline::Words,
        }
    }
}

/// A run of text within a paragraph.
///
/// All text in a run shares the same formatting (font, size, bold, etc.).
pub struct Run<'a> {
    pub(crate) inner: &'a mut CT_R,
}

impl<'a> Run<'a> {
    /// Get the text content of this run.
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Set the text content, replacing all existing content.
    pub fn set_text(&mut self, text: &str) {
        self.inner.content = vec![RunContent::Text(CT_Text::new(text))];
    }

    /// Add text to this run.
    pub fn add_text(&mut self, text: &str) {
        self.inner
            .content
            .push(RunContent::Text(CT_Text::new(text)));
    }

    /// Set bold formatting.
    pub fn bold(mut self, val: bool) -> Self {
        let rpr = self.ensure_rpr();
        rpr.bold = Some(val);
        rpr.bold_cs = Some(val);
        self
    }

    /// Set italic formatting.
    pub fn italic(mut self, val: bool) -> Self {
        let rpr = self.ensure_rpr();
        rpr.italic = Some(val);
        rpr.italic_cs = Some(val);
        self
    }

    /// Set underline formatting (simple on/off).
    pub fn underline(mut self, val: bool) -> Self {
        self.ensure_rpr().underline = Some(if val {
            ST_Underline::Single
        } else {
            ST_Underline::None
        });
        self
    }

    /// Set underline with a specific style.
    pub fn underline_style(mut self, style: UnderlineStyle) -> Self {
        self.ensure_rpr().underline = Some(style.to_st());
        self
    }

    /// Set font size in points.
    pub fn size(mut self, pt: f64) -> Self {
        let hp = HalfPoint::from_pt(pt);
        let rpr = self.ensure_rpr();
        rpr.sz = Some(hp);
        rpr.sz_cs = Some(hp);
        self
    }

    /// Set the font name.
    pub fn font(mut self, name: &str) -> Self {
        let rpr = self.ensure_rpr();
        rpr.font_ascii = Some(name.to_string());
        rpr.font_hansi = Some(name.to_string());
        rpr.font_east_asia = Some(name.to_string());
        rpr.font_cs = Some(name.to_string());
        self
    }

    /// Set text color as a hex string (e.g., "FF0000" for red).
    pub fn color(mut self, hex: &str) -> Self {
        self.ensure_rpr().color = Some(hex.to_string());
        self
    }

    /// Set the proofing language for this run (e.g. "en-US", "fr-FR").
    pub fn language(mut self, lang: &str) -> Self {
        self.ensure_rpr().lang = Some(lang.to_string());
        self
    }

    /// Set highlight color as a hex fill value.
    pub fn highlight(mut self, color: &str) -> Self {
        self.ensure_rpr().shading = Some(CT_Shd {
            val: "clear".to_string(),
            color: Some("auto".to_string()),
            fill: Some(color.to_string()),
        });
        self
    }

    /// Set strikethrough formatting.
    pub fn strike(mut self, val: bool) -> Self {
        self.ensure_rpr().strike = Some(val);
        self
    }

    /// Set double strikethrough.
    pub fn double_strike(mut self, val: bool) -> Self {
        self.ensure_rpr().dstrike = Some(val);
        self
    }

    /// Set all caps.
    pub fn all_caps(mut self, val: bool) -> Self {
        self.ensure_rpr().caps = Some(val);
        self
    }

    /// Set small caps.
    pub fn small_caps(mut self, val: bool) -> Self {
        self.ensure_rpr().small_caps = Some(val);
        self
    }

    /// Set superscript.
    pub fn superscript(mut self) -> Self {
        self.ensure_rpr().vert_align = Some("superscript".to_string());
        self
    }

    /// Set subscript.
    pub fn subscript(mut self) -> Self {
        self.ensure_rpr().vert_align = Some("subscript".to_string());
        self
    }

    /// Set character spacing (positive = expanded, negative = condensed).
    pub fn character_spacing(mut self, spacing: Length) -> Self {
        self.ensure_rpr().spacing = Some(spacing.as_twips());
        self
    }

    /// Set character width scale in percent (100 = normal).
    pub fn width_scale(mut self, percent: u32) -> Self {
        self.ensure_rpr().width_scale = Some(percent);
        self
    }

    /// Set text position (positive = raised, negative = lowered) in half-points.
    pub fn position(mut self, half_points: i32) -> Self {
        self.ensure_rpr().position = Some(half_points);
        self
    }

    /// Set hidden/vanish text.
    pub fn hidden(mut self, val: bool) -> Self {
        self.ensure_rpr().vanish = Some(val);
        self
    }

    /// Set the character style by ID.
    pub fn style(mut self, style_id: &str) -> Self {
        self.ensure_rpr().style_id = Some(style_id.to_string());
        self
    }

    /// Set the run color using a theme color name (e.g. "accent1", "accent2", "dk1", "lt1", "hlink").
    pub fn theme_color(mut self, theme: &str) -> Self {
        self.ensure_rpr().color_theme = Some(theme.to_string());
        self
    }

    /// Set the font using a theme font slot ("majorHAnsi" for headings, "minorHAnsi" for body).
    pub fn theme_font(mut self, theme: &str) -> Self {
        self.ensure_rpr().font_ascii_theme = Some(theme.to_string());
        self.ensure_rpr().font_hansi_theme = Some(theme.to_string());
        self
    }

    /// Enable kerning for text at or above the given size in points.
    pub fn kerning(mut self, threshold_pt: f64) -> Self {
        self.ensure_rpr().kern = Some((threshold_pt * 2.0) as u32);
        self
    }

    /// Set ligatures mode: "standard", "all", "standardContextual", "none".
    pub fn ligatures(mut self, mode: &str) -> Self {
        self.ensure_rpr().ligatures = Some(mode.to_string());
        self
    }

    /// Add a text shadow effect (w14:shadow).
    pub fn shadow(mut self, blur_pt: f64, offset_pt: f64, color: &str) -> Self {
        let blur_emu = (blur_pt * 12700.0) as i64;
        let dist_emu = (offset_pt * 12700.0) as i64;
        let xml = format!(
            r#"<w14:shadow w14:blurRad="{}" w14:dist="{}" w14:dir="2700000" w14:sx="100000" w14:sy="100000" w14:algn="tl" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w14:srgbClr w14:val="{}"><w14:alpha w14:val="60000"/></w14:srgbClr></w14:shadow>"#,
            blur_emu, dist_emu, color
        );
        self.ensure_rpr().extra_xml.get_or_insert_with(Vec::new).push(xml.into_bytes());
        self
    }

    /// Add a text glow effect (w14:glow).
    pub fn glow(mut self, radius_pt: f64, color: &str) -> Self {
        let rad_emu = (radius_pt * 12700.0) as i64;
        let xml = format!(
            r#"<w14:glow w14:rad="{}" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w14:srgbClr w14:val="{}"><w14:alpha w14:val="60000"/></w14:srgbClr></w14:glow>"#,
            rad_emu, color
        );
        self.ensure_rpr().extra_xml.get_or_insert_with(Vec::new).push(xml.into_bytes());
        self
    }

    /// Add a text outline effect (w14:textOutline).
    pub fn text_outline(mut self, width_pt: f64, color: &str) -> Self {
        let width_emu = (width_pt * 12700.0) as i64;
        let xml = format!(
            r#"<w14:textOutline w14:w="{}" w14:cap="flat" w14:cmpd="sng" w14:algn="ctr" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w14:solidFill><w14:srgbClr w14:val="{}"/></w14:solidFill><w14:prstDash w14:val="solid"/></w14:textOutline>"#,
            width_emu, color
        );
        self.ensure_rpr().extra_xml.get_or_insert_with(Vec::new).push(xml.into_bytes());
        self
    }

    /// Add a text reflection effect (w14:reflection).
    pub fn reflection(mut self) -> Self {
        let xml = r#"<w14:reflection w14:blurRad="6350" w14:stA="53000" w14:stPos="0" w14:endA="300" w14:endPos="35500" w14:dist="0" w14:dir="5400000" w14:fadeDir="5400000" w14:sx="100000" w14:sy="-90000" w14:kx="0" w14:ky="0" w14:algn="bl" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"/>"#;
        self.ensure_rpr().extra_xml.get_or_insert_with(Vec::new).push(xml.as_bytes().to_vec());
        self
    }

    /// Insert a footnote reference (superscript number linking to footnote).
    pub fn footnote_ref(mut self, id: i32) -> Self {
        self.inner.content.push(RunContent::FootnoteRef { id });
        self.ensure_rpr().vert_align = Some("superscript".to_string());
        self
    }

    /// Insert an endnote reference (superscript number linking to endnote).
    pub fn endnote_ref(mut self, id: i32) -> Self {
        self.inner.content.push(RunContent::EndnoteRef { id });
        self.ensure_rpr().vert_align = Some("superscript".to_string());
        self
    }

    fn ensure_rpr(&mut self) -> &mut CT_RPr {
        self.inner.properties.get_or_insert_with(CT_RPr::default)
    }
}

/// An immutable reference to a run.
pub struct RunRef<'a> {
    pub(crate) inner: &'a CT_R,
}

impl<'a> RunRef<'a> {
    /// Get the text content of this run.
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Check if bold.
    pub fn is_bold(&self) -> bool {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.bold)
            .unwrap_or(false)
    }

    /// Check if italic.
    pub fn is_italic(&self) -> bool {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.italic)
            .unwrap_or(false)
    }

    /// Check if strikethrough.
    pub fn is_strike(&self) -> bool {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.strike)
            .unwrap_or(false)
    }

    /// Get font size in points, if set.
    pub fn size(&self) -> Option<f64> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.sz)
            .map(|hp| hp.to_pt())
    }

    /// Get font name, if set.
    pub fn font_name(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.font_ascii.as_deref())
    }

    /// Get text color, if set.
    pub fn color(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.color.as_deref())
    }

    /// Get character spacing in twips, if set.
    pub fn character_spacing(&self) -> Option<Twips> {
        self.inner.properties.as_ref().and_then(|rpr| rpr.spacing)
    }

    /// Get vertical alignment (superscript/subscript), if set.
    pub fn vert_align(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.vert_align.as_deref())
    }

    /// Get the character style ID, if set.
    pub fn style_id(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.style_id.as_deref())
    }
}
