use crate::server::UPDATE_STATES;
use crate::{mutex_lock, ARGS_SHARED, COMPILED_PATH, INTERMEDIATES_PATH};
use log::{debug, info};
use std::env::temp_dir;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime};

fn compile(source: &Path, out_path: Option<&Path>) -> anyhow::Result<ExitStatus> {
    let intermediates = &*INTERMEDIATES_PATH;

    debug!("TeX {}: start compilation", source.display());
    let mut cmd = ARGS_SHARED.build_command.clone();
    cmd.push(source.as_os_str().into());
    assert!(cmd.len() >= 2);
    let mut process = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .current_dir(intermediates)
        .spawn()?;
    let status = process.wait()?;
    debug!(
        "TeX {}: done; status: {:?}",
        source.display(),
        status.code()
    );

    // TODO: issue will encounter in the case where `a.tex` and `a.TeX` are both present for example.
    let source_pdf_ext = source.with_extension("pdf");
    let pdf_name = source_pdf_ext.file_name().expect("No filename");
    let pdf_path = intermediates.join(pdf_name);
    if let Some(out_path) = out_path {
        fs::copy(&pdf_path, out_path.join(pdf_name))?;
        debug!(
            "Copy file: {} -> {}",
            pdf_path.display(),
            out_path.display()
        );
        info!("Output file: {}", out_path.display());
    }

    Ok(status)
}

pub fn pdf_name(tex_name: impl AsRef<Path>) -> PathBuf {
    tex_name.as_ref().with_extension("pdf")
}

fn worker(tex_file: &Path) -> anyhow::Result<()> {
    let tex_name = tex_file
        .file_name()
        .expect("No filename")
        .to_str()
        .expect("Invalid UTF-8");
    // compile the file first, then do the monitoring
    compile(tex_file, Some(&COMPILED_PATH))?;

    // TODO: Use `notify` crate instead of this naive file update watcher.
    let mut last_mtime = tex_file.metadata()?.modified()?;
    loop {
        let mtime = tex_file.metadata()?.modified()?;
        if mtime != last_mtime {
            info!("File changed: {}; compile it", tex_file.display());
            last_mtime = mtime;
            let result = compile(tex_file, Some(&COMPILED_PATH))?;
            // TODO: display compilation error on the webpage
            if result.success() {
                mutex_lock!(UPDATE_STATES).insert(tex_name.into() , true);
            }
        }
        sleep(Duration::from_secs_f32(0.5));
    }
}

/// Start the TeX auto builder worker thread.
pub fn start(tex_file: PathBuf) -> anyhow::Result<()> {
    spawn(move || {
        worker(&tex_file).unwrap();
    });

    Ok(())
}
