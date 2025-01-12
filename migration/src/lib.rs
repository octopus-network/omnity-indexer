pub use sea_orm_migration::prelude::*;
mod m20240507_055143_one;
mod m20240701_000001_two;
mod m20240802_000001_three;
mod m20250111_000001_four;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(m20240507_055143_one::Migration),
			Box::new(m20240701_000001_two::Migration),
			Box::new(m20240802_000001_three::Migration),
			Box::new(m20250111_000001_four::Migration),
		]
	}
}
