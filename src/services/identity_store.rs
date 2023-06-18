use std::sync::Arc;

use tokio::task::JoinHandle;
use tokio_postgres::{self, Client, Connection, Socket, tls::NoTlsStream, NoTls};
use chrono::{DateTime, Utc};
use uuid::Uuid;

struct Session {
    session_id: Uuid,
    timestamp: DateTime<Utc>
}

struct SessionRecord {
    session_id: Uuid,
    auth_code: Uuid,
    timestamp: DateTime<Utc>
}