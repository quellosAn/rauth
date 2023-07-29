use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use config::ConfigHandler;
use crypto::{verify_password, hash_password};
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
    body::{Bytes, Body},
    server::conn::http1,
    service::service_fn
};
use lettre::message::Mailbox;
use serde::Deserialize;
use services::workers::CleanupWorker;
use sql::{ update_schema, fetch_user, insert_user };
use tokio::join;
use tokio::net::TcpListener;
use lazy_static::lazy_static;
use tokio_rustls::{
    TlsAcceptor,
    rustls::ServerConfig
};
use env_logger::{
    Builder, 
    Target
};
use services::email::send_email_verification;
use log::info;
use log::error;

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

    init_logger();
    //No error handing during this stretch of code.
    //The server cannot/shouldn't be allowed to start 
    //if these initialization steps fail. These include
    //operations such as TLS setup and port binding.
    update_schema().await
        .expect("Unable to apply sql migrations, likely caused by a server connection issue.");
    let parsed_address = Ipv4Addr::from_str(&SERVER_CONFIG.server_address)
        .expect("Unable to parse provided Ip Address.");
    let addr = SocketAddr::new(
        IpAddr::V4(parsed_address), SERVER_CONFIG.server_port
    );
    let listener = TcpListener::bind(addr).await
        .expect("Unable to bind to provided port.");
    let certs = crypto::load_cert(&SERVER_CONFIG.cert)
        .expect("Unable to parse or find provided TLS certificate.");
    let mut keys = crypto::load_key(&SERVER_CONFIG.key)
        .expect("Unable to parse or find provided TLS key.");
    //TLS initialization
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))
        .expect("Unable to set up TLS with provided certificate and key.");

    let _worker = CleanupWorker::new();
    let acceptor = TlsAcceptor::from(Arc::new(config));

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let acceptor = acceptor.clone();

        match acceptor.accept(stream).await {
            Ok(stream) => {
                tokio::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(stream, service_fn(auth_service))
                        .await {
                            error!("Error serving connection: {}", err);
                    }
                });
            },
            Err(tls_error) => {
                error!("Error completing TLS handshake {}", tls_error);
            }
        }
    }
}

const REQUIRED_PARAMS: &[&str; 3] = &["response_type", "client_id", "scope"];


#[derive(Deserialize, Debug)]
pub struct LoginRequestBody {
    username: String,
    password: String
}

#[derive(Deserialize, Debug)]
pub struct CreateAccountRequest {
    username: String,
    password: String,
    email: Mailbox
}

impl CreateAccountRequest {
    async fn process_request(&self) -> Result<bool, argon2::password_hash::Error>{
        if password_valid(&self.password) {
            let hash = hash_password(&self.password)?;
            let insert_fut = insert_user(self, hash);
            if let Some(config) = &SERVER_CONFIG.email_config {
                let email_fut = send_email_verification(&self.email, config);
                let ( _, email_response ) = join!(insert_fut, email_fut);
                if email_response.is_ok() {
                    
                } 
            } 
            else 
            {
                insert_fut.await;
            }
            return Ok(true); 
        }
        return Ok(false); 
    }
}


async fn auth_service(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if upper > 1024 * 64 {
        let mut resp = Response::new(full("Body too big"));
        *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(resp);
    }
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
            
            if let Ok(login_info) = login_request {
                process_login(login_info).await;
            }
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
        (&Method::POST, "/CreateAccount") => {
            let incoming_body = req.collect().await?.to_bytes().to_vec();
            let create_request = serde_json::from_slice::<CreateAccountRequest>(&incoming_body);
            let mut res = Response::new(empty());
            if let Ok(account_info) = create_request {
                match account_info.process_request().await {
                    Ok(_) => {
                        return Ok(Response::new(empty()))
                    },
                    Err(hash_error) => {
                        error!("Password hash failed with error {}", hash_error);
                    }
                }
            } else {
                info!("/CreateAccount malformed JSON body, request discarded");
                *res.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(res);
            }
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(res);
        }
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}


async fn process_login(login_info: LoginRequestBody) {
    if let Some(user) = fetch_user(&login_info.username).await {
        if let Ok(_valid) = verify_password(&login_info.password, &user.password_hash) {
            todo!("Build out login process")
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

fn init_logger() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();
}

fn password_valid(password: &String) -> bool {
    let requirements = &SERVER_CONFIG.password_requirments;
    let pass_length = password.len();
    let mut disallowed_chars = requirements.forbidden_characters.chars();
    
    pass_length > requirements.maximum_size && 
    pass_length < requirements.minimum_size &&
    disallowed_chars.any(|pass_char| password.contains(pass_char))
}