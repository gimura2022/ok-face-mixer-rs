use std::{
    io::{self, Cursor},
    str::FromStr,
};

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use image::ImageFormat;
use log::{debug, error};
use ok_face_mixer_core::{Smile, SmileType};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
struct Info {
    left: String,
    right: String,
}

#[get("/api/mix_image.gif")]
async fn root(req: web::Query<Info>) -> impl Responder {
    debug!("processing smile: {:?}", req.0);

    let left = SmileType::from_str(&req.left);
    let right = SmileType::from_str(&req.right);

    if left
        .as_ref()
        .inspect_err(|_| error!("left smile creating error"))
        .is_err()
        || right
            .as_ref()
            .inspect_err(|_| error!("right smile creating error"))
            .is_err()
    {
        return HttpResponse::BadRequest().body("invalid smile name");
    }

    let (left, right) = (left.unwrap(), right.unwrap());

    let res = Smile::new(left, right).generate();

    debug!("writing image to gif bytes");
    let mut bytes: Vec<u8> = Vec::new();
    if res
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Gif)
        .inspect_err(|_| error!("image write error"))
        .is_err()
    {
        return HttpResponse::InternalServerError().body("failed to convert result image to bytes");
    }

    HttpResponse::Ok().content_type("image/gif").body(bytes)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .try_init()
        .expect("logger initialization error");

    debug!("running server");

    HttpServer::new(|| {
        App::new().service(root).service(
            actix_files::Files::new("/", "./ok-face-mixer-web/dist").index_file("index.html"),
        )
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
