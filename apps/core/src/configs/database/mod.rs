use migration::{DbErr, Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

const DB_NAME: &str = "actix_example";

pub async fn connect_db(db_url: String) -> Result<DatabaseConnection, DbErr> {
    // let db_con = sea_orm::Database::connect(db_url).await.unwrap();

    // db_con
    let db = Database::connect(&db_url).await?;
    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", DB_NAME),
            ))
            .await?;

            let url = format!("{}/{}", db_url, DB_NAME);
            Database::connect(&url).await?
        }

        DbBackend::Postgres => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF EXISTS \"{}\";", DB_NAME),
            ))
            .await?;
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE \"{}\";", DB_NAME),
            ))
            .await?;
            let url = format!("{}/{}", db_url, DB_NAME);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    Migrator::up(&db, None).await.unwrap();
    Ok(db)
}
