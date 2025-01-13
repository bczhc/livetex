use crate::server::{TexState, UPDATE_STATES};
use crate::{mutex_lock, ARGS_SHARED, COMPILED_PATH, INTERMEDIATES_PATH};
use log::{debug, info, warn};
use std::env::temp_dir;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime};

pub fn log_file(tex_name: &str) -> PathBuf {
    let path = Path::new(tex_name).with_extension("log");
    let log_filename = path.file_name().expect("No filename");
    let log_path = INTERMEDIATES_PATH.join(log_filename);
    log_path
}

fn compile(source: &Path, out_path: Option<&Path>) -> anyhow::Result<ExitStatus> {
    let intermediates = &*INTERMEDIATES_PATH;

    info!("TeX {}: start compilation", source.display());
    let mut cmd = ARGS_SHARED.build_command.clone();
    cmd.push("--output-directory".into());
    cmd.push(intermediates.into());
    cmd.push(source.as_os_str().into());
    assert!(cmd.len() >= 2);
    let mut process = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .current_dir(&ARGS_SHARED.root)
        .spawn()?;
    let status = process.wait()?;
    if status.success() {
        info!("TeX {}: compilation done", source.display(),);
    } else {
        warn!(
            "TeX {}: compilation done; non-zero exit status",
            source.display()
        );
    }

    // TODO: issue will encounter in the case where `a.tex` and `a.TeX` are both present for example.
    if status.success() {
        let source_pdf_ext = source.with_extension("pdf");
        let pdf_name = source_pdf_ext.file_name().expect("No filename");
        let pdf_path = intermediates.join(pdf_name);
        // TODO: it's possible a successfully compiled source produces no PDF
        //  This info can be retrieved from log like:
        //  `Output written on /tmp/t37277f-0/a.pdf (4 pages).`
        if let Some(out_path) = out_path && pdf_path.exists() {
            fs::copy(&pdf_path, out_path.join(pdf_name))?;
            debug!(
                "Copy file: {} -> {}",
                pdf_path.display(),
                out_path.display()
            );
            debug!("Output file: {}", out_path.display());
        }
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
    let result = compile(tex_file, Some(&COMPILED_PATH))?;
    mutex_lock!(UPDATE_STATES).insert(
        tex_name.into(),
        TexState {
            update: false,
            error: !result.success(),
        },
    );

    // TODO: Use `notify` crate instead of this naive file update watcher.
    let mut last_mtime = tex_file.metadata()?.modified()?;
    loop {
        let mtime = tex_file.metadata()?.modified()?;
        if mtime != last_mtime {
            info!("File changed: {}; compile it", tex_file.display());
            last_mtime = mtime;
            let result = compile(tex_file, Some(&COMPILED_PATH))?;
            mutex_lock!(UPDATE_STATES).insert(
                tex_name.into(),
                TexState {
                    update: true,
                    error: !result.success(),
                },
            );
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
