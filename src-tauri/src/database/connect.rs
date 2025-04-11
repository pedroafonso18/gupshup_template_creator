use sqlx::{postgres::PgConnection, Error, Connection};

pub async fn connect_db(db_url: &str) -> Result<PgConnection, Error> {
    println!("Attempting to connect to database...");
    let conn = PgConnection::connect(db_url).await?;
    println!("Database connection established successfully");
    Ok(conn)
}