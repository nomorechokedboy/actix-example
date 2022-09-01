use crate::common::app_state::AppState;
use actix_redis::{resp_array, Command, RespValue};
use actix_web::{get, web, HttpRequest, HttpResponse};
use entity::post;
use entity::post::Entity as Post;
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};

#[get("/paging")]
pub async fn get_posts(_req: HttpRequest, app_state: web::Data<AppState>) -> HttpResponse {
    let page = 1;
    // let page = params.page.unwrap_or(1);
    let page_size = 20;
    let cache_key = format!("products:paging:page={page}:pageSize={page_size}");

    let redis = &app_state.redis;
    let cache_res = redis
        .send(Command(resp_array!["get", cache_key.clone()]))
        .await
        .expect("Error getting post cache")
        .expect(format!("Can't get posts cache").as_str());

    match cache_res {
        RespValue::Nil => {
            let db_con = &app_state.db_con;
            let paginator = Post::find()
                .order_by_asc(post::Column::Id)
                .paginate(db_con, page_size.try_into().unwrap());

            let posts = paginator
                .fetch_page((page - 1).try_into().unwrap())
                .await
                .expect("could not retrieve posts");
            let cache = serde_json::to_string(&posts).unwrap();
            redis
                .send(Command(resp_array!["setex", cache_key, "300", cache]))
                .await
                .expect("Error setting posts cache")
                .expect("Can't get cache set result");

            HttpResponse::Ok().json(posts)
        }

        RespValue::BulkString(cache) => {
            let posts: Vec<post::Model> =
                serde_json::from_str(&String::from_utf8(cache).expect("Invalid UTF-8!")).unwrap();

            HttpResponse::Ok().json(posts)
        }

        _ => HttpResponse::InternalServerError().json("Internal server error"),
    }
}
