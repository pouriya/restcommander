// Auto-generated via `build.rs`

use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Script and configuration samples")]
pub enum CMDSample {
    #[structopt(about = "Sample Perl script")]
    Perl,
    #[structopt(skip)]
    Banner,
    #[structopt(about = "Sample Shell script")]
    Shell,
    #[structopt(about = "Sample configuration")]
    Config,
    #[structopt(about = "A self-signed SSL private key (ONLY FOR TEST PURPOSES)")]
    SelfSignedKey,
    #[structopt(about = "Sample Systemd configuration")]
    Systemd,
    #[structopt(about = "A self-signed SSL certificate (ONLY FOR TEST PURPOSES)")]
    SelfSignedCert,
    #[structopt(about = "A sample script that accepts some input options")]
    Script,
    #[structopt(about = "Sample YAML information file")]
    ScriptInfo,
    #[structopt(about = "Sample Python(v3) script")]
    Python,
}

pub fn maybe_print(sample_name: CMDSample) {
    let sample_data = match sample_name {
        CMDSample::Perl => include_str!("perl.pl").to_string(),
        CMDSample::Banner => include_str!("banner.txt").to_string(),
        CMDSample::Shell => include_str!("shell.sh").to_string(),
        CMDSample::Config => include_str!("config.toml").to_string(),
        CMDSample::SelfSignedKey => include_str!("self-signed-key.pem").to_string(),
        CMDSample::Systemd => include_str!("systemd.service").to_string(),
        CMDSample::SelfSignedCert => include_str!("self-signed-cert.pem").to_string(),
        CMDSample::Script => include_str!("script").to_string(),
        CMDSample::ScriptInfo => include_str!("script-info.yml").to_string(),
        CMDSample::Python => include_str!("python.py").to_string(),
    };
    println!("{}", sample_data);
}
