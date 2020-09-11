use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use listenfd::ListenFd;
use url::form_urlencoded::parse;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

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
    let mut video_urls = Vec::new();
    for video in playlist {
        video_urls.push(video.url);
    }
    video_urls.sort();
    let mut video_list = String::new();
    for url in video_urls {
        let video_part = include_str!("../site/partials/video.html");
        video_list.push_str(&video_part
            .replace("video_url", &url)
            .replace("video_name", &url));
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

async fn dispatch_action(form_data: String) -> impl Responder {
    let mut data = parse(form_data.as_bytes()).into_owned();
    let item = data.next().unwrap();
    match item.0.as_str() {
        "add_video" => add_video(item.1),
        "remove_video" => remove_video(item.1),
        "play_video" => play_video(
            item.1,
            data.next().unwrap().1,
            data.next().unwrap().1,
            data.next().unwrap().1),
        _ => (),
    };
    index().await
}

fn add_video(video: String) {
    let mut playlist = read_playlist();
    playlist.insert(Video { url: video });
    write_playlist(playlist);
}

fn remove_video(video: String) {
    let mut playlist = read_playlist();
    playlist.remove(&Video { url: video });
    write_playlist(playlist);
}

fn play_video(video: String, hour: String, minute: String, second: String) {
    Command::new("sh")
        .arg("-c")
        .arg("killall -9 vlc").status().unwrap();
    Command::new("sh")
        .arg("-c")
        .arg("killall -9 streamlink").status().unwrap();
    if video.starts_with("https://www.youtube.com/") {
        Command::new("vlc")
            .arg(video)
            .arg("--fullscreen")
            .spawn().unwrap();
    } else if video.starts_with("https://www.twitch.tv/") {
        Command::new("streamlink")
            .arg(video)
            .arg("best")
            .arg("--hls-start-offset")
            .arg(format!("{}:{}:{}", hour, minute, second))
            .spawn().unwrap();
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/", web::post().to(dispatch_action))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind(("0.0.0.0", 3000))?
    };

    server.run().await
}
