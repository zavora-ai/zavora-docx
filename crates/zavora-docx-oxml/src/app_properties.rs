//! Extended document properties from `docProps/app.xml`.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::error::Result;
use crate::xml_compat::{decode_reference, decode_text};

const NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties";

/// Extended properties (`docProps/app.xml`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppProperties {
    pub application: Option<String>,
    pub app_version: Option<String>,
    pub company: Option<String>,
    pub template: Option<String>,
    pub pages: Option<u32>,
    pub words: Option<u32>,
    pub characters: Option<u32>,
}

impl AppProperties {
    pub fn is_empty(&self) -> bool {
        *self == AppProperties::default()
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        // Entity references are emitted as separate events in quick-xml 0.41;
        // trimming each adjacent text event would remove meaningful spaces.
        reader.config_mut().trim_text(false);
        let mut p = AppProperties::default();
        let mut buf = Vec::new();
        let mut tag: Option<String> = None;
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    tag = Some(local(e.name().as_ref()).to_string());
                }
                Event::Text(ref e) => {
                    if let Some(ref t) = tag {
                        let v = decode_text(e);
                        if !v.is_empty() {
                            append_value(&mut p, t, &v);
                        }
                    }
                }
                Event::GeneralRef(ref reference) => {
                    if let Some(ref t) = tag {
                        append_value(&mut p, t, &decode_reference(reference));
                    }
                }
                Event::End(_) => tag = None,
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(p)
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut w = Writer::new(Vec::new());
        w.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;
        let mut root = BytesStart::new("Properties");
        root.push_attribute(("xmlns", NS));
        w.write_event(Event::Start(root))?;

        fn el<W: std::io::Write>(w: &mut Writer<W>, tag: &str, v: &Option<String>) -> Result<()> {
            if let Some(val) = v {
                w.write_event(Event::Start(BytesStart::new(tag)))?;
                w.write_event(Event::Text(BytesText::new(val)))?;
                w.write_event(Event::End(BytesEnd::new(tag)))?;
            }
            Ok(())
        }
        fn num<W: std::io::Write>(w: &mut Writer<W>, tag: &str, v: &Option<u32>) -> Result<()> {
            if let Some(n) = v {
                w.write_event(Event::Start(BytesStart::new(tag)))?;
                w.write_event(Event::Text(BytesText::new(&n.to_string())))?;
                w.write_event(Event::End(BytesEnd::new(tag)))?;
            }
            Ok(())
        }

        el(&mut w, "Template", &self.template)?;
        el(&mut w, "Application", &self.application)?;
        num(&mut w, "Pages", &self.pages)?;
        num(&mut w, "Words", &self.words)?;
        num(&mut w, "Characters", &self.characters)?;
        el(&mut w, "Company", &self.company)?;
        el(&mut w, "AppVersion", &self.app_version)?;

        w.write_event(Event::End(BytesEnd::new("Properties")))?;
        Ok(w.into_inner())
    }
}

fn append_value(properties: &mut AppProperties, tag: &str, value: &str) {
    let field = match tag {
        "Application" => Some(&mut properties.application),
        "AppVersion" => Some(&mut properties.app_version),
        "Company" => Some(&mut properties.company),
        "Template" => Some(&mut properties.template),
        _ => None,
    };
    if let Some(field) = field {
        field.get_or_insert_default().push_str(value);
        return;
    }

    match tag {
        "Pages" => properties.pages = value.trim().parse().ok(),
        "Words" => properties.words = value.trim().parse().ok(),
        "Characters" => properties.characters = value.trim().parse().ok(),
        _ => {}
    }
}

fn local(name: &[u8]) -> &str {
    let s = std::str::from_utf8(name).unwrap_or("");
    s.rsplit(':').next().unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let p = AppProperties {
            application: Some("zavora-docx".into()),
            company: Some("Zavora & Partners".into()),
            pages: Some(3),
            words: Some(120),
            ..Default::default()
        };
        let xml = p.to_xml().unwrap();
        let s = String::from_utf8(xml.clone()).unwrap();
        assert!(
            s.contains("<Company>Zavora &amp; Partners</Company>"),
            "{s}"
        );
        assert!(s.contains("<Words>120</Words>"), "{s}");
        let parsed = AppProperties::from_xml(&xml).unwrap();
        assert_eq!(parsed.company.as_deref(), Some("Zavora & Partners"));
        assert_eq!(parsed.pages, Some(3));
    }
}
