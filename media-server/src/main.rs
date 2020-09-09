use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use listenfd::ListenFd;
use url::form_urlencoded::parse;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
struct Video {
    url: String,
}

impl PartialEq for Video {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

async fn index() -> impl Responder {
    let html = include_str!("../site/index.html");
    let css = include_str!("../site/index.css");
    let playlist = read_playlist();
    let mut video_list = String::new();
    for video in playlist {
        let video_part = include_str!("../site/partials/video.html");
        video_list.push_str(&video_part
            .replace("video_url", &video.url)
            .replace("video_name", &video.url));
    }
    HttpResponse::Ok().body(html
        .replace("css_file", css)
        .replace("video_list", &video_list))
}

fn read_playlist() -> HashSet<Video> {
    let mut contents = String::new();
    match File::open("playlist.json") {
        Ok(mut file) => match file.read_to_string(&mut contents) {
            Ok(_) => (),
            _ => return HashSet::new(),
        },
        _ => return HashSet::new(),
    }
    serde_json::from_str(&contents).unwrap()
}

fn write_playlist(playlist: HashSet<Video>) {
    let mut file = File::create("playlist.json").unwrap();
    let playlist_json = serde_json::to_string(&playlist).unwrap();
    file.write_all(playlist_json.as_bytes()).unwrap();
}

async fn add_video(form_data: String) -> impl Responder {
    let mut data = parse(form_data.as_bytes()).into_owned();
    let video = Video { url: data.next().unwrap().1 };
    let mut playlist = read_playlist();
    playlist.insert(video);
    write_playlist(playlist);
    index().await
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/", web::post().to(add_video))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:3000")?
    };

    server.run().await
}
