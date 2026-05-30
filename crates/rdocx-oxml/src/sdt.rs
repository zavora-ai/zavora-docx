//! Structured Document Tags (content controls) — construction path.
//!
//! Parsed SDTs are preserved verbatim as `BodyContent::RawXml`; this module is
//! for *building* new controls. Each kind serializes to a complete `w:sdt`
//! (block-level) that callers push as raw bytes into the body.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use crate::error::Result;

/// The kind of content control to build.
#[derive(Debug, Clone, PartialEq)]
pub enum SdtKind {
    /// Plain-text control.
    Text,
    /// Rich-text control.
    RichText,
    /// Drop-down list of (display, value) options.
    DropDown(Vec<(String, String)>),
    /// Combo box of (display, value) options.
    ComboBox(Vec<(String, String)>),
    /// Date picker with a display format (e.g. "yyyy-MM-dd").
    Date(String),
    /// Checkbox (w14) with initial checked state.
    Checkbox(bool),
}

/// A buildable content control.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub struct CT_Sdt {
    pub kind: SdtKind,
    pub tag: Option<String>,
    pub alias: Option<String>,
    /// Placeholder/display text shown in sdtContent.
    pub text: Option<String>,
}

impl CT_Sdt {
    pub fn new(kind: SdtKind, tag: impl Into<String>) -> Self {
        CT_Sdt {
            kind,
            tag: Some(tag.into()),
            alias: None,
            text: None,
        }
    }

    /// Serialize the whole `w:sdt` element to bytes (block-level).
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut w = Writer::new(Vec::new());
        w.write_event(Event::Start(BytesStart::new("w:sdt")))?;
        self.write_pr(&mut w)?;
        self.write_content(&mut w)?;
        w.write_event(Event::End(BytesEnd::new("w:sdt")))?;
        Ok(w.into_inner())
    }

    fn write_pr<W: std::io::Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_event(Event::Start(BytesStart::new("w:sdtPr")))?;
        if let Some(ref a) = self.alias {
            let mut e = BytesStart::new("w:alias");
            e.push_attribute(("w:val", a.as_str()));
            w.write_event(Event::Empty(e))?;
        }
        if let Some(ref t) = self.tag {
            let mut e = BytesStart::new("w:tag");
            e.push_attribute(("w:val", t.as_str()));
            w.write_event(Event::Empty(e))?;
        }
        match &self.kind {
            SdtKind::Text => w.write_event(Event::Empty(BytesStart::new("w:text")))?,
            SdtKind::RichText => w.write_event(Event::Empty(BytesStart::new("w:richText")))?,
            SdtKind::DropDown(opts) => self.write_list("w:dropDownList", opts, w)?,
            SdtKind::ComboBox(opts) => self.write_list("w:comboBox", opts, w)?,
            SdtKind::Date(fmt) => {
                w.write_event(Event::Start(BytesStart::new("w:date")))?;
                let mut f = BytesStart::new("w:dateFormat");
                f.push_attribute(("w:val", fmt.as_str()));
                w.write_event(Event::Empty(f))?;
                w.write_event(Event::End(BytesEnd::new("w:date")))?;
            }
            SdtKind::Checkbox(checked) => {
                w.write_event(Event::Start(BytesStart::new("w14:checkbox")))?;
                let mut ch = BytesStart::new("w14:checked");
                ch.push_attribute(("w14:val", if *checked { "1" } else { "0" }));
                w.write_event(Event::Empty(ch))?;
                w.write_event(Event::End(BytesEnd::new("w14:checkbox")))?;
            }
        }
        w.write_event(Event::End(BytesEnd::new("w:sdtPr")))?;
        Ok(())
    }

    fn write_list<W: std::io::Write>(
        &self,
        tag: &str,
        opts: &[(String, String)],
        w: &mut Writer<W>,
    ) -> Result<()> {
        w.write_event(Event::Start(BytesStart::new(tag)))?;
        for (display, value) in opts {
            let mut e = BytesStart::new("w:listItem");
            e.push_attribute(("w:displayText", display.as_str()));
            e.push_attribute(("w:value", value.as_str()));
            w.write_event(Event::Empty(e))?;
        }
        w.write_event(Event::End(BytesEnd::new(tag)))?;
        Ok(())
    }

    fn write_content<W: std::io::Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_event(Event::Start(BytesStart::new("w:sdtContent")))?;
        // A single paragraph holding the display/placeholder text.
        let display = match &self.kind {
            SdtKind::Checkbox(checked) => {
                Some(if *checked { "\u{2612}".to_string() } else { "\u{2610}".to_string() })
            }
            _ => self.text.clone(),
        };
        w.write_event(Event::Start(BytesStart::new("w:p")))?;
        w.write_event(Event::Start(BytesStart::new("w:r")))?;
        let mut t = BytesStart::new("w:t");
        t.push_attribute(("xml:space", "preserve"));
        w.write_event(Event::Start(t))?;
        w.write_event(Event::Text(BytesText::new(display.as_deref().unwrap_or(""))))?;
        w.write_event(Event::End(BytesEnd::new("w:t")))?;
        w.write_event(Event::End(BytesEnd::new("w:r")))?;
        w.write_event(Event::End(BytesEnd::new("w:p")))?;
        w.write_event(Event::End(BytesEnd::new("w:sdtContent")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xml(s: &CT_Sdt) -> String {
        String::from_utf8(s.to_bytes().unwrap()).unwrap()
    }

    #[test]
    fn text_control() {
        let mut s = CT_Sdt::new(SdtKind::Text, "name");
        s.text = Some("Enter name".into());
        let x = xml(&s);
        assert!(x.contains("<w:sdt>"), "{x}");
        assert!(x.contains(r#"<w:tag w:val="name"/>"#), "{x}");
        assert!(x.contains("<w:text/>"), "{x}");
        assert!(x.contains("Enter name"), "{x}");
    }

    #[test]
    fn dropdown_control() {
        let s = CT_Sdt::new(
            SdtKind::DropDown(vec![
                ("Yes".into(), "y".into()),
                ("No".into(), "n".into()),
            ]),
            "choice",
        );
        let x = xml(&s);
        assert!(x.contains("<w:dropDownList>"), "{x}");
        assert!(x.contains(r#"w:displayText="Yes""#), "{x}");
        assert!(x.contains(r#"w:value="n""#), "{x}");
    }

    #[test]
    fn date_and_checkbox() {
        let d = xml(&CT_Sdt::new(SdtKind::Date("yyyy-MM-dd".into()), "when"));
        assert!(d.contains(r#"<w:dateFormat w:val="yyyy-MM-dd"/>"#), "{d}");
        let c = xml(&CT_Sdt::new(SdtKind::Checkbox(true), "agree"));
        assert!(c.contains("w14:checkbox"), "{c}");
        assert!(c.contains(r#"w14:val="1""#), "{c}");
    }
}
