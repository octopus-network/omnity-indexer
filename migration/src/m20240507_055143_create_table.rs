use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create enum
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("chain_type"))
                    .values(ChainType::iter())
                    .to_owned(),
            )
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("chain_state"))
                    .values(ChainState::iter())
                    .to_owned(),
            )
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("ticket_type"))
                    .values(TicketType::iter())
                    .to_owned(),
            )
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("tx_action"))
                    .values(TxAction::iter())
                    .to_owned(),
            )
            .await?;
        // Create ChainMeta table
        manager
            .create_table(
                Table::create()
                    .table(ChainMeta::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChainMeta::ChainId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChainMeta::CanisterId).text().not_null())
                    .col(
                        ColumnDef::new(ChainMeta::ChainType)
                            .not_null()
                            .enumeration(Alias::new("chain_type"), ChainType::iter()),
                    )
                    .col(
                        ColumnDef::new(ChainMeta::ChainState)
                            .not_null()
                            .enumeration(Alias::new("chain_state"), ChainState::iter()),
                    )
                    .col(ColumnDef::new(ChainMeta::ContractAddress).string().null())
                    .col(ColumnDef::new(ChainMeta::Counterparties).json().null())
                    .col(ColumnDef::new(ChainMeta::FeeToken).string().null())
                    .to_owned(),
            )
            .await?;

        // Create TokenMeta table
        manager
            .create_table(
                Table::create()
                    .table(TokenMeta::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TokenMeta::TokenId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TokenMeta::Name).string().not_null())
                    .col(ColumnDef::new(TokenMeta::Symbol).string().not_null())
                    .col(ColumnDef::new(TokenMeta::IssueChain).string().not_null())
                    .col(
                        ColumnDef::new(TokenMeta::Decimals)
                            .tiny_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TokenMeta::Icon).text().null())
                    .col(ColumnDef::new(TokenMeta::Metadata).json().null())
                    .col(ColumnDef::new(TokenMeta::DstChains).json().null())
                    .to_owned(),
            )
            .await?;

        // Create Ticket table
        manager
            .create_table(
                Table::create()
                    .table(Ticket::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Ticket::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Ticket::TicketId).text().not_null())
                    .col(
                        ColumnDef::new(Ticket::TicketType)
                            .not_null()
                            .enumeration(Alias::new("ticket_type"), TicketType::iter()),
                    )
                    .col(ColumnDef::new(Ticket::TicketTime).big_unsigned().not_null())
                    .col(ColumnDef::new(Ticket::SrcChain).string().not_null())
                    .col(ColumnDef::new(Ticket::DstChain).string().not_null())
                    .col(
                        ColumnDef::new(Ticket::Action)
                            .not_null()
                            .enumeration(Alias::new("tx_action"), TxAction::iter()),
                    )
                    .col(ColumnDef::new(Ticket::Token).string().not_null())
                    .col(ColumnDef::new(Ticket::Amount).string().not_null())
                    .col(ColumnDef::new(Ticket::Sender).string().null())
                    .col(ColumnDef::new(Ticket::Receiver).string().not_null())
                    .col(ColumnDef::new(Ticket::Memo).binary().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables
        manager
            .drop_table(Table::drop().table(Ticket::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TokenMeta::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ChainMeta::Table).to_owned())
            .await?;
        manager
            .drop_type(
                Type::drop()
                    .if_exists()
                    .names([
                        SeaRc::new(ChainType::Type) as DynIden,
                        SeaRc::new(ChainState::Type) as DynIden,
                        SeaRc::new(TicketType::Type) as DynIden,
                        SeaRc::new(TxAction::Type) as DynIden,
                    ])
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChainMeta {
    Table,
    ChainId,
    CanisterId,
    ChainType,
    ChainState,
    ContractAddress,
    Counterparties,
    FeeToken,
}

#[derive(DeriveIden)]
enum TokenMeta {
    Table,
    TokenId,
    Name,
    Symbol,
    IssueChain,
    Decimals,
    Icon,
    Metadata,
    DstChains,
}

#[derive(DeriveIden)]
enum Ticket {
    Table,
    Id,
    TicketId,
    TicketType,
    TicketTime,
    SrcChain,
    DstChain,
    Action,
    Token,
    Amount,
    Sender,
    Receiver,
    Memo,
}

#[derive(Iden, EnumIter)]
pub enum ChainType {
    #[iden = "chain_type"]
    Type,
    #[iden = "SettlementChain"]
    SettlementChain,
    #[iden = "ExecutionChain"]
    ExecutionChain,
}

#[derive(Iden, EnumIter)]
pub enum ChainState {
    #[iden = "chain_state"]
    Type,
    #[iden = "Active"]
    Active,
    #[iden = "Deactive"]
    Deactive,
}

#[derive(Iden, EnumIter)]
pub enum TicketType {
    #[iden = "ticket_type"]
    Type,
    #[iden = "Normal"]
    Normal,
    #[iden = "Resubmit"]
    Resubmit,
}

#[derive(Iden, EnumIter)]
pub enum TxAction {
    #[iden = "tx_action"]
    Type,
    #[iden = "Transfer"]
    Transfer,
    #[iden = "Redeem"]
    Redeem,
}
