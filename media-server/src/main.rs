use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use listenfd::ListenFd;

async fn index() -> impl Responder {
    let index = include_str!("../site/index.html");
    HttpResponse::Ok().body(index)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:3000")?
    };

    server.run().await
}
