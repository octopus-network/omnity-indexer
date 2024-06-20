use crate::types::*;
use crate::{
	bitcoin::{GenTicketRequest, ReleaseTokenStatus},
	icp::MintTokenStatus,
	types, Error as OmnityError,
};
use anyhow::{Error as AnyError, Result};
use candid::{Decode, Encode};
use config::{Config, ConfigError};
use ic_agent::identity::Secp256k1Identity;
use ic_agent::{agent::http_transport::ReqwestTransport, export::Principal, Agent, Identity};
use ic_btc_interface::Txid;
use lazy_static::lazy_static;
use log::{debug, info};
use sea_orm::{ConnectOptions, DatabaseConnection};
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{error::Error, future::Future, path::Path};

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

pub async fn with_agent<F, R>(f: F) -> Result<(), Box<dyn Error>>
where
	R: Future<Output = Result<(), Box<dyn Error>>>,
	F: FnOnce(Agent) -> R,
{
	let agent_identity = match std::env::var("DFX_IDENTITY") {
		Ok(identity) => {
			debug!("get identity from env var :{}", identity);

			let agent_identity = Secp256k1Identity::from_pem(identity.as_bytes())?;
			agent_identity
		}
		Err(_) => {
			let identity = read_config(|c| c.dfx_identity.to_owned())
				.ok_or_else(|| AnyError::msg("Cannot find identity file"))?;
			debug!("get identity from  config file :{identity:?}");

			let pem_file = Path::new(&identity);
			let agent_identity = Secp256k1Identity::from_pem_file(pem_file)?;
			agent_identity
		}
	};

	with_agent_as(agent_identity, f).await?;
	Ok(())
}

pub async fn with_agent_as<I, F, R>(agent_identity: I, f: F) -> Result<(), Box<dyn Error>>
where
	I: Identity + 'static,
	R: Future<Output = Result<(), Box<dyn Error>>>,
	F: FnOnce(Agent) -> R,
{
	let agent = create_agent(agent_identity).await?;
	agent.fetch_root_key().await?;

	f(agent).await
}

pub struct Database {
	pub connection: Arc<DatabaseConnection>,
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
	pub dfx_identity: Option<String>,
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

	pub fn get(&self, field: &str) -> Result<String, String> {
		match field.to_lowercase().as_str() {
			"database_url" => Ok(self.database_url.to_owned()),
			"dfx_identity" => Ok(self.dfx_identity.to_owned().unwrap()),
			"dfx_network" => Ok(self.dfx_network.to_owned()),
			"omnity_hub_canister_id" => Ok(self.omnity_hub_canister_id.to_owned()),
			"omnity_customs_bitcoin_canister_id" => {
				Ok(self.omnity_customs_bitcoin_canister_id.to_owned())
			}
			"omnity_routes_icp_canister_id" => Ok(self.omnity_routes_icp_canister_id.to_owned()),
			_ => Err(format!("Invalid field name to get '{}'", field)),
		}
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

pub async fn with_omnity_canister<F, R>(canister: &str, f: F) -> Result<(), Box<dyn Error>>
where
	R: Future<Output = Result<(), Box<dyn Error>>>,
	F: FnOnce(Agent, Principal) -> R,
{
	with_agent(|agent| async move {
		let canister_id = create_omnity_canister(canister).await?;
		f(agent, canister_id).await
	})
	.await
}

pub async fn with_omnity_bitcoin_canister_as<I, F, R>(
	identity: I,
	canister: &str,
	f: F,
) -> Result<(), Box<dyn Error>>
where
	I: Identity + 'static,
	R: Future<Output = Result<(), Box<dyn Error>>>,
	F: FnOnce(Agent, Principal) -> R,
{
	with_agent_as(identity, |agent| async move {
		let canister_id = create_omnity_canister(canister).await?;
		f(agent, canister_id).await
	})
	.await
}

pub async fn create_omnity_canister(canister: &str) -> Result<Principal, Box<dyn Error>> {
	match std::env::var(canister) {
		Ok(canister_id) => {
			info!(
				"Getting {} canister id from env var: {}",
				canister, canister_id
			);
			Ok(Principal::from_text(canister_id)?)
		}
		Err(_) => {
			let canister_id = read_config(|c| c.get(canister))?;
			info!("Getting {canister:?} canister id from config file: {canister_id:?}");
			Ok(Principal::from_text(canister_id)?)
		}
	}
}

pub enum ReturnType {
	U64(u64),
	VecChainMeta(Vec<ChainMeta>),
	VecTokenMeta(Vec<TokenMeta>),
	VecOmnityTicket(Vec<(u64, OmnityTicket)>),
	VecGenTicketRequest(Vec<GenTicketRequest>),
	MintTokenStatus(MintTokenStatus),
	ReleaseTokenStatus(ReleaseTokenStatus),
	Non(()),
}

impl ReturnType {
	pub fn convert_to_u64(&self) -> u64 {
		match self {
			Self::U64(u) => return *u,
			_ => return 0,
		}
	}
	pub fn convert_to_vec_chain_meta(&self) -> Vec<ChainMeta> {
		match self {
			Self::VecChainMeta(v) => return v.to_vec(),
			_ => return Vec::new(),
		}
	}
	pub fn convert_to_vec_token_meta(&self) -> Vec<TokenMeta> {
		match self {
			Self::VecTokenMeta(t) => return t.to_vec(),
			_ => return Vec::new(),
		}
	}
	pub fn convert_to_vec_omnity_ticket(&self) -> Vec<(u64, OmnityTicket)> {
		match self {
			Self::VecOmnityTicket(o) => return o.to_vec(),
			_ => return Vec::new(),
		}
	}
	pub fn convert_to_vec_gen_ticket_request(&self) -> Vec<GenTicketRequest> {
		match self {
			Self::VecGenTicketRequest(g) => return g.to_vec(),
			_ => return Vec::new(),
		}
	}
	pub fn convert_to_mint_token_status(&self) -> MintTokenStatus {
		match self {
			Self::MintTokenStatus(m) => return m.clone(),
			_ => return MintTokenStatus::Unknown,
		}
	}
	pub fn convert_to_release_token_status(&self) -> ReleaseTokenStatus {
		match self {
			Self::ReleaseTokenStatus(r) => return r.clone(),
			_ => return ReleaseTokenStatus::Unknown,
		}
	}
}
pub enum Arg {
	V(Vec<u8>),
	T(types::Ticket),
	U(u64),
	TI(TicketId),
}

impl Arg {
	pub async fn query_method(
		self,
		agent: Agent,
		canister_id: Principal,
		method: &str,
		log_one: &str,
		log_two: &str,
		args_two: Option<u64>,
		re_type: &str,
	) -> Result<ReturnType, Box<dyn Error>> {
		info!("{:?} {:?}", chrono::Utc::now(), log_one);

		let encoded_args: Vec<u8> = match args_two {
			Some(arg) => match self {
				Arg::V(v) => Encode!(&v, &arg)?,
				Arg::T(t) => Encode!(&t, &arg)?,
				Arg::U(u) => Encode!(&u, &arg)?,
				Arg::TI(ti) => Encode!(&ti, &arg)?,
			},
			None => match self {
				Arg::V(v) => Encode!(&v)?,
				Arg::T(t) => Encode!(&t)?,
				Arg::U(u) => Encode!(&u)?,
				Arg::TI(ti) => Encode!(&ti)?,
			},
		};
		let return_output: Vec<u8> = agent
			.query(&canister_id, method)
			.with_arg(encoded_args)
			.call()
			.await?;

		match re_type {
			"u64" => {
				let decoded_return_output =
					Decode!(&return_output, Result<u64, OmnityError>)?.unwrap();
				info!("{:?} {:?}", log_two, decoded_return_output);
				return Ok(ReturnType::U64(decoded_return_output));
			}
			"Vec<ChainMeta>" => {
				let decoded_return_output =
					Decode!(&return_output, Result<Vec<ChainMeta>, OmnityError>)?.unwrap();
				info!("{:?} {:?}", log_two, decoded_return_output);
				return Ok(ReturnType::VecChainMeta(decoded_return_output));
			}
			"Vec<TokenMeta>" => {
				let decoded_return_output =
					Decode!(&return_output, Result<Vec<TokenMeta>, OmnityError>)?.unwrap();
				info!("{:?} {:?}", log_two, decoded_return_output);
				return Ok(ReturnType::VecTokenMeta(decoded_return_output));
			}
			"Vec<(u64, OmnityTicket)>" => {
				let decoded_return_output = Decode!(
					&return_output,
					Result<Vec<(u64, OmnityTicket)>, OmnityError>
				)?
				.unwrap();
				info!("{:?} {:?}", log_two, decoded_return_output);
				return Ok(ReturnType::VecOmnityTicket(decoded_return_output));
			}
			"Vec<GenTicketRequest>" => {
				let decoded_return_output = Decode!(&return_output, Vec<GenTicketRequest>)?;
				info!("{:?} {:?}", log_two, decoded_return_output);
				return Ok(ReturnType::VecGenTicketRequest(decoded_return_output));
			}
			"MintTokenStatus" => {
				let decoded_return_output = Decode!(&return_output, MintTokenStatus)?;
				info!("{:?} {:?}", log_two, decoded_return_output);
				return Ok(ReturnType::MintTokenStatus(decoded_return_output));
			}
			"ReleaseTokenStatus" => {
				let decoded_return_output = Decode!(&return_output, ReleaseTokenStatus)?;
				info!("{:?} {:?}", log_two, decoded_return_output);
				return Ok(ReturnType::ReleaseTokenStatus(decoded_return_output));
			}
			_ => {
				let decoded_return_output =
					Decode!(&return_output, Result<(), OmnityError>)?.unwrap();
				info!("{:?} {:?}", log_two, decoded_return_output);
				return Ok(ReturnType::Non(()));
			}
		};
	}
}
