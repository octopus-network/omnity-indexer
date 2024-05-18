use config::{Config, ConfigError};

use ic_agent::agent::http_transport::ReqwestTransport;
use ic_agent::identity::{Prime256v1Identity, Secp256k1Identity};
use ic_agent::{export::Principal, identity::BasicIdentity, Agent, Identity};
use ic_btc_interface::Txid;
use ic_identity_hsm::HardwareIdentity;

use lazy_static::lazy_static;
use log::{debug, info};
use ring::signature::Ed25519KeyPair;
use sea_orm::DatabaseConnection;
use sea_orm::{ConnectOptions, DbConn};
use serde::Deserialize;

use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{error::Error, future::Future, path::Path};

const HSM_PKCS11_LIBRARY_PATH: &str = "HSM_PKCS11_LIBRARY_PATH";
const HSM_SLOT_INDEX: &str = "HSM_SLOT_INDEX";
const HSM_KEY_ID: &str = "HSM_KEY_ID";
const HSM_PIN: &str = "HSM_PIN";
// const LOCAL_NET: &str = "http://127.0.0.1:4943";

pub fn get_effective_canister_id() -> Principal {
    Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap()
}

pub fn create_identity() -> Result<Box<dyn Identity>, String> {
    if std::env::var(HSM_PKCS11_LIBRARY_PATH).is_ok() {
        create_hsm_identity().map(|x| Box::new(x) as _)
    } else {
        create_basic_identity().map(|x| Box::new(x) as _)
    }
}

fn expect_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("Need to specify the {} environment variable", name))
}

pub fn create_hsm_identity() -> Result<HardwareIdentity, String> {
    let path = expect_env_var(HSM_PKCS11_LIBRARY_PATH)?;
    let slot_index = expect_env_var(HSM_SLOT_INDEX)?
        .parse::<usize>()
        .map_err(|e| format!("Unable to parse {} value: {}", HSM_SLOT_INDEX, e))?;
    let key = expect_env_var(HSM_KEY_ID)?;
    let id = HardwareIdentity::new(path, slot_index, &key, get_hsm_pin)
        .map_err(|e| format!("Unable to create hw identity: {}", e))?;
    Ok(id)
}

fn get_hsm_pin() -> Result<String, String> {
    expect_env_var(HSM_PIN)
}

// The SoftHSM library doesn't like to have two contexts created/initialized at once.
// Trying to create two HardwareIdentity instances at the same time results in this error:
//    Unable to create hw identity: PKCS#11: CKR_CRYPTOKI_ALREADY_INITIALIZED (0x191)
//
// To avoid this, we use a basic identity for any second identity in tests.
//
// A shared container of Ctx objects might be possible instead, but my rust-fu is inadequate.
pub fn create_basic_identity() -> Result<BasicIdentity, String> {
    let rng = ring::rand::SystemRandom::new();
    let key_pair = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
        .expect("Could not generate a key pair.");

    Ok(BasicIdentity::from_key_pair(
        Ed25519KeyPair::from_pkcs8(key_pair.as_ref()).expect("Could not read the key pair."),
    ))
}

/// Create a secp256k1identity, which unfortunately will always be the same one
/// (So can only use one per test)
pub fn create_secp256k1_identity() -> Result<Secp256k1Identity, String> {
    // generated from the the following commands:
    // $ openssl ecparam -name secp256k1 -genkey -noout -out identity.pem
    // $ cat identity.pem
    let identity_file = "
-----BEGIN EC PRIVATE KEY-----
MHQCAQEEIJb2C89BvmJERgnT/vJLKpdHZb/hqTiC8EY2QtBRWZScoAcGBSuBBAAK
oUQDQgAEDMl7g3vGKLsiLDA3fBRxDE9ZkM3GezZFa5HlKM/gYzNZfU3w8Tijjd73
yeMC60IsMNxDjLqElV7+T7dkb5Ki7Q==
-----END EC PRIVATE KEY-----";

    let identity = Secp256k1Identity::from_pem(identity_file.as_bytes())
        .expect("Cannot create secp256k1 identity from PEM file.");
    Ok(identity)
}

pub fn create_prime256v1_identity() -> Result<Prime256v1Identity, String> {
    // generated from the following command:
    // $ openssl ecparam -name prime256v1 -genkey -noout -out identity.pem
    // $ cat identity.pem
    let identity_file = "\
-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIL1ybmbwx+uKYsscOZcv71MmKhrNqfPP0ke1unET5AY4oAoGCCqGSM49
AwEHoUQDQgAEUbbZV4NerZTPWfbQ749/GNLu8TaH8BUS/I7/+ipsu+MPywfnBFIZ
Sks4xGbA/ZbazsrMl4v446U5UIVxCGGaKw==
-----END EC PRIVATE KEY-----";

    let identity = Prime256v1Identity::from_pem(identity_file.as_bytes())
        .expect("Cannot create prime256v1 identity from PEM file.");
    Ok(identity)
}

pub async fn create_agent(identity: impl Identity + 'static) -> Result<Agent, String> {
    // let network = std::env::var("DFX_NETWORK").unwrap_or_else(|_| LOCAL_NET.to_string());
    let network = match std::env::var("DFX_NETWORK") {
        Ok(network) => {
            debug!("get network from env var :{}", network);
            network
        }
        Err(_) => {
            let network = read_config(|c| c.dfx_network.to_owned());
            debug!("get network from  config file :{network:?}");

            network
        }
    };

    Agent::builder()
        .with_transport(ReqwestTransport::create(network).unwrap())
        .with_identity(identity)
        .build()
        .map_err(|e| format!("{:?}", e))
}

pub async fn with_agent<F, R>(f: F)
where
    R: Future<Output = Result<(), Box<dyn Error>>>,
    F: FnOnce(Agent) -> R,
{
    let identity = match std::env::var("DFX_IDENTITY") {
        Ok(identity) => {
            debug!("get identity from env var :{}", identity);
            identity
        }
        Err(_) => {
            let identity = read_config(|c| c.dfx_identity.to_owned());
            debug!("get identity from  config file :{identity:?}");

            identity
        }
    };
    let pem_file = Path::new(&identity);
    let agent_identity = Secp256k1Identity::from_pem_file(pem_file)
        .expect("Could not create an identity from PEM file.");

    with_agent_as(agent_identity, f).await
}

pub async fn with_agent_as<I, F, R>(agent_identity: I, f: F)
where
    I: Identity + 'static,
    R: Future<Output = Result<(), Box<dyn Error>>>,
    F: FnOnce(Agent) -> R,
{
    let agent = create_agent(agent_identity)
        .await
        .expect("Could not create an agent.");
    agent
        .fetch_root_key()
        .await
        .expect("could not fetch root key");
    match f(agent).await {
        Ok(_) => {}
        Err(e) => panic!("{:?}", e),
    };
}

pub struct Database {
    pub connection: Arc<DatabaseConnection>, //
}

impl Database {
    pub async fn new(db_url: String) -> Self {
        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(false)
            .sqlx_logging_level(log::LevelFilter::Info);
        // .set_schema_search_path("omnity"); // Setting default PostgreSQL schema

        let connection = sea_orm::Database::connect(opt)
            .await
            .expect("Could not connect to database");
        assert!(connection.ping().await.is_ok());
        info!("Connected to database !");

        Database {
            connection: Arc::new(connection),
        }
    }

    pub fn get_connection(&self) -> Arc<DatabaseConnection> {
        self.connection.clone()
    }
}

pub fn get_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_nanos() as u64
}

pub fn random_txid() -> Txid {
    let txid: [u8; 32] = rand::random();
    txid.into()
}

#[derive(Debug, Deserialize, Default)]
#[allow(unused)]
pub struct Settings {
    pub database_url: String,
    pub dfx_identity: String,
    pub dfx_network: String,
    pub omnity_hub_canister_id: String,
    pub omnity_customs_bitcoin_canister_id: String,
    pub omnity_routes_icp_canister_id: String,
    pub log_config: String,
}
impl Settings {
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(config::File::with_name(config_path))
            .build()?;
        config.try_deserialize()
    }
}

lazy_static! {
    static ref CONFIG: RwLock<Settings> = RwLock::new(Settings::default());
}

pub fn mutate_config<F, R>(f: F) -> R
where
    F: FnOnce(&mut Settings) -> R,
{
    f(&mut CONFIG.write().unwrap())
}

pub fn read_config<F, R>(f: F) -> R
where
    F: FnOnce(&Settings) -> R,
{
    f(&CONFIG.read().unwrap())
}

/// Replaces the current state.
pub fn set_config(setting: Settings) {
    *CONFIG.write().unwrap() = setting;
}

pub fn spawn_sync_task<F, Fut>(db: &Database, sync_fn: F) -> tokio::task::JoinHandle<()>
where
    F: Fn(&DbConn) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let db_conn = db.get_connection();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            sync_fn(&db_conn).await;
            interval.tick().await;
        }
    })
}
