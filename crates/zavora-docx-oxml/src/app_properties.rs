//! Extended document properties from `docProps/app.xml`.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::error::Result;

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
        reader.config_mut().trim_text(true);
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
                        let v = e.unescape().unwrap_or_default().to_string();
                        if !v.is_empty() {
                            match t.as_str() {
                                "Application" => p.application = Some(v),
                                "AppVersion" => p.app_version = Some(v),
                                "Company" => p.company = Some(v),
                                "Template" => p.template = Some(v),
                                "Pages" => p.pages = v.parse().ok(),
                                "Words" => p.words = v.parse().ok(),
                                "Characters" => p.characters = v.parse().ok(),
                                _ => {}
                            }
                        }
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
            company: Some("Zavora".into()),
            pages: Some(3),
            words: Some(120),
            ..Default::default()
        };
        let xml = p.to_xml().unwrap();
        let s = String::from_utf8(xml.clone()).unwrap();
        assert!(s.contains("<Company>Zavora</Company>"), "{s}");
        assert!(s.contains("<Words>120</Words>"), "{s}");
        let parsed = AppProperties::from_xml(&xml).unwrap();
        assert_eq!(parsed.company.as_deref(), Some("Zavora"));
        assert_eq!(parsed.pages, Some(3));
    }
}
