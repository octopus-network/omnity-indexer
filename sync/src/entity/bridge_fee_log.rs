//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "bridge_fee_log")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub chain_id: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub date: String,
	pub fee_token_id: String,
	pub amount: String,
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
}

impl Related<super::chain_meta::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ChainMeta.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
