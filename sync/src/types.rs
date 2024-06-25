use crate::entity;
use candid::CandidType;
use entity::{chain_meta, sea_orm_active_enums, ticket, token_meta};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

pub type Signature = Vec<u8>;
pub type Seq = u64;
pub type Account = String;
pub type Amount = u128;
pub type ChainId = String;
pub type DstChain = ChainId;
pub type TokenId = String;
pub type Timestamp = u64;
pub type TicketId = String;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChainMeta {
	pub chain_id: ChainId,
	pub canister_id: String,
	pub chain_type: ChainType,
	// the chain default state is active
	pub chain_state: ChainState,
	// settlement chain: export contract address
	// execution chain: port contract address
	pub contract_address: Option<String>,
	pub counterparties: Option<Vec<ChainId>>,
	pub fee_token: Option<TokenId>,
}

impl core::fmt::Display for ChainMeta {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
            f,
            "\nchain id:{} \ncanister id:{} \nchain type:{:?} \nchain state:{:?} \ncontract address:{:?} \ncounterparties:{:?} \nfee_token:{:?}",
            self.chain_id,self.canister_id, self.chain_type, self.chain_state, self.contract_address,self.counterparties,self.fee_token,
        )
	}
}

impl Into<Chain> for ChainMeta {
	fn into(self) -> Chain {
		Chain {
			chain_id: self.chain_id,
			canister_id: self.canister_id,
			chain_type: self.chain_type,
			chain_state: self.chain_state,
			contract_address: self.contract_address,
			counterparties: self.counterparties,
			fee_token: self.fee_token,
		}
	}
}

impl From<ChainMeta> for chain_meta::Model {
	fn from(chain: ChainMeta) -> Self {
		chain_meta::Model {
			chain_id: chain.chain_id,
			canister_id: chain.canister_id,
			chain_type: chain.chain_type.into(),
			chain_state: chain.chain_state.into(),
			contract_address: chain.contract_address,
			counterparties: chain.counterparties.map(|cs| json!(cs)),
			fee_token: chain.fee_token,
		}
	}
}

impl From<chain_meta::Model> for ChainMeta {
	fn from(model: chain_meta::Model) -> Self {
		ChainMeta {
			chain_id: model.chain_id,
			canister_id: model.canister_id,
			chain_type: model.chain_type.into(),
			chain_state: model.chain_state.into(),
			contract_address: model.contract_address,
			counterparties: model
				.counterparties
				.map(|cs| serde_json::from_value(cs).expect("Failed to parse counterparties")),
			fee_token: model.fee_token,
		}
	}
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenMeta {
	pub token_id: TokenId,
	pub name: String,
	pub symbol: String,
	pub issue_chain: ChainId,
	pub decimals: u8,
	pub icon: Option<String>,
	pub metadata: HashMap<String, String>,
	pub dst_chains: Vec<ChainId>,
}

impl core::fmt::Display for TokenMeta {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
            f,
            "\ttoken id:{} \ntoken name:{} \nsymbol:{:?} \nissue chain:{} \ndecimals:{} \nicon:{:?} \nmetadata:{:?} \ndst chains:{:?}",
            self.token_id, self.name,self.symbol, self.issue_chain, self.decimals, self.icon,self.metadata,self.dst_chains
        )
	}
}

impl Into<Token> for TokenMeta {
	fn into(self) -> Token {
		Token {
			token_id: self.token_id,
			name: self.name,
			symbol: self.symbol,
			decimals: self.decimals,
			icon: self.icon,
			metadata: self.metadata,
		}
	}
}

impl From<TokenMeta> for token_meta::Model {
	fn from(token_meta: TokenMeta) -> Self {
		token_meta::Model {
			token_id: token_meta.token_id,
			name: token_meta.name,
			symbol: token_meta.symbol,
			issue_chain: token_meta.issue_chain,
			decimals: token_meta.decimals as i16,
			icon: token_meta.icon,
			metadata: json!(token_meta.metadata),
			dst_chains: json!(token_meta.dst_chains),
			token_on_chain: None,
		}
	}
}

impl From<token_meta::Model> for TokenMeta {
	fn from(model: token_meta::Model) -> Self {
		TokenMeta {
			token_id: model.token_id,
			name: model.name,
			symbol: model.symbol,
			issue_chain: model.issue_chain,
			decimals: model.decimals as u8,
			icon: model.icon,
			metadata: serde_json::from_value(model.metadata).expect("Failed to parse metadata"),
			dst_chains: serde_json::from_value(model.dst_chains)
				.expect("Failed to parse dst_chains"),
		}
	}
}


impl token_meta::Model {
    pub fn new(token_meta: TokenMeta, token_on_chain_data: Vec<OmnityTokenOnChain>) -> Self {
        token_meta::Model {
            token_id: token_meta.token_id,
            name: token_meta.name,
            symbol: token_meta.symbol,
            issue_chain: token_meta.issue_chain,
            decimals: token_meta.decimals as i16,
            icon: token_meta.icon,
            metadata: json!(token_meta.metadata),
            dst_chains: json!(token_meta.dst_chains),
            token_on_chain: json!(Some(token_on_chain_data)).into(),
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct OmnityTokenOnChain {
    // the chain of the token be locked
    pub chain_id: ChainId,
    pub token_id: TokenId,
    pub amount: u128,
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub enum Directive {
	AddChain(Chain),
	AddToken(Token),
	UpdateChain(Chain),
	UpdateToken(Token),
	ToggleChainState(ToggleState),
	UpdateFee(Factor),
}

impl Directive {
	pub fn to_topic(&self) -> Topic {
		match self {
			Self::AddChain(_) => Topic::AddChain,
			Self::AddToken(_) => Topic::AddToken,
			Self::ToggleChainState(_) => Topic::ToggleChainState,
			Self::UpdateFee(_) => Topic::UpdateFee,
			Self::UpdateChain(_) => Topic::UpdateChain,
			Self::UpdateToken(_) => Topic::UpdateToken,
		}
	}
}

impl core::fmt::Display for Directive {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Directive::AddChain(chain) => write!(f, "AddChain({})", chain),
			Directive::AddToken(token) => write!(f, "AddToken({})", token),
			Directive::ToggleChainState(toggle_state) => {
				write!(f, "ToggleChainState({})", toggle_state)
			}
			Directive::UpdateFee(factor) => write!(f, "UpdateFee({})", factor),
			Directive::UpdateChain(chain) => write!(f, "UpdateChain({})", chain),
			Directive::UpdateToken(token) => write!(f, "UpdateToken({})", token),
		}
	}
}
impl Directive {
	pub fn hash(&self) -> String {
		let mut hasher = sha2::Sha256::new();
		hasher.update(self.to_string().as_bytes());
		let bytes: [u8; 32] = hasher.finalize().into();
		bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
	}
}

#[derive(
	CandidType, Deserialize, Serialize, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash,
)]
pub struct DireKey {
	pub chain_id: ChainId,
	pub seq: Seq,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default)]
pub struct DireMap {
	pub dires: BTreeMap<Seq, Directive>,
}

impl DireMap {
	pub fn from(seq: Seq, dire: Directive) -> Self {
		Self {
			dires: BTreeMap::from([(seq, dire)]),
		}
	}
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Topic {
	AddChain,
	AddToken,
	UpdateChain,
	UpdateToken,
	ToggleChainState,
	UpdateFee,
}
impl core::fmt::Display for Topic {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Topic::AddChain => write!(f, "AddChain"),
			Topic::AddToken => write!(f, "AddToken"),
			Topic::ToggleChainState => {
				write!(f, "ToggleChainState",)
			}
			Topic::UpdateFee => write!(f, "UpdateFee"),
			Topic::UpdateChain => write!(f, "UpdateChain"),
			Topic::UpdateToken => write!(f, "UpdateToken"),
		}
	}
}

#[derive(
	CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum TicketType {
	#[default]
	Normal,
	Resubmit,
}

impl From<TicketType> for sea_orm_active_enums::TicketType {
	fn from(ticket_type: TicketType) -> Self {
		match ticket_type {
			TicketType::Normal => sea_orm_active_enums::TicketType::Normal,
			TicketType::Resubmit => sea_orm_active_enums::TicketType::Resubmit,
		}
	}
}
impl From<sea_orm_active_enums::TicketType> for TicketType {
	fn from(sea_ticket_type: sea_orm_active_enums::TicketType) -> Self {
		match sea_ticket_type {
			sea_orm_active_enums::TicketType::Normal => TicketType::Normal,
			sea_orm_active_enums::TicketType::Resubmit => TicketType::Resubmit,
		}
	}
}

#[derive(
	CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct OmnityTicket {
	pub ticket_id: TicketId,
	pub ticket_type: TicketType,
	pub ticket_time: Timestamp,
	pub src_chain: ChainId,
	pub dst_chain: ChainId,
	pub action: TxAction,
	pub token: TokenId,
	pub amount: String,
	pub sender: Option<Account>,
	pub receiver: Account,
	pub memo: Option<Vec<u8>>,
}
#[derive(
	CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum TicketStatus {
	#[default]
	Unknown,
	WaitingForConfirmBySrc,
	WaitingForConfirmByDest,
	Finalized,
}

impl From<TicketStatus> for sea_orm_active_enums::TicketStatus {
	fn from(status: TicketStatus) -> Self {
		match status {
			TicketStatus::Unknown => sea_orm_active_enums::TicketStatus::Unknown,
			TicketStatus::WaitingForConfirmBySrc => {
				sea_orm_active_enums::TicketStatus::WaitingForConfirmBySrc
			}
			TicketStatus::WaitingForConfirmByDest => {
				sea_orm_active_enums::TicketStatus::WaitingForConfirmByDest
			}
			TicketStatus::Finalized => sea_orm_active_enums::TicketStatus::Finalized,
		}
	}
}
impl From<sea_orm_active_enums::TicketStatus> for TicketStatus {
	fn from(status: sea_orm_active_enums::TicketStatus) -> Self {
		match status {
			sea_orm_active_enums::TicketStatus::Unknown => TicketStatus::Unknown,
			sea_orm_active_enums::TicketStatus::WaitingForConfirmBySrc => {
				TicketStatus::WaitingForConfirmBySrc
			}
			sea_orm_active_enums::TicketStatus::WaitingForConfirmByDest => {
				TicketStatus::WaitingForConfirmByDest
			}
			sea_orm_active_enums::TicketStatus::Finalized => TicketStatus::Finalized,
		}
	}
}

#[derive(
	CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Ticket {
	pub ticket_id: TicketId,
	pub ticket_seq: Option<u64>,
	pub ticket_type: TicketType,
	pub ticket_time: Timestamp,
	pub src_chain: ChainId,
	pub dst_chain: ChainId,
	pub action: TxAction,
	pub token: TokenId,
	pub amount: String,
	pub sender: Option<Account>,
	pub receiver: Account,
	pub memo: Option<Vec<u8>>,
	pub status: TicketStatus,
}

impl Ticket {
	pub fn new(
		ticket_id: TicketId,
		ticket_seq: Option<u64>,
		ticket_type: TicketType,
		ticket_time: Timestamp,
		src_chain: ChainId,
		dst_chain: ChainId,
		action: TxAction,
		token: TokenId,
		amount: String,
		sender: Option<Account>,
		receiver: Account,
		memo: Option<Vec<u8>>,
		status: TicketStatus,
	) -> Self {
		Self {
			ticket_id,
			ticket_seq,
			ticket_type,
			ticket_time,
			src_chain,
			dst_chain,
			action,
			token,
			amount,
			sender,
			receiver,
			memo,
			status,
		}
	}

	pub fn from_omnity_ticket(seq: u64, omnity_ticket: OmnityTicket) -> Self {
		Self {
			ticket_id: omnity_ticket.ticket_id.to_owned(),
			ticket_seq: Some(seq),
			ticket_type: omnity_ticket.ticket_type.to_owned(),
			ticket_time: omnity_ticket.ticket_time,
			src_chain: omnity_ticket.src_chain.to_owned(),
			dst_chain: omnity_ticket.dst_chain.to_owned(),
			action: omnity_ticket.action.to_owned(),
			token: omnity_ticket.token.to_owned(),
			amount: omnity_ticket.amount.to_owned(),
			sender: omnity_ticket.sender.to_owned(),
			receiver: omnity_ticket.receiver.to_owned(),
			memo: omnity_ticket.memo.to_owned(),
			status: TicketStatus::WaitingForConfirmByDest,
		}
	}
}

impl From<Ticket> for ticket::Model {
	fn from(ticket: Ticket) -> Self {
		ticket::Model {
			ticket_id: ticket.ticket_id,
			ticket_seq: ticket.ticket_seq.map(|seq| seq as i64),
			ticket_type: ticket.ticket_type.into(),
			ticket_time: ticket.ticket_time as i64,
			src_chain: ticket.src_chain,
			dst_chain: ticket.dst_chain,
			action: ticket.action.into(),
			token: ticket.token,
			amount: ticket.amount,
			sender: ticket.sender,
			receiver: ticket.receiver,
			memo: ticket.memo,
			status: ticket.status.into(),
		}
	}
}

impl From<ticket::Model> for Ticket {
	fn from(model: ticket::Model) -> Self {
		Ticket {
			ticket_id: model.ticket_id,
			ticket_seq: model.ticket_seq.map(|seq| seq as u64),
			ticket_type: model.ticket_type.into(),
			ticket_time: model.ticket_time as u64,
			src_chain: model.src_chain,
			dst_chain: model.dst_chain,
			action: model.action.into(),
			token: model.token,
			amount: model.amount,
			sender: model.sender,
			receiver: model.receiver,
			memo: model.memo,
			status: model.status.into(),
		}
	}
}

impl core::fmt::Display for Ticket {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
            f,
            "\nticket id:{} \nticket seq:{:?} \nticket type:{:?} \ncreated time:{} \nsrc chain:{} \ndst_chain:{} \naction:{:?} \ntoken:{} \namount:{} \nsender:{:?} \nrecevier:{} \nmemo:{:?} \nstatus:{:?}",
            self.ticket_id,
            self.ticket_seq,
            self.ticket_type,
            self.ticket_time,
            self.src_chain,
            self.dst_chain,
            self.action,
            self.token,
            self.amount,
            self.sender,
            self.receiver,
            self.memo,
            self.status,
        )
	}
}

#[derive(
	CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum ChainType {
	#[default]
	SettlementChain,
	ExecutionChain,
}

impl From<ChainType> for sea_orm_active_enums::ChainType {
	fn from(chain_type: ChainType) -> Self {
		match chain_type {
			ChainType::SettlementChain => sea_orm_active_enums::ChainType::SettlementChain,
			ChainType::ExecutionChain => sea_orm_active_enums::ChainType::ExecutionChain,
		}
	}
}
impl From<sea_orm_active_enums::ChainType> for ChainType {
	fn from(sea_chain_type: sea_orm_active_enums::ChainType) -> Self {
		match sea_chain_type {
			sea_orm_active_enums::ChainType::ExecutionChain => ChainType::ExecutionChain,
			sea_orm_active_enums::ChainType::SettlementChain => ChainType::SettlementChain,
		}
	}
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChainState {
	#[default]
	Active,
	Deactive,
}

impl From<ChainState> for sea_orm_active_enums::ChainState {
	fn from(chain_state: ChainState) -> Self {
		match chain_state {
			ChainState::Active => sea_orm_active_enums::ChainState::Active,
			ChainState::Deactive => sea_orm_active_enums::ChainState::Deactive,
		}
	}
}
impl From<sea_orm_active_enums::ChainState> for ChainState {
	fn from(sea_chain_state: sea_orm_active_enums::ChainState) -> Self {
		match sea_chain_state {
			sea_orm_active_enums::ChainState::Active => ChainState::Active,
			sea_orm_active_enums::ChainState::Deactive => ChainState::Deactive,
		}
	}
}

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq)]
pub enum ToggleAction {
	#[default]
	Activate,
	Deactivate,
}

impl From<ToggleAction> for ChainState {
	fn from(value: ToggleAction) -> Self {
		match value {
			ToggleAction::Activate => ChainState::Active,
			ToggleAction::Deactivate => ChainState::Deactive,
		}
	}
}

#[derive(
	CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum TxAction {
	#[default]
	Transfer,
	Redeem,
	Burn,
}

impl From<TxAction> for sea_orm_active_enums::TxAction {
	fn from(tx_action: TxAction) -> Self {
		match tx_action {
			TxAction::Transfer => sea_orm_active_enums::TxAction::Transfer,
			TxAction::Redeem => sea_orm_active_enums::TxAction::Redeem,
			TxAction::Burn => sea_orm_active_enums::TxAction::Burn,
		}
	}
}
impl From<sea_orm_active_enums::TxAction> for TxAction {
	fn from(sea_tx_action: sea_orm_active_enums::TxAction) -> Self {
		match sea_tx_action {
			sea_orm_active_enums::TxAction::Transfer => TxAction::Transfer,
			sea_orm_active_enums::TxAction::Redeem => TxAction::Redeem,
			sea_orm_active_enums::TxAction::Burn => TxAction::Burn,
		}
	}
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum Factor {
	UpdateTargetChainFactor(TargetChainFactor),
	UpdateFeeTokenFactor(FeeTokenFactor),
}

impl core::fmt::Display for Factor {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		match self {
			Factor::UpdateTargetChainFactor(chain_factor) => write!(f, "{}", chain_factor),
			Factor::UpdateFeeTokenFactor(token_factor) => write!(f, "{}", token_factor),
		}
	}
}
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct TargetChainFactor {
	pub target_chain_id: ChainId,
	pub target_chain_factor: u128,
}

impl core::fmt::Display for TargetChainFactor {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
			f,
			"\nchain id:{},\nchain factor:{}",
			self.target_chain_id, self.target_chain_factor,
		)
	}
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct FeeTokenFactor {
	pub fee_token: TokenId,
	pub fee_token_factor: u128,
}

impl core::fmt::Display for FeeTokenFactor {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
			f,
			"\nfee token:{},\nfee_token_factor:{}",
			self.fee_token, self.fee_token_factor,
		)
	}
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chain {
	pub chain_id: ChainId,
	pub canister_id: String,
	pub chain_type: ChainType,
	pub chain_state: ChainState,
	pub contract_address: Option<String>,
	pub counterparties: Option<Vec<ChainId>>,
	pub fee_token: Option<TokenId>,
}

impl Chain {
	pub fn chain_name(&self) -> Option<&str> {
		match self.chain_type {
			ChainType::SettlementChain => Some(&self.chain_id),
			ChainType::ExecutionChain => self.chain_id.split('-').last(),
		}
	}
}

impl From<Chain> for chain_meta::Model {
	fn from(chain: Chain) -> Self {
		chain_meta::Model {
			chain_id: chain.chain_id,
			canister_id: chain.canister_id,
			chain_type: chain.chain_type.into(),
			chain_state: chain.chain_state.into(),
			contract_address: chain.contract_address,
			counterparties: chain.counterparties.map(|cs| json!(cs)),
			fee_token: chain.fee_token,
		}
	}
}

impl From<chain_meta::Model> for Chain {
	fn from(model: chain_meta::Model) -> Self {
		Chain {
			chain_id: model.chain_id,
			canister_id: model.canister_id,
			chain_type: model.chain_type.into(),
			chain_state: model.chain_state.into(),
			contract_address: model.contract_address,
			counterparties: model
				.counterparties
				.map(|cs| serde_json::from_value(cs).expect("Failed to parse counterparties")),
			fee_token: model.fee_token,
		}
	}
}

impl core::fmt::Display for Chain {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
            f,
            "\nchain id:{} \ncanister id:{} \nchain type:{:?} \nchain state:{:?} \ncontract address:{:?} \ncounterparties:{:?} \nfee_token:{:?}",
            self.chain_id,self.canister_id, self.chain_type, self.chain_state, self.contract_address,self.counterparties,self.fee_token,
        )
	}
}

//TODO: update chain and token info
#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct ToggleState {
	pub chain_id: ChainId,
	pub action: ToggleAction,
}

impl core::fmt::Display for ToggleState {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
			f,
			"\nchain:{},\nchain state:{:?}",
			self.chain_id, self.action,
		)
	}
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Token {
	pub token_id: TokenId,
	pub name: String,
	pub symbol: String,

	pub decimals: u8,
	pub icon: Option<String>,
	pub metadata: HashMap<String, String>,
}

impl Token {
	/// return (settlmentchain,token protocol, token symbol)
	pub fn token_id_info(&self) -> Vec<&str> {
		self.token_id.split('-').collect()
	}
}
impl core::fmt::Display for Token {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
			f,
			"\ttoken id:{} \ntoken name:{} \nsymbol:{:?} \ndecimals:{} \nicon:{:?} \nmetadata:{:?}",
			self.token_id, self.name, self.symbol, self.decimals, self.icon, self.metadata
		)
	}
}

#[derive(CandidType, Deserialize, Debug, Error)]
pub enum Error {
	#[error("The chain(`{0}`) already exists")]
	ChainAlreadyExisting(String),
	#[error("The token(`{0}`) already exists")]
	TokenAlreadyExisting(String),

	#[error("not supported proposal")]
	NotSupportedProposal,
	#[error("proposal error: (`{0}`)")]
	ProposalError(String),

	#[error("generate directive error for : (`{0}`)")]
	GenerateDirectiveError(String),

	#[error("the message is malformed and cannot be decoded error")]
	MalformedMessageBytes,
	#[error("unauthorized")]
	Unauthorized,
	#[error("The `{0}` is deactive")]
	DeactiveChain(String),
	#[error("The ticket id (`{0}`) already exists!")]
	AlreadyExistingTicketId(String),
	#[error("The resubmit ticket id must exist!")]
	ResubmitTicketIdMustExist,
	#[error("The resubmit ticket must same as the old ticket!")]
	ResubmitTicketMustSame,
	#[error("The resumit ticket sent too often")]
	ResubmitTicketSentTooOften,
	#[error("not found chain: (`{0}`)")]
	NotFoundChain(String),
	#[error("not found token: (`{0}`)")]
	NotFoundToken(String),
	#[error("not found account(`{0}`) token(`{1}`) on the chain(`{2}`")]
	NotFoundAccountToken(String, String, String),
	#[error("Not found this token(`{0}`) on chain(`{1}`) ")]
	NotFoundChainToken(String, String),
	#[error("Insufficient token (`{0}`) on chain (`{1}`) !)")]
	NotSufficientTokens(String, String),
	#[error("The ticket amount(`{0}`) parse error: `{1}`")]
	TicketAmountParseError(String, String),
	#[error("ecdsa_public_key failed : (`{0}`)")]
	EcdsaPublicKeyError(String),

	#[error("sign_with_ecdsa failed: (`{0}`)")]
	SighWithEcdsaError(String),
	#[error("custom error: (`{0}`)")]
	CustomError(String),
}
