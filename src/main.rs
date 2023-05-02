use std::future::Future;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::pin::Pin;
use http_body_util::Full;
use http_body_util::Empty;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::service::Service;
use hyper::{Request, Response };
use hyper::server::conn::http1;
use hyper::body::{Bytes, Incoming};
use hyper::Method;
use hyper::StatusCode;
use stores::identity_store::IdentityStore;
use tokio::net::TcpListener;
mod stores;



#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, AuthService {
                    ident_store: IdentityStore::new()
                })
                .await {
                    println!("Error serving connection: {:?}", err);
            }
        });
    }
}

const REQUIRED_PARAMS: &[&str; 3] = &["response_type", "client_id", "scope"];

struct AuthService {
    ident_store: IdentityStore
}

impl Service<Request<Incoming>> for AuthService {
    type Response= Response<BoxBody<Bytes, hyper::Error>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        match(req.method(), req.uri().path()) {
            (&Method::GET, "/authorize") => {
                if let Some(query) = req.uri().query() {
                    let test:HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
                        .into_owned()
                        .collect();
                    if REQUIRED_PARAMS.iter().all(|param| test.contains_key(*param)) {
                        return Box::pin(async {
                            Ok(Response::new(full(
                                "Has All Params"
                            )))
                        })
                    }
                } 
                
                Box::pin(async { 
                    let mut bad_request = Response::new(empty());
                    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                    Ok(bad_request) 
                })   
                
            }
            _ => {
                Box::pin(async {
                    let mut not_found = Response::new(empty());
                    *not_found.status_mut() = StatusCode::NOT_FOUND;
                    Ok(not_found)
                })
                
            }
        }
    }
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

