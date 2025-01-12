use clap::Parser;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use livetex::{mutex_lock, server, tex_monitor, Args, ARGS, ARGS_SHARED};
use log::{debug, error, info};
use std::convert::Infallible;
use std::ffi::OsStr;
use std::io::{stdin, BufRead, BufReader, Read};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::thread::spawn;
use std::{env, fs, io};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var("RUST_LOG").is_err() {
        // enable info logging mode by default
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let args = Args::parse();
    *mutex_lock!(ARGS) = args.clone();

    // monitor all `.tex` files under the root folder. This behavior might be changed in the future.
    for d in ARGS_SHARED.root.read_dir()? {
        if let Err(d) = &d {
            error!("File read failed: {}", d);
        }
        let d = d.unwrap();
        if d.path().extension().map(|x| x.eq_ignore_ascii_case("tex")) == Some(true) {
            info!("Monitoring TeX file: {}", d.path().display());
            if d.path().to_str().is_some() {
                // we only accept filenames in valid UTF-8 encoding
                // and canonicalizable
                if let Ok(path) = d.path().canonicalize() {
                    tex_monitor::start(path)?;
                }
            }
        }
    }

    server::start_server(&args.addr).await?;
    Ok(())
}
