use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    let mut p = doc.add_paragraph("");
    p.add_run("Comments Demo").font("Arial").size(18.0).bold(true);

    // Add a comment
    doc.add_comment(1, "James K", "This needs more detail about the implementation.");

    // Paragraph with comment markers
    let mut p = doc.add_paragraph("");
    p.comment_start(1);
    p.add_run("This text has a comment attached to it.").font("Arial").size(11.0);
    p.comment_end(1);

    // Second comment
    doc.add_comment(2, "Reviewer", "Great point! Consider adding examples.");
    let mut p = doc.add_paragraph("");
    p.add_run("Regular text before. ").font("Arial").size(11.0);
    p.comment_start(2);
    p.add_run("This specific phrase is commented.").font("Arial").size(11.0);
    p.comment_end(2);
    p.add_run(" And text after.").font("Arial").size(11.0);

    doc.save("/Users/jameskaranja/Downloads/04_comments.docx").unwrap();
    println!("✓ Saved ~/Downloads/04_comments.docx");
}
