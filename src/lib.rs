use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Parser, Debug)]
/// A live integrated server that compiles TeX and serve its PDF automatically on source changes.
pub struct Args {
    /// Server address
    #[arg(short, long, default_value = "0.0.0.0:8080", required = false)]
    pub addr: String,
    /// Root directory to serve
    #[arg(short, long)]
    pub root: PathBuf,
    /// Command to build a TeX file
    #[arg(short = 'c', long, num_args = 1.., allow_hyphen_values = true)]
    pub build_command: Vec<OsString>,
}
