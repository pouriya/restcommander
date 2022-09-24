// Will be replaced by `build.rs` based on files in `www` directory

pub fn handle_static(_uri: String) -> Option<(Vec<u8>, Option<String>)> {
    None
}
