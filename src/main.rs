use std::net::SocketAddr;
use std::collections::HashMap;
use config::ConfigHandler;
use crypto::verify_password;
use http_body_util::{
    Full,
    Empty,
    BodyExt,
    combinators::BoxBody
};
use hyper::{
    Request,
    Response,
    Method, 
    StatusCode,
    body::Bytes,
    server::conn::http1,
    service::service_fn
};
use serde::Deserialize;
use services::workers::CleanupWorker;
use sql::{ update_schema, fetch_user };
use tokio::net::TcpListener;
use lazy_static::lazy_static;

mod services;
mod sql;
mod config;
mod crypto;

lazy_static! {
    pub static ref SERVER_CONFIG: ConfigHandler = ConfigHandler::parse_config(
        std::env::args().nth(1).expect("No config path provided.")
    ).unwrap();
}

#[tokio::main]
async fn main() {

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));

    //TODO: Port should be provided by config
    let listener = TcpListener::bind(addr).await.expect("Unable to bind to provided port.");
    update_schema(&SERVER_CONFIG.sql_connection_string).await;
    let _worker = CleanupWorker::new();
    
    loop {
        let (stream, _) = listener.accept().await.unwrap();

        //clone config reference and move to 
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(auth_service))
                .await {
                    println!("Error serving connection: {:?}", err);
            }
        });
    }
}

const REQUIRED_PARAMS: &[&str; 3] = &["response_type", "client_id", "scope"];


#[derive(Deserialize, Debug)]
pub struct LoginRequestBody {
    username: String,
    password: String
}

async fn auth_service(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match(req.method(), req.uri().path()) {
        (&Method::GET, "/authorize") => {
            if let Some(query) = req.uri().query() {
                let test:HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
                    .into_owned()
                    .collect();
                if REQUIRED_PARAMS.iter().all(|param| test.contains_key(*param)) {
                    return Ok(Response::new(full(
                        "Has All Params"
                    )))
                }
            } 
            
            let mut bad_request = Response::new(empty());
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            Ok(bad_request)    
            
        }
        (&Method::POST, "/Login") => {
            
            let incoming_body = req.collect().await?.to_bytes().to_vec();
            let login_request = serde_json::from_slice::<LoginRequestBody>(&incoming_body);
            
            //TODO: Decide on max request size.
            if let Ok(login_info) = login_request {
                process_login(login_info).await;
            }
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
        (&Method::POST, "/CreateAccount") => {
            Ok(Response::new(full(
                "Login Endpoint"
            )))
        }
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
            
        }
    }
}

async fn process_login(login_info: LoginRequestBody) {
    if let Some(user) = fetch_user(
        &SERVER_CONFIG.sql_connection_string, 
        &login_info.username).await {

        if let Ok(valid) = verify_password(&login_info.password, &user.password_hash) {

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

