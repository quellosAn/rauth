use chrono::{DateTime, Utc};
use tokio_postgres::{self, NoTls};

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("src\\sql\\migrations");
}


pub async fn update_schema(connection_str: &String) {
    
    let (mut client, connection) =
        tokio_postgres::connect(connection_str, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    println!("Updating Schema");

    embedded::migrations::runner().run_async(&mut client).await.unwrap();

}

pub async fn clear_expired_grants(connection_string: &String, current_timestamp: &DateTime<chrono::offset::Utc>) {
    let (client, connection) =
        tokio_postgres::connect(connection_string, NoTls).await.unwrap();

    //spin off connection thread
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    
    let _result = client.execute(
        "DELETE FROM persisted_grant WHERE create_time <= $1",
        &[&current_timestamp])
        .await.unwrap();
}

pub struct User {
    user_id: uuid::Uuid,
    access_failed_count: i32,
    email: String,
    email_confirmed: bool,
    lockout_enabled: Option<bool>,
    lockout_end: Option<chrono::DateTime<Utc>>,
    username: String,
    password_hash: String,
    phone_number: Option<String>,
    phone_number_confirmed: Option<bool>
}

pub async fn fetch_user(connection_string: &String, username: &String) -> Option<User> {
    let (client, connection) =
        tokio_postgres::connect(connection_string, NoTls).await.unwrap();

    //spin off connection thread
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

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