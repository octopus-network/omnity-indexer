use ::entity::{chain_meta, chain_meta::Entity as ChainMeta};
use ::entity::{notes, notes::Entity as Note};
use ::entity::{ticket, ticket::Entity as Ticket};
use ::entity::{token_meta, token_meta::Entity as TokenMeta};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_note_by_id(db: &DbConn, id: i32) -> Result<Option<notes::Model>, DbErr> {
        Note::find_by_id(id).one(db).await
    }

    pub async fn get_all_notes(db: &DbConn) -> Result<Vec<notes::Model>, DbErr> {
        Note::find().all(db).await
    }

    /// If ok, returns (note models, num pages).
    pub async fn find_notes_in_page(
        db: &DbConn,
        page: u64,
        notes_per_page: u64,
    ) -> Result<(Vec<notes::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Note::find()
            .order_by_asc(notes::Column::Id)
            .paginate(db, notes_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated notes
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    pub async fn get_all_chains(db: &DbConn) -> Result<Vec<chain_meta::Model>, DbErr> {
        ChainMeta::find().all(db).await
    }
    pub async fn get_all_tokens(db: &DbConn) -> Result<Vec<token_meta::Model>, DbErr> {
        TokenMeta::find().all(db).await
    }
    pub async fn get_all_tickets(db: &DbConn) -> Result<Vec<ticket::Model>, DbErr> {
        Ticket::find().all(db).await
    }
    pub async fn get_latest_tickets(db: &DbConn) -> Result<Option<ticket::Model>, DbErr> {
        Ticket::find()
            .order_by_desc(ticket::Column::Seq)
            .one(db)
            .await
    }
}
