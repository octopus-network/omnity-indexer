//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "token_ledger_id_on_chain")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub chain_id: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub token_id: String,
	pub contract_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::chain_meta::Entity",
		from = "Column::ChainId",
		to = "super::chain_meta::Column::ChainId",
		on_update = "NoAction",
		on_delete = "NoAction"
	)]
	ChainMeta,
	#[sea_orm(
		belongs_to = "super::token_meta::Entity",
		from = "Column::TokenId",
		to = "super::token_meta::Column::TokenId",
		on_update = "NoAction",
		on_delete = "NoAction"
	)]
	TokenMeta,
}

impl Related<super::chain_meta::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ChainMeta.def()
	}
}

impl Related<super::token_meta::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::TokenMeta.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
