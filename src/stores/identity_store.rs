use tokio::task::JoinHandle;
use tokio::time::{ interval, Duration };
use chrono::{DateTime, Utc};
use tokio_postgres::{self, NoTls};
use uuid::Uuid;

pub struct IdentityStore {
    cleanup_handle: JoinHandle<()>

}

impl IdentityStore {
    pub fn new(connection_string: String) -> IdentityStore {
        IdentityStore {
            cleanup_handle: tokio::spawn(async move {
                let local_connection_string = connection_string.clone();
                let mut loop_interval = interval(Duration::from_secs(60));
                loop {
                    let current_timestamp = Utc::now();
                    loop_interval.tick().await;
                    let (client, connection) =
                        tokio_postgres::connect(&local_connection_string, NoTls).await.unwrap();
                    
                    //spin off connection thread
                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            eprintln!("connection error: {}", e);
                        }
                    });
                    
                    let _result = client.execute(
            "DELETE FROM persisted_grant WHERE create_time <= $n",
                        &[&current_timestamp])
                        .await.unwrap();
                    //TODO: need to come up with come unified error
                    //propogation system to handle errors from service threads
                }
            })
        } 
    }

    //pub fn verify_session(&self, session_id: String) -> bool {

    //}
}

struct Session {
    session_id: Uuid,
    timestamp: DateTime<Utc>
}

struct SessionRecord {
    session_id: Uuid,
    auth_code: Uuid,
    timestamp: DateTime<Utc>
}