//! Shapes and text boxes — DrawingML `wps:wsp` construction.
//!
//! Builds a self-contained inline drawing (`w:drawing > wp:inline > a:graphic >
//! wps:wsp`) emitted as a run inside a paragraph. Like equations, the drawing
//! is run content, so callers wrap the result in a `w:p`.

use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

use crate::error::Result;
use crate::units::Emu;

const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const WPS_NS: &str = "http://schemas.microsoft.com/office/word/2010/wordprocessingShape";
const WP_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";

/// A buildable shape or text box.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub width: Emu,
    pub height: Emu,
    /// Preset geometry name (e.g. "rect", "ellipse", "roundRect", "rightArrow").
    pub geom: String,
    /// Optional solid fill color (hex, e.g. "DDEEFF").
    pub fill: Option<String>,
    /// Optional outline (hex color, width in EMU).
    pub line: Option<(String, i64)>,
    /// Paragraph text lines rendered inside the shape (empty = bare shape).
    pub text: Vec<String>,
    /// Shape name shown in Word.
    pub name: String,
}

impl Shape {
    /// A rectangular text box containing the given lines.
    pub fn text_box(width: Emu, height: Emu, lines: Vec<String>) -> Self {
        Shape {
            width,
            height,
            geom: "rect".to_string(),
            fill: None,
            line: Some(("000000".to_string(), 6350)),
            text: lines,
            name: "TextBox".to_string(),
        }
    }

    /// A preset shape with no text.
    pub fn preset(width: Emu, height: Emu, geom: &str) -> Self {
        Shape {
            width,
            height,
            geom: geom.to_string(),
            fill: None,
            line: None,
            text: Vec::new(),
            name: "Shape".to_string(),
        }
    }

    /// Serialize the full `w:r > w:drawing` run.
    pub fn to_run_bytes(&self) -> Result<Vec<u8>> {
        let mut w = Writer::new(Vec::new());
        let mut buf = itoa::Buffer::new();
        w.write_event(Event::Start(BytesStart::new("w:r")))?;
        w.write_event(Event::Start(BytesStart::new("w:drawing")))?;

        let mut inl = BytesStart::new("wp:inline");
        inl.push_attribute(("xmlns:wp", WP_NS));
        for a in ["distT", "distB", "distL", "distR"] {
            inl.push_attribute((a, "0"));
        }
        w.write_event(Event::Start(inl))?;

        let mut ext = BytesStart::new("wp:extent");
        ext.push_attribute(("cx", buf.format(self.width.0)));
        ext.push_attribute(("cy", buf.format(self.height.0)));
        w.write_event(Event::Empty(ext))?;
        let mut dp = BytesStart::new("wp:docPr");
        dp.push_attribute(("id", "1"));
        dp.push_attribute(("name", self.name.as_str()));
        w.write_event(Event::Empty(dp))?;

        let mut g = BytesStart::new("a:graphic");
        g.push_attribute(("xmlns:a", A_NS));
        w.write_event(Event::Start(g))?;
        let mut gd = BytesStart::new("a:graphicData");
        gd.push_attribute(("uri", WPS_NS));
        w.write_event(Event::Start(gd))?;

        let mut wsp = BytesStart::new("wps:wsp");
        wsp.push_attribute(("xmlns:wps", WPS_NS));
        w.write_event(Event::Start(wsp))?;

        // cNvSpPr — mark as text box when it holds text.
        let mut cnv = BytesStart::new("wps:cNvSpPr");
        if !self.text.is_empty() {
            cnv.push_attribute(("txBox", "1"));
        }
        w.write_event(Event::Empty(cnv))?;

        // spPr — geometry, fill, line.
        w.write_event(Event::Start(BytesStart::new("wps:spPr")))?;
        w.write_event(Event::Start(BytesStart::new("a:xfrm")))?;
        let mut off = BytesStart::new("a:off");
        off.push_attribute(("x", "0"));
        off.push_attribute(("y", "0"));
        w.write_event(Event::Empty(off))?;
        let mut e2 = BytesStart::new("a:ext");
        e2.push_attribute(("cx", buf.format(self.width.0)));
        e2.push_attribute(("cy", buf.format(self.height.0)));
        w.write_event(Event::Empty(e2))?;
        w.write_event(Event::End(BytesEnd::new("a:xfrm")))?;
        let mut prst = BytesStart::new("a:prstGeom");
        prst.push_attribute(("prst", self.geom.as_str()));
        w.write_event(Event::Start(prst))?;
        w.write_event(Event::Empty(BytesStart::new("a:avLst")))?;
        w.write_event(Event::End(BytesEnd::new("a:prstGeom")))?;
        if let Some(ref fill) = self.fill {
            w.write_event(Event::Start(BytesStart::new("a:solidFill")))?;
            let mut c = BytesStart::new("a:srgbClr");
            c.push_attribute(("val", fill.as_str()));
            w.write_event(Event::Empty(c))?;
            w.write_event(Event::End(BytesEnd::new("a:solidFill")))?;
        }
        if let Some((ref color, width)) = self.line {
            let mut ln = BytesStart::new("a:ln");
            ln.push_attribute(("w", buf.format(width)));
            w.write_event(Event::Start(ln))?;
            w.write_event(Event::Start(BytesStart::new("a:solidFill")))?;
            let mut c = BytesStart::new("a:srgbClr");
            c.push_attribute(("val", color.as_str()));
            w.write_event(Event::Empty(c))?;
            w.write_event(Event::End(BytesEnd::new("a:solidFill")))?;
            w.write_event(Event::End(BytesEnd::new("a:ln")))?;
        }
        w.write_event(Event::End(BytesEnd::new("wps:spPr")))?;

        // txbx — text box content (always emit at least one paragraph if text).
        if !self.text.is_empty() {
            w.write_event(Event::Start(BytesStart::new("wps:txbx")))?;
            w.write_event(Event::Start(BytesStart::new("w:txbxContent")))?;
            for line in &self.text {
                w.write_event(Event::Start(BytesStart::new("w:p")))?;
                w.write_event(Event::Start(BytesStart::new("w:r")))?;
                let mut t = BytesStart::new("w:t");
                t.push_attribute(("xml:space", "preserve"));
                w.write_event(Event::Start(t))?;
                w.write_event(Event::Text(BytesText::new(line)))?;
                w.write_event(Event::End(BytesEnd::new("w:t")))?;
                w.write_event(Event::End(BytesEnd::new("w:r")))?;
                w.write_event(Event::End(BytesEnd::new("w:p")))?;
            }
            w.write_event(Event::End(BytesEnd::new("w:txbxContent")))?;
            w.write_event(Event::End(BytesEnd::new("wps:txbx")))?;
        }

        // bodyPr — required for the shape to lay out.
        let mut body = BytesStart::new("wps:bodyPr");
        body.push_attribute(("rot", "0"));
        body.push_attribute(("anchor", "t"));
        w.write_event(Event::Empty(body))?;

        w.write_event(Event::End(BytesEnd::new("wps:wsp")))?;
        w.write_event(Event::End(BytesEnd::new("a:graphicData")))?;
        w.write_event(Event::End(BytesEnd::new("a:graphic")))?;
        w.write_event(Event::End(BytesEnd::new("wp:inline")))?;
        w.write_event(Event::End(BytesEnd::new("w:drawing")))?;
        w.write_event(Event::End(BytesEnd::new("w:r")))?;
        Ok(w.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sh: &Shape) -> String {
        String::from_utf8(sh.to_run_bytes().unwrap()).unwrap()
    }

    #[test]
    fn text_box_has_content() {
        let x = s(&Shape::text_box(
            Emu(2000000),
            Emu(1000000),
            vec!["Hello".into()],
        ));
        assert!(x.contains("wps:wsp"), "{x}");
        assert!(x.contains(r#"txBox="1""#), "{x}");
        assert!(x.contains("<w:txbxContent>"), "{x}");
        assert!(x.contains("<w:t xml:space=\"preserve\">Hello</w:t>"), "{x}");
        assert!(x.contains(r#"prst="rect""#), "{x}");
    }

    #[test]
    fn preset_shape_with_fill() {
        let mut sh = Shape::preset(Emu(900000), Emu(900000), "ellipse");
        sh.fill = Some("FFCC00".into());
        let x = s(&sh);
        assert!(x.contains(r#"prst="ellipse""#), "{x}");
        assert!(x.contains(r#"<a:srgbClr val="FFCC00"/>"#), "{x}");
        assert!(!x.contains("txBox"), "{x}");
    }
}
