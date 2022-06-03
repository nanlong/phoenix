use std::net::SocketAddr;

use axum::{response::Html, routing::get};
use axum_extra::routing::SpaRouter;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use phoenix::channels;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "phoenix=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app: _ = axum::Router::new()
        .route("/", get(index))
        .merge(SpaRouter::new("/assets", "./priv/static/assets"))
        .merge(channels::user_socket::router())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 4001));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("templates/index.html"))
}
