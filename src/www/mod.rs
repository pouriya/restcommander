// Auto-generated via `build.rs`

pub fn handle_static(_uri: String) -> Option<(Vec<u8>, Option<String>)> {
    match _uri.as_str() {
        "index.js" => Some((
            include_bytes!("index.js").to_vec(),
            Some("text/javascript".to_string()),
        )),
        "commands.html" => Some((
            include_bytes!("commands.html").to_vec(),
            Some("text/html".to_string()),
        )),
        "bootstrap.bundle.min.js" => Some((
            include_bytes!("bootstrap.bundle.min.js").to_vec(),
            Some("text/javascript".to_string()),
        )),
        "commands.js" => Some((
            include_bytes!("commands.js").to_vec(),
            Some("text/javascript".to_string()),
        )),
        "login.js" => Some((
            include_bytes!("login.js").to_vec(),
            Some("text/javascript".to_string()),
        )),
        "configuration.js" => Some((
            include_bytes!("configuration.js").to_vec(),
            Some("text/javascript".to_string()),
        )),
        "styles.css" => Some((
            include_bytes!("styles.css").to_vec(),
            Some("text/css".to_string()),
        )),
        "favicon.ico" => Some((include_bytes!("favicon.ico").to_vec(), None)),
        "login.html" => Some((
            include_bytes!("login.html").to_vec(),
            Some("text/html".to_string()),
        )),
        "index.html" => Some((
            include_bytes!("index.html").to_vec(),
            Some("text/html".to_string()),
        )),
        "bootstrap.min.css" => Some((
            include_bytes!("bootstrap.min.css").to_vec(),
            Some("text/css".to_string()),
        )),
        "api.js" => Some((
            include_bytes!("api.js").to_vec(),
            Some("text/javascript".to_string()),
        )),
        "utils.js" => Some((
            include_bytes!("utils.js").to_vec(),
            Some("text/javascript".to_string()),
        )),
        _ => None,
    }
}
