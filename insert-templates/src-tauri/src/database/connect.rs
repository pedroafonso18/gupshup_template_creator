use sqlx::{postgres::PgPool, Error, Pool, Postgres};

pub async fn connect_db(db_url: &str) -> Result<Pool<Postgres>, Error> {
    let pool = PgPool::connect(db_url).await?;

    Ok(pool)
}