//! What a hand edit costs.
//!
//! The interface changes a paragraph by asking the capability server for
//! `update_paragraph_text`, which removes the content at that index and inserts a plain paragraph
//! in its place. So the question is not whether the new words arrive — they do — but what goes
//! with the old ones: the style, the formatting inside the paragraph, and whatever else shares
//! that index.
//!
//! The spreadsheet side had one bug of exactly this shape, and it survived because every test
//! looked at the thing that was edited rather than everything that was not.

use std::io::Read;

use zavora_docx::{Document, Length};

fn body_of(path: &str) -> String {
    let file = std::fs::File::open(path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut out = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    out
}

fn parts_of(path: &str) -> Vec<String> {
    let file = std::fs::File::open(path).unwrap();
    let zip = zip::ZipArchive::new(file).unwrap();
    zip.file_names().map(str::to_string).collect()
}

/// A document with the furniture a real one has.
fn a_realistic_document(path: &str) {
    let _ = std::fs::remove_file(path);
    let mut doc = Document::new();

    doc.add_paragraph("Master Services Agreement")
        .style("Heading1");
    doc.add_paragraph("This agreement is made between the parties named below.");
    doc.add_paragraph("1. Definitions").style("Heading2");
    {
        // A paragraph with formatting inside it, which is most paragraphs in a real document.
        let mut p = doc.add_paragraph("The term ");
        p.add_run("Services").bold(true);
        p.add_run(" means the work described in Schedule A.");
    }

    {
        let mut table = doc.add_table(2, 2);
        table.cell(0, 0).unwrap().set_text("Item");
        table.cell(0, 1).unwrap().set_text("Amount");
        table.cell(1, 0).unwrap().set_text("Retainer");
        table.cell(1, 1).unwrap().set_text("5,000");
    }

    doc.set_header("Zavora — commercial in confidence");
    doc.set_footer("Schedule A follows");

    let png: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    doc.add_picture(&png, "logo.png", Length::inches(1.0), Length::inches(1.0));

    doc.save(path).expect("saving should work");
}

/// The fixture has to contain what the tests below ask about, or they prove nothing.
#[test]
fn the_fixture_has_the_furniture() {
    let path = "/tmp/zavora-doc-furniture.docx";
    a_realistic_document(path);

    let body = body_of(path);
    let parts = parts_of(path);
    assert!(body.contains("<w:tbl"), "a table");
    assert!(body.contains("Heading1"), "a styled heading");
    assert!(
        body.contains("<w:b/>") || body.contains("<w:b "),
        "bold text"
    );
    assert!(
        parts.iter().any(|n| n.contains("header")),
        "a header: {parts:?}"
    );
    assert!(
        parts.iter().any(|n| n.contains("media/")),
        "a picture: {parts:?}"
    );
}

/// Editing a paragraph elsewhere must not take the rest of the document with it.
#[test]
fn changing_one_paragraph_keeps_the_rest() {
    let path = "/tmp/zavora-doc-elsewhere.docx";
    a_realistic_document(path);
    let tables_before = body_of(path).matches("<w:tbl>").count();

    let mut doc = Document::open(path).expect("reopening");
    doc.set_paragraph_text(1, "This agreement is made between Zavora and the client.");
    doc.save(path).expect("saving");

    let body = body_of(path);
    let parts = parts_of(path);
    assert!(
        body.contains("Zavora and the client"),
        "the change should be there"
    );
    assert_eq!(
        body.matches("<w:tbl>").count(),
        tables_before,
        "the table went missing when a different paragraph was changed"
    );
    assert!(
        body.contains("Master Services Agreement"),
        "the heading text went"
    );
    assert!(body.contains("Heading1"), "the heading's style went");
    for wanted in ["header", "footer", "media/"] {
        assert!(
            parts.iter().any(|name| name.contains(wanted)),
            "{wanted} was lost by editing a paragraph: {parts:?}"
        );
    }
}

/// Editing a heading's words must leave it a heading.
#[test]
fn changing_a_headings_words_leaves_it_a_heading() {
    let path = "/tmp/zavora-doc-heading.docx";
    a_realistic_document(path);

    let mut doc = Document::open(path).expect("reopening");
    assert!(
        doc.set_paragraph_text(0, "Master Services Agreement (2026)"),
        "paragraph 0 is a paragraph"
    );
    doc.save(path).expect("saving");

    let body = body_of(path);
    assert!(
        body.contains("Master Services Agreement (2026)"),
        "the new words"
    );
    assert!(
        body.contains("Heading1"),
        "the heading became body text: changing its words took its style"
    );
}

/// A table sits in the content sequence too. Editing "paragraph 4" must not delete it.
#[test]
fn an_index_that_lands_on_a_table_does_not_delete_it() {
    let path = "/tmp/zavora-doc-table-index.docx";
    a_realistic_document(path);
    let tables_before = body_of(path).matches("<w:tbl>").count();

    let mut doc = Document::open(path).expect("reopening");
    // The fifth thing in this document is the table. Asked to set its text, the answer must be
    // "that is not a paragraph" rather than replacing it — a table quietly becoming a line of
    // text is the worst outcome available here.
    assert!(
        !doc.set_paragraph_text(4, "Replaced"),
        "a table is not a paragraph and must be refused"
    );
    doc.save(path).expect("saving");

    let body = body_of(path);
    assert_eq!(
        body.matches("<w:tbl>").count(),
        tables_before,
        "editing by index deleted a table"
    );
}
