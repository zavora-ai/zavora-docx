use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    let mut p = doc.add_paragraph("");
    p.add_run("Document Protection Demo").font("Arial").size(18.0).bold(true);

    let mut p = doc.add_paragraph("");
    p.add_run("This document is protected as read-only. You should see a restriction notice when trying to edit in Word.").font("Arial").size(11.0);

    let mut p = doc.add_paragraph("");
    p.add_run("To unprotect: Review > Restrict Editing > Stop Protection").font("Arial").size(11.0);

    // Protect as read-only
    doc.protect_readonly();

    doc.save("/Users/jameskaranja/Downloads/08_protection.docx").unwrap();
    println!("✓ Saved ~/Downloads/08_protection.docx");
}
