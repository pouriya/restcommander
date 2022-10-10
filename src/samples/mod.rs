// Auto-generated via `build.rs`

use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Script and configuration samples")]
pub enum CMDSample {
    #[structopt(about = "Sample Systemd configuration")]
    Systemd,
    #[structopt(about = "Sample Shell script")]
    Shell,
    #[structopt(about = "Sample configuration")]
    Config,
    #[structopt(about = "Sample Python(v3) script")]
    Python,
    #[structopt(about = "A script to test service HTTP API status codes and body")]
    TestScript,
    #[structopt(about = "YAML information for test script")]
    TestScriptInfo,
    #[structopt(about = "A self-signed SSL private key (ONLY FOR TEST PURPOSES)")]
    SelfSignedKey,
    #[structopt(about = "Sample Perl script")]
    Perl,
    #[structopt(about = "A self-signed SSL certificate (ONLY FOR TEST PURPOSES)")]
    SelfSignedCert,
}

pub fn maybe_print(sample_name: CMDSample) {
    let sample_data = match sample_name {
        CMDSample::Systemd => include_str!("systemd.service").to_string(),
        CMDSample::Shell => include_str!("shell.sh").to_string(),
        CMDSample::Config => include_str!("config.toml").to_string(),
        CMDSample::Python => include_str!("python.py").to_string(),
        CMDSample::TestScript => include_str!("test-script").to_string(),
        CMDSample::TestScriptInfo => include_str!("test-script-info.yml").to_string(),
        CMDSample::SelfSignedKey => include_str!("self-signed-key.pem").to_string(),
        CMDSample::Perl => include_str!("perl.pl").to_string(),
        CMDSample::SelfSignedCert => include_str!("self-signed-cert.pem").to_string(),
    };
    println!("{}", sample_data);
}
