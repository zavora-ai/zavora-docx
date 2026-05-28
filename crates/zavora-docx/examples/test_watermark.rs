use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    // Set watermark
    doc.set_text_watermark("DRAFT", "C0C0C0", None);

    let mut p = doc.add_paragraph("");
    p.add_run("Watermark Demo").font("Arial").size(18.0).bold(true);

    let mut p = doc.add_paragraph("");
    p.add_run("This document has a diagonal 'DRAFT' watermark visible on every page.").font("Arial").size(11.0);

    let mut p = doc.add_paragraph("");
    p.add_run("The watermark appears behind the text as a semi-transparent gray shape.").font("Arial").size(11.0);

    doc.save("/Users/jameskaranja/Downloads/05_watermark.docx").unwrap();
    println!("✓ Saved ~/Downloads/05_watermark.docx");
}
