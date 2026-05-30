//! Bookmarks — typed `w:bookmarkStart`/`w:bookmarkEnd` and `REF` cross-references.

use crate::namespace::W_NS;

/// A bookmark spanning a range, identified by id and name.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub struct CT_Bookmark {
    pub id: u32,
    pub name: String,
}

impl CT_Bookmark {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        CT_Bookmark { id, name: name.into() }
    }

    /// `<w:bookmarkStart .../>` element bytes.
    pub fn start_xml(&self) -> Vec<u8> {
        format!(
            r#"<w:bookmarkStart w:id="{}" w:name="{}" xmlns:w="{W_NS}"/>"#,
            self.id, self.name
        )
        .into_bytes()
    }

    /// `<w:bookmarkEnd .../>` element bytes.
    pub fn end_xml(&self) -> Vec<u8> {
        format!(r#"<w:bookmarkEnd w:id="{}" xmlns:w="{W_NS}"/>"#, self.id).into_bytes()
    }

    /// A complex `REF` field run sequence referencing this bookmark by name.
    pub fn cross_reference_xml(&self) -> Vec<u8> {
        format!(
            concat!(
                r#"<w:r xmlns:w="{ns}"><w:fldChar w:fldCharType="begin"/></w:r>"#,
                r#"<w:r xmlns:w="{ns}"><w:instrText xml:space="preserve"> REF {name} \h </w:instrText></w:r>"#,
                r#"<w:r xmlns:w="{ns}"><w:fldChar w:fldCharType="separate"/></w:r>"#,
                r#"<w:r xmlns:w="{ns}"><w:t xml:space="preserve">{name}</w:t></w:r>"#,
                r#"<w:r xmlns:w="{ns}"><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            ns = W_NS,
            name = self.name
        )
        .into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmark_xml() {
        let b = CT_Bookmark::new(1, "chap1");
        let s = String::from_utf8(b.start_xml()).unwrap();
        assert!(s.contains(r#"w:id="1""#) && s.contains(r#"w:name="chap1""#), "{s}");
        let e = String::from_utf8(b.end_xml()).unwrap();
        assert!(e.contains("bookmarkEnd"), "{e}");
        let r = String::from_utf8(b.cross_reference_xml()).unwrap();
        assert!(r.contains("REF chap1"), "{r}");
    }
}
