use std::net::SocketAddr;

use axum::{response::Html, routing::get};
use axum_extra::routing::SpaRouter;
use serde_json::{json, Value};
use tower_http::trace::TraceLayer;

use phoenix::channels::{user_channel::UserChannel, user_socket::UserSocket};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let mut user_socket = UserSocket::new();

    user_socket.join(|_payload, socket| async move {
        let mut socket = socket.lock().await;
        socket.joined();
    });

    let mut user_channel = UserChannel::new("room:*");

    user_channel.join(|_payload, socket| async move {
        let socket = socket.lock().await;

        socket
            .boardcast("room:*", "boardcast", json!({"message": "hello world!"}))
            .await;
        socket.reply("ok", Value::Null).await;
    });

    user_channel.handle_event("test", |payload, socket| async move {
        tracing::info!("{}", payload);
        let socket = socket.lock().await;
        socket.reply("ok", Value::Null).await;
    });

    user_channel.handle_event("test2", |payload, socket| async move {
        tracing::info!("{}", payload);

        socket.lock().await.reply("ok", Value::Null).await;
    });

    user_socket.channel(user_channel);

    let app: _ = axum::Router::new()
        .route("/", get(index))
        .merge(SpaRouter::new("/assets", "./priv/static/assets"))
        .merge(UserSocket::router(user_socket))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 4001));
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("templates/index.html"))
}
