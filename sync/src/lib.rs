mod db;

#[cfg(debug_assertions)]
use dotenvy::dotenv;

#[tokio::main]
pub async fn main() {
    #[cfg(debug_assertions)]
    dotenv().ok();
}
