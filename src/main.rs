use actix_web::{
    get,
    http::header::ContentType,
    web::{self},
    App, HttpResponse, HttpServer, Responder, Result,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Response {
    title: String,
    img: String,
}
#[derive(Deserialize)]
struct Info {
    timecode: String,
}
#[get("/")]
async fn index() -> impl Responder {
    "yeah"
}
#[get("/webpage")]
async fn webpage() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body("<h1>Y</h1>")
}
#[get("/api/get")]
async fn get(info: web::Query<Info>) -> Result<impl Responder> {
    let response = Response {
        img: format!("placeholder for img in timecode {}", info.timecode.as_str()),
        title: format!(
            "placeholder for title in timecode {}",
            info.timecode.as_str()
        ),
    };

    Ok(web::Json(response))
}
#[get("/api/latest")]
async fn latest() -> Result<impl Responder> {
    let response = Response {
        img: "latest".to_owned(),
        title: "latest".to_owned(),
    };

    Ok(web::Json(response))
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(webpage)
            .service(get)
            .service(latest)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
// async fn
