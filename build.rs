use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

const BOOTSTRAP_JS_FILENAME: &str = "bootstrap.bundle.min.js";
const BOOTSTRAP_CSS_FILENAME: &str = "bootstrap.min.css";
const BOOTSTRAP_VERSION_FILENAME: &str = "bootstrap-version.txt";

fn main() {
    let mod_rs_filename = PathBuf::from("src").join("www").join("mod.rs");
    let mut mod_rs_file = fs::File::create(mod_rs_filename.clone()).unwrap();
    // Start function body:
    mod_rs_file.write_all(
        r#"// Auto-generated via `build.rs`

pub fn handle_static(uri: String) -> Option<(Vec<u8>, Option<String>)> {"#.as_bytes()).unwrap();

    // Check if there are files in `www` directory (except `README.md` & `bootstrap-version.txt`):
    if !fs::read_dir("www")
        .unwrap()
        .fold(
            false,
            |has_file, filename| {
                let filename = PathBuf::from(filename.unwrap().path().file_name().unwrap());
                has_file || filename != PathBuf::from("README.md") || filename != PathBuf::from(BOOTSTRAP_VERSION_FILENAME)
            }
        ) {
        println!("cargo:warning=There is no file in `www` directory");
        // Close function body:
        mod_rs_file.write_all(" None }".as_bytes()).unwrap();
        exit(0);
    }

    // Make sure if `www` directory contains bootstrap files. Since they are used in HTML files:
    let (has_bootstrap_js, has_bootstrap_css) = fs::read_dir("www")
        .unwrap()
        .fold(
            (false, false,),
            |(has_bootstrap_js, has_bootstrap_css), filename| {
                let filename = PathBuf::from(filename.unwrap().path().file_name().unwrap());
                if filename == PathBuf::from(BOOTSTRAP_JS_FILENAME) {
                    (true, has_bootstrap_css)
                } else if filename == PathBuf::from(BOOTSTRAP_CSS_FILENAME) {
                    (has_bootstrap_js, true)
                } else {
                    (has_bootstrap_js, has_bootstrap_css)
                }
            }
        );
    if !has_bootstrap_js || !has_bootstrap_css {
        println!(
            "cargo:warning=Could not found {} in `www` directory, Will replace public bootstrap links inside `*.html` files",
            if !has_bootstrap_js && !has_bootstrap_js {
                format!("`{}` and `{}`", BOOTSTRAP_JS_FILENAME, BOOTSTRAP_CSS_FILENAME)
            } else if !has_bootstrap_js {
                format!("`{}`", BOOTSTRAP_JS_FILENAME)
            } else {
                format!("`{}`", BOOTSTRAP_CSS_FILENAME)
            }
        );
    }

    // Make `src/www/mod.rs` body from files in `www` directory (except `README.md` & `bootstrap-version.txt`):
    let match_body = fs::read_dir("www")
        .unwrap()
        .fold(
            String::new(),
            |source_code, filename| {
                let filename = PathBuf::from(filename.unwrap().path().file_name().unwrap());
                if filename == PathBuf::from("README.md") || filename == PathBuf::from(BOOTSTRAP_VERSION_FILENAME) {
                    return source_code;
                }
                let match_left_side = format!("{:?}", filename);
                let extension = filename.extension().unwrap();
                let maybe_mime_type = if extension  == OsStr::new("html") {
                    "Some(\"text/html\".to_string())"
                } else if extension  == OsStr::new("css") {
                    "Some(\"text/css\".to_string())"
                } else if extension  == OsStr::new("js") {
                    "Some(\"text/javascript\".to_string())"
                } else {
                    "None"
                }.to_string();
                let match_right_side = format!("Some((include_bytes!({:?}).to_vec(), {}))", filename, maybe_mime_type);
                let match_line = format!("        {} => {},", match_left_side, match_right_side);
                let (from, to) = (PathBuf::from("www").join(filename.clone()), PathBuf::from("src").join("www").join(filename.clone()));
                println!("cargo:warning={:?} -> {:?}", from, to);
                fs::copy(from, to.clone()).unwrap();
                if extension == OsStr::new("html") {
                    if !has_bootstrap_js || !has_bootstrap_css {
                        let mut data = fs::read_to_string(to.clone()).unwrap();
                        let bootstrap_version = fs::read_to_string(PathBuf::from("www").join(BOOTSTRAP_VERSION_FILENAME)).unwrap().trim().to_string();
                        if !has_bootstrap_js {
                            data = data.replace(
                                format!("\"{}\"", BOOTSTRAP_JS_FILENAME).as_str(),
                                format!("\"https://cdn.jsdelivr.net/npm/bootstrap@{}/dist/js/{}\"", bootstrap_version, BOOTSTRAP_JS_FILENAME).as_str()
                            );
                        }
                        if !has_bootstrap_css {
                            data = data.replace(
                                format!("\"{}\"", BOOTSTRAP_CSS_FILENAME).as_str(),
                                format!("\"https://cdn.jsdelivr.net/npm/bootstrap@{}/dist/css/{}\"", bootstrap_version, BOOTSTRAP_CSS_FILENAME).as_str()
                            );
                        }
                        fs::write(to.clone(), data).unwrap();
                        println!("cargo:warning=Updated bootstrap link(s) inside {:?}", to)
                    }
                }
                format!("{}\n{}", source_code, match_line)
            }
        );
    mod_rs_file.write_all("\n    match uri.as_str() {".as_bytes()).unwrap();
    mod_rs_file.write_all(match_body.as_bytes()).unwrap();
    mod_rs_file.write_all(
        r#"
        _ => None,
    }
}
"#.as_bytes()
    ).unwrap();
    println!("cargo:warning=Updated {:?}", mod_rs_filename)
}
