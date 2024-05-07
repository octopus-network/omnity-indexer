use async_graphql::{Context, Object, Result};
use entity::{async_graphql, notes};
use omnity_indexer_service::Query;

use crate::db::Database;

#[derive(Default)]
pub struct NoteQuery;

#[Object]
impl NoteQuery {
    async fn get_notes(&self, ctx: &Context<'_>) -> Result<Vec<notes::Model>> {
        let db = ctx.data::<Database>().unwrap();
        let conn = db.get_connection();

        Ok(Query::get_all_notes(conn)
            .await
            .map_err(|e| e.to_string())?)
    }

    async fn get_note_by_id(&self, ctx: &Context<'_>, id: i32) -> Result<Option<notes::Model>> {
        let db = ctx.data::<Database>().unwrap();
        let conn = db.get_connection();

        Ok(Query::find_note_by_id(conn, id)
            .await
            .map_err(|e| e.to_string())?)
    }
}
