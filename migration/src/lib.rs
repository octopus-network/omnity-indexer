pub use sea_orm_migration::prelude::*;
mod m20240507_055143_one;
mod m20240606_000001_two;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(m20240507_055143_one::Migration),
			Box::new(m20240606_000001_two::Migration),
		]
	}
}
