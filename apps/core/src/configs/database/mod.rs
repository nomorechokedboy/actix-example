use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;

pub async fn connect_db(db_url: String) -> DatabaseConnection {
    println!("DBURI: {db_url}");
    let db_con = sea_orm::Database::connect(db_url).await.unwrap();
    Migrator::up(&db_con, None).await.unwrap();

    db_con
}
