use clap::Parser;
use livetex::Args;
use log::{error, info};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::thread::spawn;
use std::{env, io};
use std::io::{BufRead, BufReader, Read};
use std::convert::Infallible;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

#[tokio::main]
async fn main() -> anyhow::Result<() >{
    // enable info logging mode by default
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();
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

