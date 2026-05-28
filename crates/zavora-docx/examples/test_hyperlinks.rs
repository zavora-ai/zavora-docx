use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    let mut p = doc.add_paragraph("");
    p.add_run("Hyperlinks Demo").font("Arial").size(18.0).bold(true);

    // External hyperlink
    let rel_id = doc.add_hyperlink_rel("https://www.rust-lang.org");
    let mut p = doc.add_paragraph("");
    p.add_run("Visit the Rust website: ").font("Arial").size(11.0);
    p.add_hyperlink_run("rust-lang.org", Some(&rel_id), None)
        .color("0563C1").underline(true);

    // Another external link
    let rel_id2 = doc.add_hyperlink_rel("https://github.com/zavora-ai");
    let mut p = doc.add_paragraph("");
    p.add_run("Check out ").font("Arial").size(11.0);
    p.add_hyperlink_run("Zavora AI on GitHub", Some(&rel_id2), None)
        .color("0563C1").underline(true);
    p.add_run(" for more projects.").font("Arial").size(11.0);

    // Internal link (bookmark reference - will work with Feature 3)
    let mut p = doc.add_paragraph("");
    p.add_run("Jump to: ").font("Arial").size(11.0);
    p.add_hyperlink_run("Section Two", None, Some("section_two"))
        .color("0563C1").underline(true);

    doc.save("/tmp/zavora-docx-tests/02_hyperlinks.docx").unwrap();
    println!("✓ Saved /tmp/zavora-docx-tests/02_hyperlinks.docx");
}
