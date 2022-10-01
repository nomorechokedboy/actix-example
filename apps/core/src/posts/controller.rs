use crate::common::{app_state::AppState, base_paging_query::BasePagingQuery};
use actix_redis::{resp_array, Command, RespValue};
use actix_web::{
    get,
    web::{self, Query},
    HttpRequest, HttpResponse,
};
use entity::post;
use entity::post::Entity as Post;
use fake::{faker::lorem::en::Paragraph, Fake};
use migration::Condition;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};

#[get("/{id}")]
pub async fn get_detail(id: web::Path<i32>, app_state: web::Data<AppState>) -> HttpResponse {
    let id = id.into_inner();
    let db = &app_state.db_con;
    let redis = &app_state.redis;
    let cache_key = format!("post:detail:id={id}");

    let cache_res = redis
        .send(Command(resp_array!["get", cache_key.clone()]))
        .await
        .expect("Can't send command")
        .expect("Can't get result from redis");

    if let RespValue::BulkString(res) = cache_res {
        let post: post::Model = serde_json::from_str(
            &String::from_utf8(res).expect("Should convert from utf8 to string"),
        )
        .expect("Can't serialize cache string");

        return HttpResponse::Ok().json(post);
    }

    let post = Post::find_by_id(id).one(db).await;
    if let Ok(post) = post {
        let post = post.unwrap();
        let cache = serde_json::to_string(&post).expect("Should serialize post");
        redis
            .send(Command(resp_array!["setex", cache_key, "300", cache]))
            .await
            .expect("Should set command")
            .expect("Should return ok");

        return HttpResponse::Ok().json(post);
    }

    HttpResponse::Ok().json(format!("Error getting post with id: {id}"))
}

#[get("/seed")]
pub async fn seed_posts(app_state: web::Data<AppState>) -> HttpResponse {
    let mut posts: Vec<post::ActiveModel> = Vec::new();
    let db = &app_state.db_con;

    for _i in 0..30000 {
        posts.push(post::ActiveModel {
            text: Set(Paragraph(1..2).fake()),
            title: Set(Paragraph(1..2).fake()),
            ..Default::default()
        });
    }

    let res = Post::insert_many(posts).exec(db).await;
    match res {
        Ok(id) => HttpResponse::Ok().json(id.last_insert_id),
        Err(_) => HttpResponse::Ok().json("Server error"),
    }
}

#[get("/paging")]
pub async fn get_posts(
    _req: HttpRequest,
    query: Query<BasePagingQuery>,
    app_state: web::Data<AppState>,
) -> HttpResponse {
    let page = query.page.unwrap_or_default();
    let page_size = query.page_size.unwrap_or_else(|| 20);
    let search = query.search.as_deref().unwrap_or_default();
    let cache_key = format!("products:paging:page={page}:pageSize={page_size}:search={search}");

    let redis = &app_state.redis;
    let cache_res = redis
        .send(Command(resp_array!["get", cache_key.clone()]))
        .await;

    if let Ok(Ok(RespValue::BulkString(cache_string))) = cache_res {
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
        .fetch_page((page).try_into().unwrap())
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
