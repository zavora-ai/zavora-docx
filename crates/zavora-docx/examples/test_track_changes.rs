use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    let mut p = doc.add_paragraph("");
    p.add_run("Track Changes Demo").font("Arial").size(18.0).bold(true);

    // Paragraph with tracked insertion
    let mut p = doc.add_paragraph("");
    p.add_run("This is the original text. ").font("Arial").size(11.0);
    p.add_tracked_insert("This text was added by the reviewer.", "James K");

    // Paragraph with tracked deletion
    let mut p = doc.add_paragraph("");
    p.add_run("Keep this part ").font("Arial").size(11.0);
    p.add_tracked_delete("remove this part", "Reviewer");
    p.add_run(" and keep this too.").font("Arial").size(11.0);

    // Mixed changes
    let mut p = doc.add_paragraph("");
    p.add_tracked_delete("Old heading text", "Editor");
    p.add_tracked_insert("New improved heading text", "Editor");

    doc.save("/Users/jameskaranja/Downloads/06_track_changes.docx").unwrap();
    println!("✓ Saved ~/Downloads/06_track_changes.docx");
}
