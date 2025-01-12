use clap::Parser;
use livetex::Args;
use log::{error, info};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::thread::spawn;
use std::{env, io};
use std::io::{BufRead, BufReader, Read};

fn main() -> anyhow::Result<()> {
    // enable info logging mode by default
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();
    info!("Listening on {}...", args.addr);
    let listener = TcpListener::bind(SocketAddr::from_str(&args.addr)?)?;
    loop {
        match listener.accept() {
            Ok(s) => {
                spawn(move || {
                    info!("Accepted: {}", s.1);
                    let result = handle_client(s.0);
                    if let Err(e) = result {
                        error!("Error occurred on handling client: {}", e);
                        panic!("{}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failure on accept: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) -> anyhow::Result<()> {
    use io::Write;
    let mut request = String::new();
    BufReader::new(stream.try_clone()?).read_line(&mut request)?;
    println!("{}", request);
    
    writeln!(&mut stream, "{}", "HTTP/1.1 200 OK\r\n\r\nhello, world")?;
    drop(stream);

    Ok(())
}
