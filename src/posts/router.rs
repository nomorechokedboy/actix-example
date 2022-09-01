use super::controller::get_posts;
use actix_web::Scope;

pub fn post_scope() -> Scope {
    Scope::new("/post").service(get_posts)
}
