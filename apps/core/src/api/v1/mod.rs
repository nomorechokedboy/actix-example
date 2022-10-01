use crate::posts::router::post_scope;
use actix_web::Scope;

pub fn v1_scope() -> Scope {
    Scope::new("/api/v1").service(post_scope())
}
