use crate::ARGS_SHARED;
use log::{debug, info};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime};

fn compile(source: &Path) -> anyhow::Result<ExitStatus> {
    debug!("TeX {}: start compilation", source.display());
    let mut cmd = ARGS_SHARED.build_command.clone();
    cmd.push(source.as_os_str().into());
    assert!(cmd.len() >= 2);
    let mut process = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?;
    let status = process.wait()?;
    debug!("TeX {}: done; status: {}", source.display(), status);
    Ok(status)
}

fn worker(tex_file: &Path) -> anyhow::Result<()> {
    // compile the file first, then do the monitoring
    compile(tex_file)?;

    // TODO: Use `notify` crate instead of this naive file update watcher.
    let mut last_mtime = tex_file.metadata()?.modified()?;
    loop {
        let mtime = tex_file.metadata()?.modified()?;
        if mtime != last_mtime {
            info!("File changed: {}; compile it", tex_file.display());
            last_mtime = mtime;
            compile(tex_file)?;
        }
        sleep(Duration::from_secs_f32(0.5));
    }
}

/// Start the TeX auto builder worker thread.
pub fn start(tex_file: PathBuf) -> anyhow::Result<()> {
    spawn(move || worker(&tex_file));

    Ok(())
}
