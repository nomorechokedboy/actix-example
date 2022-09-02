use crate::common::{app_state::AppState, base_paging_query::BasePagingQuery};
use actix_redis::{resp_array, Command, RespValue};
use actix_web::{
    get,
    web::{self, Query},
    HttpRequest, HttpResponse,
};
use entity::post;
use entity::post::Entity as Post;
use migration::Condition;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

#[get("/paging")]
pub async fn get_posts(
    _req: HttpRequest,
    query: Query<BasePagingQuery>,
    app_state: web::Data<AppState>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let search = query.search.clone().unwrap_or("".to_string());
    let cache_key = format!("products:paging:page={page}:pageSize={page_size}:search={search}");

    let redis = &app_state.redis;
    let cache_res = redis
        .send(Command(resp_array!["get", cache_key.clone()]))
        .await
        .expect("Error getting post cache")
        .expect(format!("Can't get posts cache").as_str());

    if let RespValue::BulkString(cache_string) = cache_res {
        let posts: Vec<post::Model> =
            serde_json::from_str(&String::from_utf8(cache_string).expect("Invalid UTF-8!"))
                .unwrap();

        return HttpResponse::Ok().json(posts);
    }

    let db_con = &app_state.db_con;
    let paginator = Post::find()
        .filter(
            Condition::any()
                .add(post::Column::Title.like(format!("%{}%", &search).as_str()))
                .add(post::Column::Text.like(format!("%{}%", &search).as_str())),
        )
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
