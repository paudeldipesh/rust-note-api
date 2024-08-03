use dotenv::dotenv;
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref ADDRESS: String = set_address();
    pub static ref DATABASE_URL: String = set_database_url();
    pub static ref PORT: u16 = set_port();
    pub static ref SECRET: String = set_secret();
}

fn set_address() -> String {
    dotenv().ok();
    env::var("ADDRESS").expect("ADDRESS must be set")
}

fn set_secret() -> String {
    dotenv().ok();
    env::var("SECRET").expect("SECRET must be set")
}

fn set_database_url() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

fn set_port() -> u16 {
    dotenv().ok();
    env::var("PORT")
        .expect("PORT must be set")
        .parse::<u16>()
        .expect("PORT must be a valid u16 number")
}
