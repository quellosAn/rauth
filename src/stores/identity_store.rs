use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use tokio::task::JoinHandle;
use tokio::time::{ interval, Duration, Instant };
use uuid::Uuid;

pub struct IdentityStore {
    session_queue: Arc<Mutex<VecDeque<Session>>>,
    session_store: Arc<DashMap<String, SessionRecord>>,
    cleanup_handle: JoinHandle<()>
}

impl IdentityStore {
    pub fn new() -> IdentityStore {
        let local_session_queue = Arc::new(Mutex::new(VecDeque::new()));
        let local_session_store = Arc::new(DashMap::new());
        IdentityStore {
            session_queue: local_session_queue.clone(),
            session_store: local_session_store.clone(),
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
                            let expired_sessions = locked_queue.drain(removal_point + 1..);
                            for sessions in expired_sessions {
                                local_session_store.remove(&sessions.session_id.to_string());
                            }
                        }
                    }
                }
            })
        } 
    }

    pub fn verify_session(&self, session_id: String) -> bool {
        match self.session_store.remove(&session_id) {
            Some((_, session)) => {
                if Instant::now().duration_since(session.timestamp).as_secs() > 60 {
                    return false;
                }
                return true;
            },
            None => false,
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