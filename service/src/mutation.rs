use ::entity::chain_meta;
use ::entity::chain_meta::Entity as ChainMeta;
use ::entity::ticket;
use ::entity::ticket::Entity as Ticket;
use ::entity::token_meta;
use ::entity::token_meta::Entity as TokenMeta;
use ::entity::{notes, notes::Entity as Note};

use sea_orm::sea_query::OnConflict;
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_note(db: &DbConn, form_data: notes::Model) -> Result<notes::Model, DbErr> {
        let active_model = notes::ActiveModel {
            title: Set(form_data.title.to_owned()),
            text: Set(form_data.text.to_owned()),
            ..Default::default()
        };
        let res = Note::insert(active_model).exec(db).await?;

        Ok(notes::Model {
            id: res.last_insert_id,
            ..form_data
        })
    }

    pub async fn update_note_by_id(
        db: &DbConn,
        id: i32,
        form_data: notes::Model,
    ) -> Result<notes::Model, DbErr> {
        let note: notes::ActiveModel = Note::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find note.".to_owned()))
            .map(Into::into)?;

        notes::ActiveModel {
            id: note.id,
            title: Set(form_data.title.to_owned()),
            text: Set(form_data.text.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_note(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let note: notes::ActiveModel = Note::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find note.".to_owned()))
            .map(Into::into)?;

        note.delete(db).await
    }

    pub async fn delete_all_notes(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Note::delete_many().exec(db).await
    }
    pub async fn save_chain(
        db: &DbConn,
        form_data: chain_meta::Model,
    ) -> Result<chain_meta::Model, DbErr> {
        let active_model = chain_meta::ActiveModel {
            chain_id: Set(form_data.chain_id.to_owned()),
            canister_id: Set(form_data.canister_id.to_owned()),
            chain_type: Set(form_data.chain_type.to_owned()),
            chain_state: Set(form_data.chain_state.to_owned()),
            contract_address: Set(form_data.contract_address.to_owned()),
            counterparties: Set(form_data.counterparties.to_owned()),
            fee_token: Set(form_data.fee_token.to_owned()),
        };
        // let res = ChainMeta::insert(active_model).exec(db).await?;
        // let res = active_model.save(db).await?;
        let on_conflict = OnConflict::column(chain_meta::Column::ChainId)
            .do_nothing()
            .to_owned();
        let insert_result = ChainMeta::insert(active_model.clone())
            .on_conflict(on_conflict)
            .exec(db)
            .await;
        match insert_result {
            Ok(ret) => {
                println!("insert chain result : {:?}", ret);
            }
            Err(e) => {
                println!("insert chain error: {:?}, need to update chain", e);
                let res = ChainMeta::update(active_model)
                    .filter(chain_meta::Column::ChainId.eq(form_data.chain_id.to_owned()))
                    .exec(db)
                    .await
                    .map(|chain| chain);
                println!("update chain result : {:?}", res);
            }
        }
        Ok(chain_meta::Model { ..form_data })
    }

    pub async fn save_token(
        db: &DbConn,
        form_data: token_meta::Model,
    ) -> Result<token_meta::Model, DbErr> {
        let active_model = token_meta::ActiveModel {
            token_id: Set(form_data.token_id.to_owned()),
            name: Set(form_data.name.to_owned()),
            symbol: Set(form_data.symbol.to_owned()),
            issue_chain: Set(form_data.issue_chain.to_owned()),
            decimals: Set(form_data.decimals.to_owned()),
            icon: Set(form_data.icon.to_owned()),
            metadata: Set(form_data.metadata.to_owned()),
            dst_chains: Set(form_data.dst_chains.to_owned()),
        };
        // let res = TokenMeta::insert(active_model).exec(db).await?;
        // let res = active_model.save(db).await?;
        let on_conflict = OnConflict::column(token_meta::Column::TokenId)
            .do_nothing()
            .to_owned();
        let insert_result = TokenMeta::insert(active_model.clone())
            .on_conflict(on_conflict)
            .exec(db)
            .await;
        match insert_result {
            Ok(ret) => {
                println!("insert token result : {:?}", ret);
            }
            Err(e) => {
                println!("insert token error: {:?}, need to update token", e);
                let res = TokenMeta::update(active_model)
                    .filter(token_meta::Column::TokenId.eq(form_data.token_id.to_owned()))
                    .exec(db)
                    .await
                    .map(|token| token);
                println!("update token result : {:?}", res);
            }
        }

        Ok(token_meta::Model { ..form_data })
    }
    pub async fn save_ticket(
        db: &DbConn,
        form_data: ticket::Model,
    ) -> Result<ticket::Model, DbErr> {
        let active_model = ticket::ActiveModel {
            ticket_id: Set(form_data.ticket_id.to_owned()),
            ticket_seq: Set(form_data.ticket_seq.to_owned()),
            ticket_type: Set(form_data.ticket_type.to_owned()),
            ticket_time: Set(form_data.ticket_time.to_owned()),
            src_chain: Set(form_data.src_chain.to_owned()),
            dst_chain: Set(form_data.dst_chain.to_owned()),
            action: Set(form_data.action.to_owned()),
            token: Set(form_data.token.to_owned()),
            amount: Set(form_data.amount.to_owned()),
            sender: Set(form_data.sender.to_owned()),
            receiver: Set(form_data.receiver.to_owned()),
            memo: Set(form_data.memo.to_owned()),
        };
        let on_conflict = OnConflict::column(ticket::Column::TicketId)
            .do_nothing()
            .to_owned();
        let insert_result = Ticket::insert(active_model.clone())
            .on_conflict(on_conflict)
            .exec(db)
            .await;
        match insert_result {
            Ok(ret) => {
                println!("insert ticket result : {:?}", ret);
            }
            Err(e) => {
                println!("insert ticket error: {:?}, need to update ticket", e);
                let res = Ticket::update(active_model)
                    .filter(ticket::Column::TicketId.eq(form_data.ticket_id.to_owned()))
                    .exec(db)
                    .await
                    .map(|ticket| ticket);
                println!("update ticket result : {:?}", res);
            }
        }

        Ok(ticket::Model { ..form_data })
    }
}
