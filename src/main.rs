use std::
    fs::exists
;

use actix_web::{
    get, http::header::ContentType, web::{self}, App, HttpRequest, HttpResponse, HttpServer, Responder, Result
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// Response used for returning comics. 
#[derive(Serialize)]
struct Response {
    id: i64,
    title: String,
    img: String,
}
// Parameter struct for /comic?get
#[derive(Deserialize)]
struct Info {
    id: i64,
}
#[get("/")]
async fn index() -> impl Responder {
    "APIs."
}
#[get("/webpage")]
async fn webpage() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body("<h1>Y</h1>")
}
#[get("/api/get")]
async fn get(info: web::Query<Info>) -> Result<impl Responder> {
    let conn = initdb().unwrap();
    let mut response = Response {
        id: 0,
        img: String::new(),
        title: String::new(),
    };
    let mut respponsedbtitle = conn
        .prepare(format!("select * from entries where id={}", info.id).as_str())
        .unwrap();
    match respponsedbtitle.query_row([], |row| {
        response.id = row.get(0).unwrap();
        response.title = row.get(1).unwrap();
        response.img = row.get(2).unwrap();
        Ok(())
    }) {
        Ok(_) => (),
        Err(_) => {
            return Ok(web::Json(Response {
                id: 0,
                img: "dne".to_owned(),
                title: "dne".to_owned(),
            }).customize().append_header(("access-control-allow-origin", "*")));
        }
    };

    Ok(web::Json(response).customize().append_header(("access-control-allow-origin", "*")))
}
#[derive(Deserialize)]
struct EhentaiIntake{
    gid: i64, 
    token: String, 
}
#[get("/api/ehentaiproxy")]
async fn ehentai(params: web::Query<EhentaiIntake>) -> HttpResponse{
    let gid = params.gid; 
    let token = &params.token;
    let proxy = reqwest::Proxy::http("http://127.0.0.1:7890").unwrap();
    let client = reqwest::Client::new();
    let res = client.post("https://e-hentai.org/api.php")
    .body(format!("{{\"method\": \"gdata\",\"gidlist\": [[{},\"{}\"]],\"namespace\": 1}}", gid,token))
    .header("Content-Type", "application/json")
    .send()
    .await.unwrap().text().await.unwrap();
    println!("{}",res);
    HttpResponse::Ok().content_type(ContentType::plaintext()).insert_header(("access-control-allow-origin", "*")).insert_header(("Content-Type", "application/json")).body(res)


}

#[get("/api/latest")]
async fn latest() -> Result<impl Responder> {
    let mut response = Response {
        id: 0, 
        img: String::new(),
        title: String::new(),
    };
    let conn = initdb().unwrap();
    let mut respponsedb = conn.prepare("select max(id) from entries").unwrap();
    let mut latest: i64 = i64::default();
    respponsedb
        .query_row([], |row| {
            latest = row.get(0).unwrap();
            Ok(())
        })
        .unwrap();
    let mut respponsedbtitle = conn
        .prepare(format!("select * from entries where id={}", latest).as_str())
        .unwrap();
    respponsedbtitle
        .query_row([], |row| {
            response.id = latest;
            response.title = row.get(1).unwrap();
            response.img = row.get(2).unwrap();
            Ok(())
        })
        .unwrap();

    Ok(web::Json(response).customize().append_header(("Access-Control-Allow-Origin", "*")).append_header(("Access-Control-Allow-Methods","POST, GET")).append_header(("X-Content-Type-Options", "nosniff")))
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(webpage)
            .service(get)
            .service(latest)
            .service(ehentai)
            
    })
    .bind(("0.0.0.0", 5766))?
    .run()
    .await
}

fn initdb() -> Result<Connection> {
    match exists("./data.db").unwrap() {
        true => {
            let conn = Connection::open("./data.db").unwrap();
            return Ok(conn);
        }
        false => {
            let conn = Connection::open("./data.db").unwrap();
            conn.execute(
                "create table entries(id integer primary key autoincrement , title text, img text)",
                (),
            )
            .unwrap();
            return Ok(conn);
        }
    }
}
