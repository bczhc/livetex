use crate::tex_monitor::{log_file, pdf_name};
use crate::{mutex_lock, ARGS_SHARED, COMPILED_PATH};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{header, Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use lazy_regex::regex;
use log::{debug, error, info};
use mime::Mime;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;
use tokio::net::TcpListener;

/// Map key is the TeX source filename (with extension).
pub static UPDATE_STATES: Lazy<Mutex<HashMap<String, TexState>>> =
    Lazy::new(|| Mutex::new(Default::default()));

#[derive(Debug, Serialize)]
pub struct TexState {
    /// If the PDF changes.
    pub update: bool,
    /// Error occurs during compilation.
    pub error: bool,
}

// TODO: refactor hyper-related code
//  The esoteric type system!

fn response_json<S: Serialize>(obj: &S) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(obj).unwrap();
    Response::builder()
        .header(CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
        .body(Full::new(json.into()))
        .unwrap()
}

fn response_empty() -> Response<Full<Bytes>> {
    Response::new(Full::new(Bytes::new()))
}

fn response_content(content: String, mime: Mime) -> Response<Full<Bytes>> {
    Response::builder()
        .header(CONTENT_TYPE, mime.to_string())
        .body(Full::new(Bytes::from(content)))
        .unwrap()
}
static INDEX_HTML: &str = include_str!("../res/index.html");

macro handle_result($r:expr) {
    match $r {
        Ok(r) => r,
        Err(e) => {
            error!("Server handler error: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::new()))
                .unwrap()
        }
    }
}

fn response_file(file: &Path, mime: Mime) -> Response<Full<Bytes>> {
    // TODO: use async stream
    let r: anyhow::Result<_> = try {
        let mut buf = Vec::new();
        File::open(file)?.read_to_end(&mut buf)?;
        Response::builder()
            .header(CONTENT_TYPE, mime.to_string())
            .body(Full::new(buf.into()))
            .unwrap()
    };
    handle_result!(r)
}

fn response_pdf(file: &Path) -> Response<Full<Bytes>> {
    response_file(file, mime::APPLICATION_PDF)
}

fn escape_js_string(text: &str) -> String {
    let mut string = String::new();
    use fmt::Write;
    for x in text.encode_utf16() {
        write!(&mut string, r#"\u{:04x}"#, x).unwrap();
    }
    format!("\"{string}\"")
}

fn serve_index(tex_name: &str) -> Response<Full<Bytes>> {
    let guard = mutex_lock!(UPDATE_STATES);
    debug!("{:?}", &*guard);
    let content = INDEX_HTML
        .replace(
            "const TEX_NAME = ''",
            format!("const TEX_NAME = {}", escape_js_string(tex_name)).as_str(),
        );
    Response::builder()
        .header(CONTENT_TYPE, mime::TEXT_HTML.to_string())
        .body(Full::new(content.into()))
        .unwrap()
}

/// Handles a request.
///
/// ## Routes
///
/// - GET /state/<tex-name>
///
///    Get the state of the tex file build.
///
/// - DELETE /update/<tex-name>
///
///    Like above, but reset the update status. This means the browser has done the refresh.
///
/// - GET /pdf/<tex-name>
///
///    Fetch the pdf corresponding to the tex file.
///
/// - GET /<tex-name>
///
///    Returns the main live page of the tex file.
///
/// - GET /log/<tex-name>
///
///    Returns build log of the tex file.
async fn handle_request(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    macro first_capture($r:expr, $h:expr) {
        $r.captures_iter($h)
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
    }

    let path = req.uri().path();
    let method = req.method();
    debug!("Request: {} {}", method, path);

    let regex1 = regex!("^/update/(.*)$");
    let regex2 = regex!("^/pdf/(.*)$");
    let regex4 = regex!("^/log/(.*)$");
    let regex3 = regex!(r#"^/(.*?\.tex)$"#);
    let regex5 = regex!("^/state/(.*)$");
    match () {
        _ if regex5.is_match(path) => {
            let tex_name = first_capture!(regex5, path);
            let guard = mutex_lock!(UPDATE_STATES);
            let tex_state = guard.get(tex_name);
            Ok(response_json(&tex_state))
        }
        _ if regex1.is_match(path) && method == Method::DELETE => {
            // DELETE /update/<tex-name>
            let tex_name = first_capture!(regex1, path);
            mutex_lock!(UPDATE_STATES)
                .get_mut(tex_name)
                .into_iter()
                .for_each(|x| x.update = false);
            Ok(response_empty())
        }
        _ if regex2.is_match(path) => {
            // GET /pdf/<tex-name>
            let tex_name = first_capture!(regex2, path);
            let pdf_path = COMPILED_PATH.join(pdf_name(tex_name));
            Ok(response_pdf(&pdf_path))
        }
        _ if regex4.is_match(path) => {
            let tex_name = first_capture!(regex4, path);
            let log_path = log_file(tex_name);
            Ok(response_file(&log_path, mime::TEXT_PLAIN))
        }
        _ if regex3.is_match(path) => {
            // GET /<tex-name>
            let tex_name = first_capture!(regex3, path);
            Ok(serve_index(tex_name))
        }
        _ => {
            error!("No route for path: {}", path);
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::new()))
                .unwrap())
        }
    }
}

pub async fn start_server(addr: &str) -> anyhow::Result<()> {
    info!("Listening on {}...", addr);
    let listener = TcpListener::bind(SocketAddr::from_str(addr)?).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}
