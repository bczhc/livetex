use clap::Parser;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use livetex::{mutex_lock, tex_monitor, Args, ARGS, ARGS_SHARED};
use log::{error, info};
use std::convert::Infallible;
use std::ffi::OsStr;
use std::io::{stdin, BufRead, BufReader, Read};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::thread::spawn;
use std::{env, io};
use tokio::net::TcpListener;

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

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
            tex_monitor::start(d.path())?;
        }
    }

    stdin().read_to_string(&mut String::new())?;
    return Ok(());
    info!("Listening on {}...", args.addr);
    let listener = TcpListener::bind(SocketAddr::from_str(&args.addr)?).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
