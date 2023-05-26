use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let query = Table::alter()
            .table(User::Table)
            .add_column(
                ColumnDef::new(User::Birth)
                    .date_time()
                    .not_null()
                    .default(Expr::cust("NOW()")),
            )
            .to_owned();

        manager.exec_stmt(query).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        todo!();
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum User {
    Table,
    Birth,
}
