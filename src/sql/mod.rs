use chrono::{DateTime, Utc};
use tokio_postgres::{NoTls, Client};

use crate::{CreateAccountRequestBody, SERVER_CONFIG};

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("src\\sql\\migrations");
}

async fn connect() -> Client {

    let (client, connection) = tokio_postgres::connect(&SERVER_CONFIG.sql_connection_string, NoTls).await.unwrap();
    tokio::spawn(connection);
    client
}

pub async fn update_schema() {
    
    let mut client = connect().await;
    println!("Updating Schema");

    embedded::migrations::runner().run_async(&mut client).await.unwrap();

}

pub async fn clear_expired_grants(connection_string: &String, current_timestamp: &DateTime<chrono::offset::Utc>) {
    let client = connect().await;
    
    let _result = client.execute(
        "DELETE FROM persisted_grant WHERE create_time <= $1",
        &[&current_timestamp])
        .await.unwrap();
}

pub struct User {
    pub user_id: uuid::Uuid,
    pub access_failed_count: i32,
    pub email: String,
    pub email_confirmed: bool,
    pub lockout_enabled: Option<bool>,
    pub lockout_end: Option<chrono::DateTime<Utc>>,
    pub username: String,
    pub password_hash: String,
    pub phone_number: Option<String>,
    pub phone_number_confirmed: Option<bool>
}

pub async fn fetch_user(username: &String) -> Option<User> {
    let client = connect().await;

    let result = client.query_one(
        "
                    SELECT 
                        user_id,
                        access_failed_count,
                        email,
                        email_confirmed,
                        lockout_enabled,
                        lockout_end,
                        username,
                        password_hash,
                        phone_number,
                        phone_number_confirmed
                    FROM 
                        application_user
                    WHERE
                        username = $1;
                        ", 
        &[&username]).await;

        match result {
            Ok(user) => {
                Some(User {
                    user_id: user.get("user_id"),
                    access_failed_count: user.get("access_failed_count"),
                    email: user.get("email"),
                    email_confirmed: user.get("email_confirmed"),
                    lockout_enabled: user.get("lockout_enabled"),
                    lockout_end: user.get("lockout_end"),
                    username: user.get("username"),
                    password_hash: user.get("password_hash"),
                    phone_number: user.get("phone_number"),
                    phone_number_confirmed: user.get("phone_number_confirmed")
                })
            }
            Err(_) => None,
        }
    
}

pub async fn insert_user(create_body: CreateAccountRequestBody, password_hash: String) {
    let client = connect().await;
    
    client.execute(
        "
            INSERT INTO application_user
            (
                access_failed_count, email, email_confirmed, 
                lockout_enabled, lockout_end, username, password_hash, 
                created_on, last_modified_on, phone_number, phone_number_confirmed)
            VALUES
            (0, $1, b'1', NULL, NULL, $2, $3, $4, $5, NULL, NULL)
        ", 
        &[&create_body.email, 
                &create_body.username, 
                &password_hash, 
                &Utc::now(), 
                &Utc::now()]
    ).await.unwrap();
}