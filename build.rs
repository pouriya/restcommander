use capitalize::Capitalize;
use md5::compute;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const BOOTSTRAP_JS_FILENAME: &str = "bootstrap.bundle.min.js";
const BOOTSTRAP_CSS_FILENAME: &str = "bootstrap.min.css";
const BOOTSTRAP_VERSION_FILENAME: &str = "bootstrap-version.txt";
const SAMPLE_DESCRIPTIONS_FILENAME: &str = "sample-description.cfg";

macro_rules! log {
    ($text:expr) => {
        println!("cargo:warning={}", $text);
    };
    ($text:expr, $($parameters:expr),+) => {
        println!("cargo:warning={}", format!($text, $($parameters),+))
    }
}

fn check_md5(
    source_directory: PathBuf,
    destination_directory: PathBuf,
    excluded_file_list: Vec<PathBuf>,
) -> bool {
    fs::read_dir(source_directory.clone())
        .unwrap()
        .try_for_each(|source_file| {
            let source_file = source_file.unwrap().path();
            let source_file_name = PathBuf::from(source_file.file_name().unwrap());
            if excluded_file_list.contains(&source_file_name) {
                return Ok(());
            }
            let destination_file = destination_directory.join(source_file_name);
            if !destination_file.exists() {
                log!("New file {:?} detected", source_file);
                return Err(());
            }
            let (source_file_data, destination_file_data) = (
                fs::read(source_file.clone()).unwrap(),
                fs::read(destination_file.clone()).unwrap(),
            );
            if compute(source_file_data) == compute(destination_file_data) {
                return Ok(());
            }
            log!("Content of file {:?} is changed", source_file);
            Err(())
        })
        .map(|_| false)
        .unwrap_or(true)
}

fn maybe_build_src_www() {
    let excluded_file_list = [BOOTSTRAP_VERSION_FILENAME, "README.md"]
        .map(|x| PathBuf::from(x))
        .to_vec();
    if !check_md5(
        PathBuf::from("www"),
        PathBuf::from("src").join("www"),
        excluded_file_list.clone(),
    ) {
        // No file is changed
        return;
    }
    let mod_rs_filename = PathBuf::from("src").join("www").join("mod.rs");
    let mut mod_rs_file = fs::File::create(mod_rs_filename.clone()).unwrap();
    log!("Attempt to regenerate {:?}", mod_rs_filename);
    // Start function body:
    mod_rs_file
        .write_all(
            r#"// Auto-generated via `build.rs`

pub fn handle_static(_uri: String) -> Option<(Vec<u8>, Option<String>)> {"#
                .as_bytes(),
        )
        .unwrap();

    // Check if there are files in `www` directory:
    if !fs::read_dir("www")
        .unwrap()
        .fold(false, |has_file, filename| {
            let filename = PathBuf::from(filename.unwrap().path().file_name().unwrap());
            has_file || !excluded_file_list.contains(&filename)
        })
    {
        log!("There is no file in `www` directory");
        // Close function body:
        mod_rs_file.write_all(" None }".as_bytes()).unwrap();
        log!("Generated {:?} successfully", mod_rs_file);
        return;
    }

    // Make sure if `www` directory contains bootstrap files. Since they are used in HTML files:
    let (has_bootstrap_js, has_bootstrap_css) = fs::read_dir("www").unwrap().fold(
        (false, false),
        |(has_bootstrap_js, has_bootstrap_css), filename| {
            let filename = PathBuf::from(filename.unwrap().path().file_name().unwrap());
            if filename == PathBuf::from(BOOTSTRAP_JS_FILENAME) {
                (true, has_bootstrap_css)
            } else if filename == PathBuf::from(BOOTSTRAP_CSS_FILENAME) {
                (has_bootstrap_js, true)
            } else {
                (has_bootstrap_js, has_bootstrap_css)
            }
        },
    );
    if !has_bootstrap_js || !has_bootstrap_css {
        log!(
            "Could not found {} in `www` directory, Will replace public bootstrap links inside `*.html` files",
            if !has_bootstrap_js && !has_bootstrap_js {
                format!("`{}` and `{}`", BOOTSTRAP_JS_FILENAME, BOOTSTRAP_CSS_FILENAME)
            } else if !has_bootstrap_js {
                format!("`{}`", BOOTSTRAP_JS_FILENAME)
            } else {
                format!("`{}`", BOOTSTRAP_CSS_FILENAME)
            }
        );
    }
    // Make `src/www/mod.rs` body from files in `www` directory:
    mod_rs_file
        .write_all("\n    match _uri.as_str() {".as_bytes())
        .unwrap();
    let match_body = fs::read_dir("www")
        .unwrap()
        .fold(String::new(), |source_code, filename| {
            let filename = PathBuf::from(filename.unwrap().path().file_name().unwrap());
            if excluded_file_list.contains(&filename) {
                return source_code;
            }
            let match_left_side = format!("{:?}", filename);
            let extension = filename.extension().unwrap();
            let maybe_mime_type = if extension == OsStr::new("html") {
                "Some(\"text/html\".to_string())"
            } else if extension == OsStr::new("css") {
                "Some(\"text/css\".to_string())"
            } else if extension == OsStr::new("js") {
                "Some(\"text/javascript\".to_string())"
            } else {
                "None"
            }
            .to_string();
            let match_right_side = format!(
                "Some((include_bytes!({:?}).to_vec(), {}))",
                filename, maybe_mime_type
            );
            let match_line = format!("        {} => {},", match_left_side, match_right_side);
            let (from, to) = (
                PathBuf::from("www").join(filename.clone()),
                PathBuf::from("src").join("www").join(filename.clone()),
            );
            fs::copy(from.clone(), to.clone()).unwrap();
            log!("{:?} -> {:?}", from, to);
            if extension == OsStr::new("html") && (!has_bootstrap_js || !has_bootstrap_css) {
                let mut data = fs::read_to_string(to.clone()).unwrap();
                let bootstrap_version =
                    fs::read_to_string(PathBuf::from("www").join(BOOTSTRAP_VERSION_FILENAME))
                        .unwrap()
                        .trim()
                        .to_string();
                if !has_bootstrap_js {
                    data = data.replace(
                        format!("\"{}\"", BOOTSTRAP_JS_FILENAME).as_str(),
                        format!(
                            "\"https://cdn.jsdelivr.net/npm/bootstrap@{}/dist/js/{}\"",
                            bootstrap_version, BOOTSTRAP_JS_FILENAME
                        )
                        .as_str(),
                    );
                }
                if !has_bootstrap_css {
                    data = data.replace(
                        format!("\"{}\"", BOOTSTRAP_CSS_FILENAME).as_str(),
                        format!(
                            "\"https://cdn.jsdelivr.net/npm/bootstrap@{}/dist/css/{}\"",
                            bootstrap_version, BOOTSTRAP_CSS_FILENAME
                        )
                        .as_str(),
                    );
                }
                fs::write(to.clone(), data).unwrap();
                log!("Updated bootstrap link(s) inside {:?}", to);
            }
            format!("{}\n{}", source_code, match_line)
        });
    mod_rs_file.write_all(match_body.as_bytes()).unwrap();
    mod_rs_file
        .write_all(
            r#"
        _ => None,
    }
}
"#
            .as_bytes(),
        )
        .unwrap();
    mod_rs_file.flush().unwrap();
    log!("Regenerated {:?}", mod_rs_filename);
}

fn maybe_build_src_samples() {
    let excluded_file_list = ["README.md", SAMPLE_DESCRIPTIONS_FILENAME]
        .map(|x| PathBuf::from(x))
        .to_vec();
    if !check_md5(
        PathBuf::from("samples"),
        PathBuf::from("src").join("samples"),
        excluded_file_list.clone(),
    ) {
        // No file is changed
        return;
    }
    let mod_rs_filename = PathBuf::from("src").join("samples").join("mod.rs");
    let mut mod_rs_file = fs::File::create(mod_rs_filename.clone()).unwrap();
    log!("Attempt to regenerate {:?}", mod_rs_filename);
    mod_rs_file
        .write_all(
            r#"// Auto-generated via `build.rs`

use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Script and configuration samples")]
pub enum CMDSample {"#
                .as_bytes(),
        )
        .unwrap();
    if !fs::read_dir("samples")
        .unwrap()
        .fold(false, |has_file, filename| {
            let filename = PathBuf::from(filename.unwrap().path().file_name().unwrap());
            has_file || !excluded_file_list.contains(&filename)
        })
    {
        log!("There is no file in `samples` directory");
        // Close function body:
        mod_rs_file
            .write_all(
                r#"}

pub fn maybe_print(sample_name: CMDSample) {
    let sample_data = "There is no sample to print".to_string();
    println!("{}", sample_data);
}
"#
                .as_bytes(),
            )
            .unwrap();
        log!("Generated {:?} successfully", mod_rs_file);
        return;
    }
    let mut descriptions = HashMap::new();
    if PathBuf::from("samples")
        .join(SAMPLE_DESCRIPTIONS_FILENAME)
        .exists()
    {
        // Load file descriptions from `SAMPLE_DESCRIPTIONS_FILENAME`
        // Its data is in form of <FILENAME> = <DESCRIPTION>
        for line in fs::read_to_string(PathBuf::from("samples").join(SAMPLE_DESCRIPTIONS_FILENAME))
            .unwrap()
            .lines()
        {
            let line_part_list = line
                .splitn(2, '=')
                .map(|x| x.to_string())
                .collect::<Vec<_>>();
            let (filename, description) = (
                PathBuf::from(line_part_list[0].trim()),
                line_part_list[1].trim().to_string(),
            );
            descriptions.insert(filename, description);
        }
    }
    let (enum_body, function_body) = fs::read_dir("samples").unwrap().fold(
        (String::new(), String::new()),
        |(enum_body, function_body), file| {
            let file = file.unwrap().path();
            let file_name = PathBuf::from(file.file_name().unwrap());
            if excluded_file_list.contains(&file_name) {
                return (enum_body, function_body);
            }
            let file_stem = file_name.file_stem().unwrap().to_str().unwrap().to_string();
            let sample_name = file_stem
                .split('-')
                .fold(String::new(), |sample_name, word| {
                    format!("{}{}", sample_name, word.capitalize())
                });
            let variant = if descriptions.contains_key(&file_name) {
                format!(
                    "\n    #[structopt(about = \"{}\")]\n    {},",
                    descriptions.get(&file_name).unwrap(),
                    sample_name
                )
            } else {
                format!("\n    {},", sample_name)
            };
            let (from, to) = (
                PathBuf::from("samples").join(file_name.clone()),
                PathBuf::from("src").join("samples").join(file_name.clone()),
            );
            fs::copy(from.clone(), to.clone()).unwrap();
            log!("{:?} -> {:?}", from, to);
            (
                format!("{}{}", enum_body, variant),
                format!(
                    "{}{}",
                    function_body,
                    format!(
                        "\n        CMDSample::{} => include_str!({:?}).to_string(),",
                        sample_name, file_name
                    )
                ),
            )
        },
    );
    mod_rs_file.write_all(enum_body.as_bytes()).unwrap();
    mod_rs_file.write_all("\n}\n".as_bytes()).unwrap();
    mod_rs_file
        .write_all(
            r#"
pub fn maybe_print(sample_name: CMDSample) {
    let sample_data = match sample_name {"#
                .as_bytes(),
        )
        .unwrap();
    mod_rs_file.write_all(function_body.as_bytes()).unwrap();
    mod_rs_file
        .write_all("\n    };\n    println!(\"{}\", sample_data);\n}\n".as_bytes())
        .unwrap();
    mod_rs_file.flush().unwrap();
    log!("Regenerated {:?}", mod_rs_filename);
}

fn main() {
    maybe_build_src_www();
    maybe_build_src_samples();
}
