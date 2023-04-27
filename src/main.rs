use std::net::SocketAddr;
use http_body_util::Full;
use http_body_util::Empty;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::{Request, Response };
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::body::Bytes;
use hyper::Method;
use hyper::StatusCode;
use tokio::net::TcpListener;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    let listener = TcpListener::bind(addr).await.unwrap();
    
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(auth_process))
                .await {
                    println!("Error serving connection: {:?}", err);
            }
        });
    }
}

const REQUIRED_PARAMS: &[&str; 2] = &["response_type", "client_id"];

async fn auth_process(
    req: Request<hyper::body::Incoming>
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match(req.method(), req.uri().path()) {
        (&Method::GET, "/authorize") => {
            if let Some(query) = req.uri().query() {
                let test:HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
                    .into_owned()
                    .collect();
                if REQUIRED_PARAMS.iter().all(|param| test.contains_key(*param)) {
                    return Ok(Response::new(full(
                        "Has All Params"
                    )));
                }
            } 
            
            let mut bad_request = Response::new(empty());
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            Ok(bad_request)     
            
        },
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

enum AuthorizeParams {
    ResponseType(String),
    ClientId(String),
    RedirectUri(String), //TODO: Maybe Make URI Type...
    Scope(String),
    State(String) //security measure to prevent CSRF/Replay
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}