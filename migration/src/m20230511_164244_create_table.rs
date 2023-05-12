use entity::transaction;
use sea_orm_migration::{prelude::*, sea_orm::TransactionTrait};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Book {
    Table,
    Isbn,
    Title,
    Author,
    Publisher,
    OutPrice,
    InventoryCount,
    OnShelfCount,
}

#[derive(Iden)]
enum OrderList {
    Table,
    Id,
    TotalPrice,
    TotalCount,
    Status,
    CreatedAt,
    UpdatedAt,
    BookIsbn,
    OperatorId,
}

#[derive(Iden)]
enum Transaction {
    Table,
    Id,
    CreatedAt,
    TotalPrice,
    TicketId,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
    PasswordSalt,
    SecretKey,
    Role,
    RealName,
    Sex,
}

enum TicketStatus {
    EnumName,
    Pending,
    StockPaid,
    Done,
    Revoked,
}

impl Iden for TicketStatus {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                TicketStatus::EnumName => "ticket_status",
                TicketStatus::Pending => "Pending",
                TicketStatus::StockPaid => "StockPaid",
                TicketStatus::Done => "Done",
                TicketStatus::Revoked => "Revoked",
            }
        )
        .expect("Unable to write iden")
    }
}

impl IntoIterator for TicketStatus {
    type Item = Self;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            TicketStatus::Pending,
            TicketStatus::StockPaid,
            TicketStatus::Done,
            TicketStatus::Revoked,
        ]
        .into_iter()
    }
}

enum Sex {
    EnumName,
    Male,
    Female,
    NonBinary,
}

impl Iden for Sex {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Sex::EnumName => "sex",
                Sex::Male => "Male",
                Sex::Female => "Female",
                Sex::NonBinary => "NonBinary",
            }
        )
        .expect("Unable to write iden")
    }
}

impl IntoIterator for Sex {
    type Item = Self;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![Sex::Male, Sex::Female, Sex::NonBinary].into_iter()
    }
}

fn book() -> TableCreateStatement {
    Table::create()
        .table(Book::Table)
        .col(ColumnDef::new(Book::Isbn).string().not_null().primary_key())
        .col(ColumnDef::new(Book::Title).string().not_null())
        .col(ColumnDef::new(Book::Author).string().not_null())
        .col(ColumnDef::new(Book::Publisher).string().not_null())
        .col(ColumnDef::new(Book::OutPrice).float().not_null())
        .col(ColumnDef::new(Book::InventoryCount).integer().not_null())
        .col(ColumnDef::new(Book::OnShelfCount).integer().not_null())
        .to_owned()
}

fn order_list() -> TableCreateStatement {
    Table::create()
        .table(OrderList::Table)
        .col(
            ColumnDef::new(OrderList::Id)
                .integer()
                .not_null()
                .primary_key().auto_increment(),
        )
        .col(ColumnDef::new(OrderList::TotalPrice).float().not_null())
        .col(ColumnDef::new(OrderList::TotalCount).integer().not_null())
        .col(
            ColumnDef::new(OrderList::Status)
                .enumeration(TicketStatus::EnumName, TicketStatus::EnumName)
                .not_null(),
        )
        .col(ColumnDef::new(OrderList::CreatedAt).date_time().not_null())
        .col(ColumnDef::new(OrderList::UpdatedAt).date_time().not_null())
        .col(ColumnDef::new(OrderList::BookIsbn).string().not_null())
        .col(ColumnDef::new(OrderList::OperatorId).integer().not_null())
        .to_owned()
}

fn transaction() -> TableCreateStatement {
    Table::create()
        .table(Transaction::Table)
        .col(
            ColumnDef::new(Transaction::Id)
                .big_integer()
                .not_null()
                .primary_key().auto_increment(),
        )
        .col(
            ColumnDef::new(Transaction::CreatedAt)
                .date_time()
                .not_null(),
        )
        .col(ColumnDef::new(Transaction::TotalPrice).float().not_null())
        .col(ColumnDef::new(Transaction::TicketId).integer().not_null())
        .to_owned()
}

fn user() -> TableCreateStatement {
    Table::create()
        .table(User::Table)
        .col(ColumnDef::new(User::Id).integer().not_null().primary_key().auto_increment())
        .col(ColumnDef::new(User::PasswordSalt).string().not_null())
        .col(ColumnDef::new(User::SecretKey).string().not_null())
        .col(ColumnDef::new(User::Role).string().not_null())
        .col(ColumnDef::new(User::RealName).string().not_null())
        .col(
            ColumnDef::new(User::Sex)
                .enumeration(Sex::EnumName, Sex::EnumName)
                .not_null(),
        )
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        let ts = db.begin().await?;
        for mut statement in [book(), order_list(), transaction(), user()] {
            ts.execute(backend.build(statement.if_not_exists())).await?;
        }
        ts.commit().await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Book::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(OrderList::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Transaction::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        Ok(())
    }
}
