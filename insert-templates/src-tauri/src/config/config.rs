use dotenv::dotenv;
use std::env;

pub struct EnvVars {
    pub db_url: String,
    pub apikey: String,
    pub cookie: String,
}

pub fn load() -> EnvVars {
    dotenv().ok();

    let db_url = env::var("DB_URL").expect("No DB_URL found in the .env file!");
    let apikey = env::var("APIKEY").expect("No APIKEY found in the .env file!");
    let cookie = env::var("COOKIE").expect("No COOKIE found in the .env file!");

    EnvVars {
        db_url,
        apikey,
        cookie,
    }
}