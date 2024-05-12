//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use super::sea_orm_active_enums::TicketType;
use super::sea_orm_active_enums::TxAction;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "ticket")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub ticket_id: String,
    pub ticket_seq: Option<i64>,
    pub ticket_type: TicketType,
    pub ticket_time: i64,
    pub src_chain: String,
    pub dst_chain: String,
    pub action: TxAction,
    pub token: String,
    pub amount: String,
    pub sender: Option<String>,
    pub receiver: String,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub memo: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
