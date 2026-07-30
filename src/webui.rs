use crate::config::WebUiConfig;
use axum::{
    Router,
    body::Body,
    extract::OriginalUri,
    http::{
        HeaderValue, Method, Response, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
};

struct EmbeddedAsset {
    path: &'static str,
    content_type: &'static str,
    bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/webui_assets.rs"));

pub async fn start_server(config: WebUiConfig) -> std::io::Result<()> {
    if !config.enabled {
        println!("Embedded WebUI server disabled");
        return Ok(());
    }

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let app = Router::new().fallback(serve_asset);

    println!("Embedded WebUI listening on http://{addr}");
    axum::serve(listener, app).await
}

async fn serve_asset(method: Method, OriginalUri(uri): OriginalUri) -> Response<Body> {
    if method != Method::GET && method != Method::HEAD {
        return Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::empty())
            .expect("valid method-not-allowed response");
    }

    let requested_path = uri.path().trim_start_matches('/').trim_end_matches('/');
    let requested_path = if requested_path.is_empty() {
        "index.html"
    } else {
        requested_path
    };

    let asset = find_asset(requested_path);
    let Some(asset) = asset else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .expect("valid not-found response");
    };

    let cache_control = if asset.path.starts_with("_app/immutable/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    let body = if method == Method::HEAD {
        Body::empty()
    } else {
        Body::from(asset.bytes)
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static(asset.content_type))
        .header(CACHE_CONTROL, HeaderValue::from_static(cache_control))
        .body(body)
        .expect("valid embedded asset response")
}

fn find_asset(requested_path: &str) -> Option<&'static EmbeddedAsset> {
    if let Some(asset) = EMBEDDED_ASSETS
        .iter()
        .find(|asset| asset.path == requested_path)
    {
        return Some(asset);
    }

    let html_path = format!("{requested_path}.html");
    if let Some(asset) = EMBEDDED_ASSETS.iter().find(|asset| asset.path == html_path) {
        return Some(asset);
    }

    let index_path = format!("{requested_path}/index.html");
    EMBEDDED_ASSETS
        .iter()
        .find(|asset| asset.path == index_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Uri;

    #[test]
    fn embedded_webui_contains_entrypoint() {
        assert!(
            EMBEDDED_ASSETS
                .iter()
                .any(|asset| asset.path == "index.html")
        );
    }

    #[tokio::test]
    async fn root_serves_embedded_html() {
        let response = serve_asset(Method::GET, OriginalUri(Uri::from_static("/"))).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE),
            Some(&HeaderValue::from_static("text/html; charset=utf-8"))
        );
    }

    #[tokio::test]
    async fn prerendered_route_serves_embedded_html() {
        let response = serve_asset(Method::GET, OriginalUri(Uri::from_static("/control"))).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE),
            Some(&HeaderValue::from_static("text/html; charset=utf-8"))
        );
    }

    #[tokio::test]
    async fn unknown_asset_returns_not_found() {
        let response = serve_asset(Method::GET, OriginalUri(Uri::from_static("/missing.js"))).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
