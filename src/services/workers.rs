use tokio::task::JoinHandle;
use tokio::time::{ interval, Duration };
use chrono::{Utc};
use std::sync::Arc;

use crate::config::{ConfigHandler};
use crate::sql::clear_expired_grants;

pub struct CleanupWorker {
    pub handle: JoinHandle<()>
}

impl CleanupWorker {
    pub fn new(config: Arc<ConfigHandler>) -> CleanupWorker {
        CleanupWorker {
            handle: tokio::spawn(async move {
                let mut loop_interval = interval(Duration::from_secs(60));
                loop {
                    let current_timestamp = Utc::now();
                    loop_interval.tick().await;
                    
                    clear_expired_grants(&config.sql_connection_string, &current_timestamp).await;
                    
                    //TODO: need to come up with come unified error
                    //propogation system to handle errors from service threads
                }
            })
        } 
    }
}
