use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    // Add a title
    let mut p = doc.add_paragraph("");
    p.add_run("Footnotes & Endnotes Demo").font("Arial").size(18.0).bold(true);

    // Paragraph with a footnote reference
    let fn_id = doc.add_footnote("This is the first footnote. It appears at the bottom of the page.");
    let mut p = doc.add_paragraph("");
    p.add_run("This paragraph has a footnote reference").font("Arial").size(11.0);
    p.add_run("").footnote_ref(fn_id);

    // Another paragraph with a second footnote
    let fn_id2 = doc.add_footnote("Second footnote with more detailed information about the topic.");
    let mut p = doc.add_paragraph("");
    p.add_run("Here is another sentence with a different footnote").font("Arial").size(11.0);
    p.add_run("").footnote_ref(fn_id2);

    // Paragraph with an endnote
    let en_id = doc.add_endnote("This endnote appears at the end of the document.");
    let mut p = doc.add_paragraph("");
    p.add_run("This paragraph references an endnote").font("Arial").size(11.0);
    p.add_run("").endnote_ref(en_id);

    doc.save("/tmp/zavora-docx-tests/01_footnotes.docx").unwrap();
    println!("✓ Saved /tmp/zavora-docx-tests/01_footnotes.docx");
}
