use crate::common::{app_state::AppState, base_paging_query::BasePagingQuery};
use actix_web::{error, get, post, web, Error, HttpRequest, HttpResponse};
use entity::post;
use entity::post::Entity as Post;
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryOrder, Set};

const DEFAULT_POSTS_PER_PAGE: u64 = 20;

#[get("/")]
pub async fn list(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let conn = &data.db_con;

    // get params
    let params = web::Query::<BasePagingQuery>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.page_size.unwrap_or(DEFAULT_POSTS_PER_PAGE);
    let paginator = Post::find()
        .order_by_asc(post::Column::Id)
        .paginate(conn, posts_per_page.try_into().unwrap());
    let num_pages = paginator.num_pages().await.ok().unwrap();

    let posts = paginator
        .fetch_page((page - 1).try_into().unwrap())
        .await
        .expect("could not retrieve posts");
    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    let body = template
        .render("index.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[get("/new")]
pub async fn new(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let ctx = tera::Context::new();
    let body = template
        .render("new.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[post("/")]
pub async fn create(
    data: web::Data<AppState>,
    post_form: web::Form<post::Model>,
) -> Result<HttpResponse, Error> {
    let conn = &data.db_con;

    let form = post_form.into_inner();

    post::ActiveModel {
        title: Set(form.title.to_owned()),
        text: Set(form.text.to_owned()),
        ..Default::default()
    }
    .save(conn)
    .await
    .expect("could not insert post");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

#[get("/{id}")]
pub async fn edit(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.db_con;
    let template = &data.templates;

    let post: post::Model = Post::find_by_id(id.into_inner())
        .one(conn)
        .await
        .expect("could not find post")
        .unwrap();

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);

    let body = template
        .render("edit.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[post("/{id}")]
pub async fn update(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    post_form: web::Form<post::Model>,
) -> Result<HttpResponse, Error> {
    let conn = &data.db_con;
    let form = post_form.into_inner();

    post::ActiveModel {
        id: Set(id.into_inner()),
        title: Set(form.title.to_owned()),
        text: Set(form.text.to_owned()),
    }
    .save(conn)
    .await
    .expect("could not edit post");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

#[post("/delete/{id}")]
pub async fn delete(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.db_con;

    let post: post::ActiveModel = Post::find_by_id(id.into_inner())
        .one(conn)
        .await
        .unwrap()
        .unwrap()
        .into();

    post.delete(conn).await.unwrap();

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

pub async fn not_found(
    data: web::Data<AppState>,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("uri", request.uri().path());

    let template = &data.templates;
    let body = template
        .render("error/404.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
