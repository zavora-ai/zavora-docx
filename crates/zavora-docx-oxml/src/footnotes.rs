//! Footnote and endnote elements: `CT_Footnotes`, `CT_Footnote`.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::io::Write;

use crate::error::Result;
use crate::namespace::{W_NS, matches_local_name};
use crate::text::CT_P;

/// A single footnote or endnote.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Footnote {
    /// Footnote ID (matches w:footnoteReference w:id in the document body).
    pub id: i32,
    /// Paragraphs making up the footnote content.
    pub paragraphs: Vec<CT_P>,
}

/// Collection of footnotes parsed from `word/footnotes.xml`.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Footnotes {
    pub footnotes: Vec<CT_Footnote>,
    /// Mandatory separator/continuation items (id 0 and -1) preserved verbatim.
    pub separators: Vec<Vec<u8>>,
}

#[allow(non_snake_case)]
impl CT_Footnotes {
    pub fn new() -> Self {
        CT_Footnotes {
            footnotes: Vec::new(),
            separators: Vec::new(),
        }
    }

    /// Get a footnote by its ID.
    pub fn get_by_id(&self, id: i32) -> Option<&CT_Footnote> {
        self.footnotes.iter().find(|f| f.id == id)
    }

    /// Parse from XML bytes (the content of footnotes.xml).
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);

        let mut footnotes = Vec::new();
        let mut separators: Vec<Vec<u8>> = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_local_name(name.as_ref(), b"footnote")
                        || matches_local_name(name.as_ref(), b"endnote")
                    {
                        let mut id: i32 = 0;
                        for attr in e.attributes().flatten() {
                            if matches_local_name(attr.key.as_ref(), b"id") {
                                id = std::str::from_utf8(&attr.value)
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0);
                            }
                        }

                        // Preserve separator/continuation items (id 0 and -1) verbatim.
                        if id <= 0 {
                            separators.push(crate::raw_xml::capture_element(&mut reader, e)?);
                        } else {
                            let paragraphs = parse_footnote_content(&mut reader)?;
                            footnotes.push(CT_Footnote { id, paragraphs });
                        }
                    } else if matches_local_name(name.as_ref(), b"footnotes")
                        || matches_local_name(name.as_ref(), b"endnotes")
                    {
                        // Continue into the root element
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Footnotes {
            footnotes,
            separators,
        })
    }

    /// Serialize to XML bytes as footnotes.
    pub fn to_xml_footnotes(&self) -> Result<Vec<u8>> {
        self.to_xml_root("w:footnotes", "w:footnote")
    }

    /// Serialize to XML bytes as endnotes.
    pub fn to_xml_endnotes(&self) -> Result<Vec<u8>> {
        self.to_xml_root("w:endnotes", "w:endnote")
    }

    fn to_xml_root(&self, root_tag: &str, item_tag: &str) -> Result<Vec<u8>> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut start = BytesStart::new(root_tag);
        start.push_attribute(("xmlns:w", W_NS));
        start.push_attribute((
            "xmlns:r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ));
        writer.write_event(Event::Start(start))?;

        // Separator/continuation items must come first.
        for raw in &self.separators {
            writer.get_mut().write_all(raw)?;
        }

        let mut buf = itoa::Buffer::new();
        for footnote in &self.footnotes {
            let mut fn_start = BytesStart::new(item_tag);
            fn_start.push_attribute(("w:id", buf.format(footnote.id)));
            writer.write_event(Event::Start(fn_start))?;

            for p in &footnote.paragraphs {
                p.to_xml(&mut writer)?;
            }

            writer.write_event(Event::End(BytesEnd::new(item_tag)))?;
        }

        writer.write_event(Event::End(BytesEnd::new(root_tag)))?;

        Ok(writer.into_inner())
    }
}

impl Default for CT_Footnotes {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse the content of a single footnote/endnote (paragraphs until closing tag).
fn parse_footnote_content(reader: &mut Reader<&[u8]>) -> Result<Vec<CT_P>> {
    let mut paragraphs = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if matches_local_name(name.as_ref(), b"p") {
                    paragraphs.push(CT_P::from_xml(reader)?);
                } else {
                    reader.read_to_end_into(name, &mut Vec::new())?;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if matches_local_name(name.as_ref(), b"footnote")
                    || matches_local_name(name.as_ref(), b"endnote")
                {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => {}
        }
        buf.clear();
    }

    Ok(paragraphs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_footnotes_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:footnote w:id="0">
                <w:p><w:r><w:t>separator</w:t></w:r></w:p>
            </w:footnote>
            <w:footnote w:id="1">
                <w:p><w:r><w:t>First footnote text.</w:t></w:r></w:p>
            </w:footnote>
            <w:footnote w:id="2">
                <w:p><w:r><w:t>Second footnote.</w:t></w:r></w:p>
                <w:p><w:r><w:t>With two paragraphs.</w:t></w:r></w:p>
            </w:footnote>
        </w:footnotes>"#;

        let footnotes = CT_Footnotes::from_xml(xml).unwrap();
        // id=0 (separator) is skipped
        assert_eq!(footnotes.footnotes.len(), 2);
        assert_eq!(footnotes.footnotes[0].id, 1);
        assert_eq!(footnotes.footnotes[0].paragraphs.len(), 1);
        assert_eq!(
            footnotes.footnotes[0].paragraphs[0].text(),
            "First footnote text."
        );
        assert_eq!(footnotes.footnotes[1].id, 2);
        assert_eq!(footnotes.footnotes[1].paragraphs.len(), 2);
    }

    #[test]
    fn parse_endnotes_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:endnote w:id="0">
                <w:p><w:r><w:t>separator</w:t></w:r></w:p>
            </w:endnote>
            <w:endnote w:id="1">
                <w:p><w:r><w:t>An endnote.</w:t></w:r></w:p>
            </w:endnote>
        </w:endnotes>"#;

        let endnotes = CT_Footnotes::from_xml(xml).unwrap();
        assert_eq!(endnotes.footnotes.len(), 1);
        assert_eq!(endnotes.footnotes[0].id, 1);
    }

    #[test]
    fn get_footnote_by_id() {
        let footnotes = CT_Footnotes {
            footnotes: vec![
                CT_Footnote {
                    id: 1,
                    paragraphs: vec![],
                },
                CT_Footnote {
                    id: 2,
                    paragraphs: vec![],
                },
            ],
            separators: vec![],
        };
        assert!(footnotes.get_by_id(1).is_some());
        assert!(footnotes.get_by_id(2).is_some());
        assert!(footnotes.get_by_id(3).is_none());
    }

    #[test]
    fn round_trip_footnotes() {
        let mut fn1_para = CT_P::new();
        fn1_para.add_run("First footnote.");

        let mut fn2_para = CT_P::new();
        fn2_para.add_run("Second footnote.");

        let footnotes = CT_Footnotes {
            footnotes: vec![
                CT_Footnote {
                    id: 1,
                    paragraphs: vec![fn1_para],
                },
                CT_Footnote {
                    id: 2,
                    paragraphs: vec![fn2_para],
                },
            ],
            separators: vec![],
        };

        let xml = footnotes.to_xml_footnotes().unwrap();
        let parsed = CT_Footnotes::from_xml(&xml).unwrap();
        assert_eq!(parsed.footnotes.len(), 2);
        assert_eq!(parsed.footnotes[0].id, 1);
        assert_eq!(parsed.footnotes[0].paragraphs[0].text(), "First footnote.");
        assert_eq!(parsed.footnotes[1].id, 2);
        assert_eq!(parsed.footnotes[1].paragraphs[0].text(), "Second footnote.");
    }

    #[test]
    fn preserves_separator_footnotes() {
        // Word's mandatory separator (id 0) and continuation (id -1) items must
        // survive a round-trip, or footnotes render without the separator line.
        let xml = br#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p><w:r><w:t>Note.</w:t></w:r></w:p></w:footnote></w:footnotes>"#;
        let parsed = CT_Footnotes::from_xml(xml).unwrap();
        assert_eq!(parsed.footnotes.len(), 1);
        assert_eq!(parsed.separators.len(), 2);
        let out = String::from_utf8(parsed.to_xml_footnotes().unwrap()).unwrap();
        assert!(out.contains("w:separator"), "separator lost: {out}");
        assert!(out.contains("continuationSeparator"), "continuation lost: {out}");
    }
}
