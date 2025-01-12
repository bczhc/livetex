#![feature(decl_macro)]
#![feature(try_blocks)]

use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub mod tex_monitor;

pub static ARGS: Lazy<Mutex<Args>> = Lazy::new(|| Mutex::new(Default::default()));
pub static ARGS_SHARED: Lazy<Args> = Lazy::new(|| {
    mutex_lock!(ARGS).clone()
});

#[derive(Parser, Debug, Default, Clone)]
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

pub macro mutex_lock($m:expr) {
    $m.lock().unwrap()
}
