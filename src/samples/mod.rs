use crate::settings::CMDSample;

pub fn maybe_print(sample_name: CMDSample) -> Option<String> {
    match get_sample(sample_name) {
        Ok(sample_content) => {
            print!("{}", sample_content);
            None
        }
        Err(reason) => Some(reason),
    }
}

fn get_sample(sample_name: CMDSample) -> Result<String, String> {
    match sample_name {
        CMDSample::Config => config(),
        CMDSample::Python => python(),
        CMDSample::Shell => shell(),
        CMDSample::Perl => perl(),
        CMDSample::SystemdService => systemd_service(),
        CMDSample::SelfSignedKey => self_signed_key(),
        CMDSample::SelfSignedCert => self_signed_cert(),
        CMDSample::TestScript => test_script(),
        CMDSample::TestScriptInfo => test_script_info(),
    }
}

fn config() -> Result<String, String> {
    Ok(include_str!("restcommander.toml").to_string())
}

fn python() -> Result<String, String> {
    Ok(include_str!("python.py").to_string())
}

fn shell() -> Result<String, String> {
    Ok(include_str!("shell.sh").to_string())
}

fn perl() -> Result<String, String> {
    Ok(include_str!("perl.pl").to_string())
}

fn self_signed_key() -> Result<String, String> {
    Ok(include_str!("key.pem").to_string())
}

fn self_signed_cert() -> Result<String, String> {
    Ok(include_str!("cert.pem").to_string())
}

fn systemd_service() -> Result<String, String> {
    Ok(include_str!("restcommander.service").to_string())
}

fn test_script() -> Result<String, String> {
    Ok(include_str!("test").to_string())
}

fn test_script_info() -> Result<String, String> {
    Ok(include_str!("test.yml").to_string())
}
