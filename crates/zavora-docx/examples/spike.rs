use zavora_docx::Document;
fn main() {
    let path = std::env::args().nth(1).expect("path");
    let doc = Document::open(&path).expect("open");
    let mut i = 0;
    loop {
        match doc.render_page_to_png(i, 110.0) {
            Ok(Some(png)) => {
                std::fs::write(format!("/tmp/spike_p{i}.png"), &png).unwrap();
                println!("page {i}: {} bytes", png.len());
                i += 1;
            }
            Ok(None) => break,
            Err(e) => { println!("page {i} error: {e}"); break; }
        }
        if i > 20 { break; }
    }
    println!("total pages: {i}");
}
