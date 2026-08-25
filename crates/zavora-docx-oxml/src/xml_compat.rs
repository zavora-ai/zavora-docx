use quick_xml::events::{BytesRef, BytesText};

pub(crate) fn decode_text(text: &BytesText<'_>) -> String {
    text.decode().unwrap_or_default().into_owned()
}

pub(crate) fn decode_reference(reference: &BytesRef<'_>) -> String {
    if let Ok(Some(character)) = reference.resolve_char_ref() {
        return character.to_string();
    }

    let name = reference.decode().unwrap_or_default();
    match name.as_ref() {
        "lt" => "<".to_string(),
        "gt" => ">".to_string(),
        "amp" => "&".to_string(),
        "apos" => "'".to_string(),
        "quot" => "\"".to_string(),
        other => format!("&{other};"),
    }
}
