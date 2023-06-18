use tokio::task::JoinHandle;
use tokio::time::{ interval, Duration };
use chrono::Utc;

use crate::sql::clear_expired_grants;
use crate::SERVER_CONFIG;

pub struct CleanupWorker {
    pub handle: JoinHandle<()>
}

impl CleanupWorker {
    pub fn new() -> CleanupWorker {
        CleanupWorker {
            handle: tokio::spawn(async move {
                let mut loop_interval = interval(Duration::from_secs(60));
                loop {
                    let current_timestamp = Utc::now();
                    loop_interval.tick().await;
                    
                    clear_expired_grants(&SERVER_CONFIG.sql_connection_string, &current_timestamp).await;
                    
                    //TODO: need to come up with come unified error
                    //propogation system to handle errors from service threads
                    //if there ends up being more than one
                }
            })
        } 
    }
}
