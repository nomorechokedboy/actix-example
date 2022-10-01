use actix::Addr;
use actix_redis::RedisActor;
use sea_orm::DatabaseConnection;

#[derive(Debug, Clone)]
pub struct AppState {
    pub templates: tera::Tera,
    pub db_con: DatabaseConnection,
    pub redis: Addr<RedisActor>,
}
