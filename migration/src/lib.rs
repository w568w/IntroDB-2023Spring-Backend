pub use sea_orm_migration::prelude::*;

mod m20230511_164244_create_table;
mod m20230526_035013_add_birth;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230511_164244_create_table::Migration),
            Box::new(m20230526_035013_add_birth::Migration),
        ]
    }
}
