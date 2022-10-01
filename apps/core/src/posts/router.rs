use super::controller::{get_detail, get_posts, seed_posts};
use actix_web::Scope;

pub fn post_scope() -> Scope {
    Scope::new("/post")
        .service(get_posts)
        .service(seed_posts)
        .service(get_detail)
}
