use crate::entity::sea_orm_active_enums::TicketStatus;
use crate::entity::{chain_meta, chain_meta::Entity as ChainMeta};

use crate::entity::{ticket, ticket::Entity as Ticket};
use crate::entity::{token_meta, token_meta::Entity as TokenMeta};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn get_all_chains(db: &DbConn) -> Result<Vec<chain_meta::Model>, DbErr> {
        ChainMeta::find().all(db).await
    }
    pub async fn get_all_tokens(db: &DbConn) -> Result<Vec<token_meta::Model>, DbErr> {
        TokenMeta::find().all(db).await
    }
    pub async fn get_ticket_by_id(
        db: &DbConn,
        ticket_id: String,
    ) -> Result<Option<ticket::Model>, DbErr> {
        Ticket::find_by_id(ticket_id).one(db).await
    }
    pub async fn get_all_tickets(db: &DbConn) -> Result<Vec<ticket::Model>, DbErr> {
        Ticket::find().all(db).await
    }
    pub async fn get_latest_ticket(db: &DbConn) -> Result<Option<ticket::Model>, DbErr> {
        Ticket::find()
            .filter(ticket::Column::TicketSeq.is_not_null())
            .order_by_desc(ticket::Column::TicketSeq)
            .one(db)
            .await
    }
    pub async fn get_unconfirmed_tickets(
        db: &DbConn,
        dest: String,
    ) -> Result<Vec<ticket::Model>, DbErr> {
        Ticket::find()
            .filter(
                Condition::all()
                    // The ticket is not finalized
                    .add(ticket::Column::Status.ne(TicketStatus::Finalized))
                    // The ticket's destination chain matches `dest`
                    .add(ticket::Column::DstChain.eq(dest)),
            )
            .all(db)
            .await
    }
}
