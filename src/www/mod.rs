pub fn get_index_html() -> String {
    include_str!("index.html").to_string()
}

pub fn handle_static(uri: String) -> Option<Vec<u8>> {
    match uri.as_str() {
        "index.html" => Some(include_bytes!("index.html").to_vec()),
        "panel.js" => Some(include_bytes!("panel.js").to_vec()),
        "favicon.ico" => Some(include_bytes!("favicon.ico").to_vec()),
        "style.css" => Some(include_bytes!("style.css").to_vec()),
        "inconsolata.woff2" => Some(include_bytes!("inconsolata.woff2").to_vec()),
        _ => None,
    }
}
