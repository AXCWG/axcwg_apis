use actix_web::{
    get,
    http::header::ContentType,
    options, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder, Result,
};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir, exists, rename, File, OpenOptions},
    io::Write,
};

fn log(info: &str) {
    let data = format!("[{}] {info}", chrono::offset::Utc::now());
    match exists("./logs/latest.log").unwrap(){
        true=>(),
        false=>{File::create("./logs/latest.log").unwrap();},
    }
    let mut data_file = OpenOptions::new()
        .append(true)
        .open("./logs/latest.log")
        .unwrap();
    data_file.write(format!("{}\n", data).as_bytes()).unwrap();
    println!("{}", data);
}

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

// Reverse API for images. Was originally on the off-site VPS but going to abandon.
#[get("/ehimages/{url:.*}")]
async fn ehimages(path: web::Path<(String,)>) -> HttpResponse {
    let (url,) = path.into_inner();
    let proxy = reqwest::Proxy::all("http://localhost:7890").unwrap();
    let client = reqwest::Client::builder().proxy(proxy).build().unwrap();
    let url = format!("https://ehgt.org/{url}");
    log(url.clone().as_str());
    let res = client.get(url).send().await.unwrap().bytes().await.unwrap();
    HttpResponse::Ok().body(res)
}
// The API for getting comics
#[get("/api/get")]
async fn get(info: web::Query<Info>) -> Result<impl Responder> {
    let conn = initdb().unwrap();
    // init
    let mut response = Response {
        id: 0,
        img: String::new(),
        title: String::new(),
    };
    // query
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
            // return special value
            return Ok(web::Json(Response {
                id: 0,
                img: "dne".to_owned(),
                title: "dne".to_owned(),
            })
            .customize()
            .insert_header(("Access-Control-Allow-Origin", "*")));
        }
    };
    log(format!("Now calling {}...", response.title).as_str());
    // return result
    Ok(web::Json(response)
        .customize()
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Methods", "GET, POST"))
        .insert_header(("X-Content-Type-Options", "nosniff")))
}
// params for get proxy.
#[derive(Deserialize)]
struct EhentaiIntake {
    gid: i64,
    token: String,
}
#[options("/api/ehentaiproxyget")]
async fn ehentaipreflight() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .insert_header(("Access-Control-Request-Headers", "*"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Methods", "GET, POST"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .body(String::new())
}
#[get("/api/ehentaiproxyget")]
async fn ehentai(paramsoptional: Option<web::Query<EhentaiIntake>>) -> HttpResponse {
    if let Some(params) = paramsoptional {
        let gid = params.gid;
        let token = &params.token;
        let proxy = reqwest::Proxy::all("http://localhost:7890").unwrap();
        let client = reqwest::Client::builder().proxy(proxy).build().unwrap();
        let res = client
            .post("https://e-hentai.org/api.php")
            .body(format!(
                "{{\"method\": \"gdata\",\"gidlist\": [[{},\"{}\"]],\"namespace\": 1}}",
                gid, token
            ))
            .header("Content-Type", "application/json")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        log(format!("Asked for {}", gid).as_str());
        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .insert_header(("Access-Control-Allow-Origin", "*"))
            .insert_header(("Access-Control-Allow-Methods", "GET, POST"))
            .insert_header(("X-Content-Type-Options", "nosniff"))
            .insert_header(("Content-Type", "application/json"))
            .body(res)
    } else {
        log("Asked for null");
        HttpResponse::Ok()
            .insert_header(("Access-Control-Allow-Origin", "*"))
            .insert_header(("Access-Control-Allow-Methods", "GET, POST"))
            .insert_header(("X-Content-Type-Options", "nosniff"))
            .body("Null")
    }
}

// params for post proxy. in POST
#[derive(Serialize, Deserialize)]
struct EhentaiIntakePostMode {
    method: String,
    gidlist: Vec<(i64, String)>,
    namespace: i64,
}
#[options("/api/ehentaiproxypost")]
async fn ehentaipreflightpost() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Access-Control-Allow-Headers", "*"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Methods", "GET, POST"))
        .insert_header(("Origin", "*"))
        .body(())
}

#[post("/api/ehentaiproxypost")]
async fn ehentaipost(paramsoptional: Option<web::Json<EhentaiIntakePostMode>>) -> HttpResponse {
    if let Some(params) = paramsoptional {
        let gid = params.gidlist[0].0;
        let token = &params.gidlist[0].1;
        let proxy = reqwest::Proxy::all("http://127.0.0.1:7890").unwrap();
        let client = reqwest::Client::builder().proxy(proxy).build().unwrap();
        let res = client
            .post("https://e-hentai.org/api.php")
            .body(format!(
                "{{\"method\": \"{}\",\"gidlist\": [[{},\"{}\"]],\"namespace\": {}}}",
                params.method, gid, token, params.namespace
            ))
            .header("Content-Type", "application/json")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        log(format!("Asked for {}", gid).as_str());
        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .insert_header(("Access-Control-Allow-Origin", "*"))
            .insert_header(("Access-Control-Allow-Methods", "GET, POST"))
            .insert_header(("X-Content-Type-Options", "nosniff"))
            .insert_header(("Content-Type", "application/json"))
            .body(res)
    } else {
        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .insert_header(("Access-Control-Allow-Credentials: true", "*"))
            .insert_header(("Access-Control-Allow-Origin", "*"))
            .insert_header(("Access-Control-Allow-Methods", "GET, POST"))
            .insert_header(("X-Content-Type-Options", "nosniff"))
            .body("null")
    }
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
    log(format!("Asked for latest: {}", response.title).as_str());
    Ok(web::Json(response)
        .customize()
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Methods", "POST, GET"))
        .insert_header(("X-Content-Type-Options", "nosniff")))
}

#[get("/updater/v1/tt20")]
async fn fauxupd() -> HttpResponse {
    HttpResponse::Ok()
            .content_type(ContentType::json())
            .body("{\"status\":true,\"latest\":\"0.7.0\",\"updateMessage\":\"TT20 0.7.0 has released, with more tick accelerators and bug fixes. Download it on Modrinth!\"}")
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    checkloglatest();
    log("Starting...");
    HttpServer::new(|| {
        App::new()
            // .wrap(Cors::default().allowed_origin("*"))
            .service(get)
            .service(latest)
            .service(ehentai)
            .service(ehentaipost)
            .service(ehentaipreflight)
            .service(ehentaipreflightpost)
            .service(fauxupd)
            .service(ehimages)
    })
    .bind(("0.0.0.0", 5766))?
    .run()
    .await
}
fn checkloglatest() {
    match exists("./logs").unwrap() {
        false => create_dir("./logs").unwrap(),
        true => (),
    }
    match exists("./logs/latest.log").unwrap() {
        true => {
            rename(
                "./logs/latest.log",
                format!("./logs/{}.log", chrono::offset::Utc::now().timestamp()),
            )
            .unwrap();
        }
        _ => (),
    }
}
fn initdb() -> Result<Connection> {
    match exists("./data.db").unwrap() {
        true => {
            log("Opening db connection...");
            let conn = Connection::open("./data.db").unwrap();
            return Ok(conn);
        }
        false => {
            log("No dbs found. Recreating one...");
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
