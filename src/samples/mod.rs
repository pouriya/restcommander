// Will be replaced by `build.rs` based on files in `samples` directory

use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Script and configuration samples")]
pub enum CMDSample {}

pub fn maybe_print(sample_name: CMDSample) {
    let sample_data = "There is no sample to print".to_string();
    println!("{}", sample_data);
}
