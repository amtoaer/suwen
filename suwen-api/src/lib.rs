use axum::Router;
use axum_reverse_proxy::ReverseProxy;
use tower_http::compression::CompressionLayer;

pub fn router() -> Router {
    Router::new()
        .merge(ReverseProxy::new("/", "http://localhost:4173"))
        .layer(CompressionLayer::new().gzip(true).br(true).zstd(true))
}
