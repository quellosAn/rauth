
use tokio_postgres::{NoTls};

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("src\\sql\\migrations");
}

pub async fn update_schema() {
    
    let (mut client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=diceman16 dbname=dev port=5432", NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    

    embedded::migrations::runner().run_async(&mut client).await.unwrap();

}