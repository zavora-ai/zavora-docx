//! `word/settings.xml` — document-level settings (CT_Settings).
//!
//! Typed for the settings we author; every other child is captured verbatim
//! into `extra_xml` so opened documents round-trip losslessly.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::io::Write;

use crate::error::Result;
use crate::namespace::{matches_local_name, W_NS};
use crate::units::Twips;

/// Proofing/theme language triple (`w:themeFontLang`): (val, eastAsia, bidi).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThemeFontLang {
    pub val: Option<String>,
    pub east_asia: Option<String>,
    pub bidi: Option<String>,
}

/// `word/settings.xml` root.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(non_camel_case_types)]
pub struct CT_Settings {
    pub update_fields: bool,
    pub even_odd_headers: bool,
    pub auto_hyphenation: bool,
    /// Document protection edit mode (e.g. "readOnly"), enforcement implied.
    pub protection: Option<String>,
    pub default_tab_stop: Option<Twips>,
    pub mirror_margins: bool,
    pub track_changes: bool,
    pub zoom_percent: Option<u32>,
    pub theme_font_lang: Option<ThemeFontLang>,
    /// Unknown children captured verbatim for round-trip.
    pub extra_xml: Vec<Vec<u8>>,
}

#[allow(non_snake_case)]
impl CT_Settings {
    /// True when nothing needs to be written.
    pub fn is_empty(&self) -> bool {
        !self.update_fields
            && !self.even_odd_headers
            && !self.auto_hyphenation
            && self.protection.is_none()
            && self.default_tab_stop.is_none()
            && !self.mirror_margins
            && !self.track_changes
            && self.zoom_percent.is_none()
            && self.theme_font_lang.is_none()
            && self.extra_xml.is_empty()
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);
        let mut s = CT_Settings::default();
        let mut buf = Vec::new();
        loop {
            // Read into a fresh event we can branch on by kind.
            let ev = reader.read_event_into(&mut buf)?;
            match ev {
                Event::Start(e) => {
                    let name = e.name();
                    let local = name.as_ref().to_vec();
                    if matches_local_name(&local, b"settings") {
                        // root; descend
                    } else if !s.apply_known(&local, &e) {
                        // Unknown element with children: capture the whole subtree.
                        s.extra_xml.push(crate::raw_xml::capture_element(&mut reader, &e)?);
                    }
                }
                Event::Empty(e) => {
                    let name = e.name();
                    let local = name.as_ref().to_vec();
                    if !matches_local_name(&local, b"settings") && !s.apply_known(&local, &e) {
                        s.extra_xml.push(crate::raw_xml::capture_empty_element(&e)?);
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(s)
    }

    /// Apply a recognized child element; returns true if handled.
    fn apply_known(&mut self, local: &[u8], e: &BytesStart) -> bool {
        if matches_local_name(local, b"updateFields") {
            self.update_fields = true;
        } else if matches_local_name(local, b"evenAndOddHeaders") {
            self.even_odd_headers = true;
        } else if matches_local_name(local, b"autoHyphenation") {
            self.auto_hyphenation = true;
        } else if matches_local_name(local, b"mirrorMargins") {
            self.mirror_margins = true;
        } else if matches_local_name(local, b"trackChanges") {
            self.track_changes = true;
        } else if matches_local_name(local, b"documentProtection") {
            self.protection = attr(e, b"edit");
        } else if matches_local_name(local, b"defaultTabStop") {
            self.default_tab_stop = attr(e, b"val").and_then(|v| v.parse().ok()).map(Twips);
        } else if matches_local_name(local, b"zoom") {
            self.zoom_percent = attr(e, b"percent").and_then(|v| v.parse().ok());
        } else if matches_local_name(local, b"themeFontLang") {
            self.theme_font_lang = Some(ThemeFontLang {
                val: attr(e, b"val"),
                east_asia: attr(e, b"eastAsia"),
                bidi: attr(e, b"bidi"),
            });
        } else {
            return false;
        }
        true
    }

    pub fn to_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), Some("yes"))))?;
        let mut root = BytesStart::new("w:settings");
        root.push_attribute(("xmlns:w", W_NS));
        writer.write_event(Event::Start(root))?;

        if self.zoom_percent.is_some() {
            let mut z = BytesStart::new("w:zoom");
            let v = self.zoom_percent.unwrap().to_string();
            z.push_attribute(("w:percent", v.as_str()));
            writer.write_event(Event::Empty(z))?;
        }
        if self.mirror_margins {
            writer.write_event(Event::Empty(BytesStart::new("w:mirrorMargins")))?;
        }
        if self.track_changes {
            writer.write_event(Event::Empty(BytesStart::new("w:trackChanges")))?;
        }
        if let Some(ref edit) = self.protection {
            let mut p = BytesStart::new("w:documentProtection");
            p.push_attribute(("w:edit", edit.as_str()));
            p.push_attribute(("w:enforcement", "1"));
            writer.write_event(Event::Empty(p))?;
        }
        if let Some(tab) = self.default_tab_stop {
            let mut t = BytesStart::new("w:defaultTabStop");
            let v = tab.0.to_string();
            t.push_attribute(("w:val", v.as_str()));
            writer.write_event(Event::Empty(t))?;
        }
        if self.auto_hyphenation {
            let mut h = BytesStart::new("w:autoHyphenation");
            h.push_attribute(("w:val", "true"));
            writer.write_event(Event::Empty(h))?;
        }
        if let Some(ref l) = self.theme_font_lang {
            let mut e = BytesStart::new("w:themeFontLang");
            if let Some(ref v) = l.val {
                e.push_attribute(("w:val", v.as_str()));
            }
            if let Some(ref v) = l.east_asia {
                e.push_attribute(("w:eastAsia", v.as_str()));
            }
            if let Some(ref v) = l.bidi {
                e.push_attribute(("w:bidi", v.as_str()));
            }
            writer.write_event(Event::Empty(e))?;
        }
        if self.even_odd_headers {
            writer.write_event(Event::Empty(BytesStart::new("w:evenAndOddHeaders")))?;
        }
        if self.update_fields {
            let mut u = BytesStart::new("w:updateFields");
            u.push_attribute(("w:val", "true"));
            writer.write_event(Event::Empty(u))?;
        }
        for raw in &self.extra_xml {
            writer.get_mut().write_all(raw)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:settings")))?;
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        self.to_xml(&mut Writer::new(&mut out))?;
        Ok(out)
    }
}

fn attr(e: &BytesStart, key: &[u8]) -> Option<String> {
    e.attributes().flatten().find_map(|a| {
        let an = a.key.as_ref();
        // match local name ignoring w: prefix
        let matches = an == key || an.rsplit(|&b| b == b':').next() == Some(key);
        if matches {
            String::from_utf8(a.value.into_owned()).ok()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_typed_settings() {
        let s = CT_Settings {
            mirror_margins: true,
            default_tab_stop: Some(Twips(720)),
            zoom_percent: Some(150),
            ..Default::default()
        };
        let xml = String::from_utf8(s.to_bytes().unwrap()).unwrap();
        assert!(xml.contains("w:mirrorMargins"), "{xml}");
        assert!(xml.contains(r#"w:val="720""#), "{xml}");
        assert!(xml.contains(r#"w:percent="150""#), "{xml}");
    }

    #[test]
    fn preserves_unknown_settings() {
        let xml = r#"<?xml version="1.0"?><w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:mirrorMargins/><w:doNotExpandShiftReturn/></w:settings>"#;
        let s = CT_Settings::from_xml(xml.as_bytes()).unwrap();
        assert!(s.mirror_margins);
        let out = String::from_utf8(s.to_bytes().unwrap()).unwrap();
        assert!(out.contains("w:doNotExpandShiftReturn"), "lost unknown: {out}");
    }

    #[test]
    fn round_trips_lang() {
        let xml = r#"<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:themeFontLang w:val="fr-FR"/></w:settings>"#;
        let s = CT_Settings::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(s.theme_font_lang.as_ref().unwrap().val.as_deref(), Some("fr-FR"));
        let out = String::from_utf8(s.to_bytes().unwrap()).unwrap();
        assert!(out.contains(r#"w:val="fr-FR""#), "{out}");
    }
}
