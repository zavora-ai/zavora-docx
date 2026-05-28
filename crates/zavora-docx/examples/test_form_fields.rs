use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    let mut p = doc.add_paragraph("");
    p.add_run("Form Fields Demo").font("Arial").size(18.0).bold(true);

    // Text field
    let mut p = doc.add_paragraph("");
    p.add_run("Name: ").font("Arial").size(11.0);
    p.add_text_field("FullName", "Enter your name");

    // Another text field
    let mut p = doc.add_paragraph("");
    p.add_run("Email: ").font("Arial").size(11.0);
    p.add_text_field("Email", "user@example.com");

    // Checkboxes
    let mut p = doc.add_paragraph("");
    p.add_run("Agree to terms: ").font("Arial").size(11.0);
    p.add_checkbox("AgreeTerms", false);

    let mut p = doc.add_paragraph("");
    p.add_run("Subscribe to newsletter: ").font("Arial").size(11.0);
    p.add_checkbox("Newsletter", true);

    // Dropdown
    let mut p = doc.add_paragraph("");
    p.add_run("Department: ").font("Arial").size(11.0);
    p.add_dropdown("Department", &["Engineering", "Marketing", "Sales", "HR"], 0);

    doc.save("/Users/jameskaranja/Downloads/07_form_fields.docx").unwrap();
    println!("✓ Saved ~/Downloads/07_form_fields.docx");
}
