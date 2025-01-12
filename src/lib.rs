#![feature(decl_macro)]
#![feature(try_blocks)]

use clap::Parser;
use log::debug;
use once_cell::sync::Lazy;
use std::ffi::OsString;
use std::{fs, mem};
use std::path::PathBuf;
use std::sync::Mutex;

pub mod tex_monitor;

pub static ARGS: Lazy<Mutex<Args>> = Lazy::new(|| Mutex::new(Default::default()));
pub static ARGS_SHARED: Lazy<Args> = Lazy::new(|| mutex_lock!(ARGS).clone());
pub static COMPILED_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let compiled_path = ARGS_SHARED.root.join("live-compiled");
    if !compiled_path.exists() {
        fs::create_dir(&compiled_path).unwrap();
        debug!("Created directory: {}", compiled_path.display());
    }
    compiled_path
});
pub static INTERMEDIATES_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let dir = temp_dir::TempDir::new().unwrap();
    let path = dir.path().to_path_buf();
    mem::forget(dir);
    path
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
    /// Command to build a TeX file. This argument should be present last.
    #[arg(short = 'c', long, num_args = 1.., allow_hyphen_values = true)]
    pub build_command: Vec<OsString>,
}

pub macro mutex_lock($m:expr) {
    $m.lock().unwrap()
}
