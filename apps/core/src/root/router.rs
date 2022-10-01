use super::controller::*;
use actix_web::Scope;

pub fn root_scope() -> Scope {
    Scope::new("")
        .service(list)
        .service(new)
        .service(create)
        .service(edit)
        .service(update)
        .service(delete)
}
