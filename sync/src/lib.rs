pub mod customs;
pub mod hub;
pub mod routes;
pub mod types;
pub mod universal;
pub mod utils;
pub use utils::*;

#[cfg(debug_assertions)]
use dotenvy::dotenv;

#[tokio::main]
pub async fn main() {
    #[cfg(debug_assertions)]
    dotenv().ok();
}
