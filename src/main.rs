use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::collections::HashMap;
use dashmap::DashMap;
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
use tokio::task::JoinHandle;
use tokio::time::{ interval, Duration, Instant };
use uuid::Uuid;



#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    let listener = TcpListener::bind(addr).await.unwrap();
    let identityStore = IdentityStore::new();
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

const REQUIRED_PARAMS: &[&str; 3] = &["response_type", "client_id", "scope"];

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
            
        }
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
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

struct IdentityStore {
    session_queue: Arc<Mutex<VecDeque<Session>>>,
    session_store: DashMap<String, SessionRecord>,
    cleanup_handle: JoinHandle<()>
}

impl IdentityStore {
    fn new() -> IdentityStore {
        let local_session_queue = Arc::new(Mutex::new(VecDeque::new()));
        let local_session_store = DashMap::new();
        IdentityStore {
            session_queue: local_session_queue.clone(),
            session_store: local_session_store,
            cleanup_handle: tokio::spawn(async move {
                let mut loop_interval = interval(Duration::from_secs(60));
                loop {
                    loop_interval.tick().await;
                    //TODO: write some sort of logic to resize queue if required
                    let mut locked_queue = local_session_queue.lock().unwrap();
                    if !locked_queue.is_empty() {
                        let current_timestamp = Instant::now();
                        let removal_point = locked_queue.partition_point(
                            |session| current_timestamp.duration_since(session.timestamp).as_secs() > 60
                        );
                        if removal_point != 0 {
                            locked_queue.drain(removal_point + 1..);
                        }
                    }
                    
                }
            })
        } 
    }
}

struct Session {
    session_id: Uuid,
    timestamp: Instant
}

struct SessionRecord {
    session_id: Uuid,
    auth_code: Uuid,
    timestamp: Instant
}