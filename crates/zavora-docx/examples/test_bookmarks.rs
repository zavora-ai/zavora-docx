use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    let mut p = doc.add_paragraph("");
    p.add_run("Bookmarks Demo").font("Arial").size(18.0).bold(true);

    // Internal link to a bookmark below
    let mut p = doc.add_paragraph("");
    p.add_run("Click to jump to: ").font("Arial").size(11.0);
    p.add_hyperlink_run("Section Two", None, Some("section_two"))
        .color("0563C1").underline(true);

    // Some filler content
    let mut p = doc.add_paragraph("");
    p.add_run("This is section one with some content...").font("Arial").size(11.0);

    // Bookmarked paragraph
    let mut p = doc.add_paragraph("");
    p.add_run("Section Two - Bookmarked").font("Arial").size(14.0).bold(true);
    p.bookmark(1, "section_two");

    let mut p = doc.add_paragraph("");
    p.add_run("This paragraph is the target of the internal link above.").font("Arial").size(11.0);

    doc.save("/Users/jameskaranja/Downloads/03_bookmarks.docx").unwrap();
    println!("✓ Saved ~/Downloads/03_bookmarks.docx");
}
