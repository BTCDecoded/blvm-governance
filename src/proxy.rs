//! Optional front door for blvm-commons.
//!
//! Unset `listen` means this process does not accept HTTP. A set `commons_url`
//! is where block posts are forwarded. GitHub webhooks are not forwarded.

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::routing::post;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

use crate::error::GovernanceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inbound {
    Block,
    Ignore,
    Bad,
}

/// Where this process posts its own blocks.
/// `commons_url` wins, so a proxy does not post back to its own listen address.
pub fn block_post_url(webhook_url: Option<&str>, commons_url: Option<&str>) -> Option<String> {
    if let Some(base) = filled(commons_url) {
        return Some(format!("{}/webhooks/block", base.trim_end_matches('/')));
    }
    filled(webhook_url).map(str::to_string)
}

pub fn classify_inbound(body: &[u8]) -> Inbound {
    match serde_json::from_slice::<Value>(body) {
        Ok(value) if value.get("block_hash").and_then(Value::as_str).is_some() => Inbound::Block,
        Ok(_) => Inbound::Ignore,
        Err(_) => Inbound::Bad,
    }
}

fn filled(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

struct ProxyState {
    client: Client,
    commons_block_url: String,
}

/// Listen and forward block posts. Other bodies are acknowledged and not forwarded.
pub async fn serve(listen: String, commons_url: String) -> Result<(), GovernanceError> {
    let Some(commons_block_url) = block_post_url(None, Some(commons_url.as_str())) else {
        return Err(GovernanceError::ConfigError(
            "commons_url is empty".to_string(),
        ));
    };
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| GovernanceError::WebhookError(format!("proxy client: {e}")))?;
    let state = Arc::new(ProxyState {
        client,
        commons_block_url,
    });
    let app = Router::new()
        .route("/webhooks/block", post(handle))
        .route("/", post(handle))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .map_err(|e| GovernanceError::ConfigError(format!("bind {listen}: {e}")))?;
    tracing::info!(
        "governance proxy listening on {} and forwarding block posts to commons",
        listener
            .local_addr()
            .map(|addr| addr.to_string())
            .unwrap_or(listen)
    );
    axum::serve(listener, app)
        .await
        .map_err(|e| GovernanceError::WebhookError(format!("proxy server: {e}")))
}

async fn handle(State(state): State<Arc<ProxyState>>, body: Bytes) -> Response {
    match classify_inbound(&body) {
        Inbound::Bad => StatusCode::BAD_REQUEST.into_response(),
        Inbound::Ignore => StatusCode::NO_CONTENT.into_response(),
        Inbound::Block => forward_block(&state, body).await,
    }
}

async fn forward_block(state: &ProxyState, body: Bytes) -> Response {
    match state
        .client
        .post(&state.commons_block_url)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await
    {
        Ok(response) => {
            let status =
                StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let bytes = response.bytes().await.unwrap_or_default();
            (status, bytes).into_response()
        }
        Err(error) => (StatusCode::BAD_GATEWAY, error.to_string()).into_response(),
    }
}

trait IntoResponseStatus {
    fn into_response(self) -> Response;
}

impl IntoResponseStatus for StatusCode {
    fn into_response(self) -> Response {
        axum::response::IntoResponse::into_response(self)
    }
}

impl IntoResponseStatus for (StatusCode, Bytes) {
    fn into_response(self) -> Response {
        axum::response::IntoResponse::into_response(self)
    }
}

impl IntoResponseStatus for (StatusCode, String) {
    fn into_response(self) -> Response {
        axum::response::IntoResponse::into_response(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commons_url_wins_and_empty_config_posts_nowhere() {
        assert_eq!(
            block_post_url(Some("http://leaf/hook"), Some("http://127.0.0.1:3000/")),
            Some("http://127.0.0.1:3000/webhooks/block".to_string())
        );
        assert_eq!(
            block_post_url(Some("http://proxy:9417/webhooks/block"), None),
            Some("http://proxy:9417/webhooks/block".to_string())
        );
        assert_eq!(block_post_url(None, Some("  ")), None);
    }

    #[test]
    fn only_a_block_body_is_forwarded() {
        assert_eq!(
            classify_inbound(br#"{"block_hash":"aa","block_height":1}"#),
            Inbound::Block
        );
        assert_eq!(
            classify_inbound(br#"{"event_type":"proposal_created"}"#),
            Inbound::Ignore
        );
        assert_eq!(classify_inbound(br#"{"zen":"github"}"#), Inbound::Ignore);
        assert_eq!(classify_inbound(b"not-json"), Inbound::Bad);
    }

    #[tokio::test]
    async fn proxy_forwards_a_block_and_drops_other_posts() {
        let commons = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let commons_addr = commons.local_addr().unwrap();
        let seen = Arc::new(std::sync::Mutex::new(None));
        let seen_route = Arc::clone(&seen);
        let commons_app = Router::new().route(
            "/webhooks/block",
            post(move |body: Bytes| {
                let seen_route = Arc::clone(&seen_route);
                async move {
                    *seen_route.lock().unwrap() = Some(body);
                    (StatusCode::OK, "ack")
                }
            }),
        );
        tokio::spawn(async move {
            axum::serve(commons, commons_app).await.unwrap();
        });

        let proxy = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let proxy_addr = proxy.local_addr().unwrap();
        drop(proxy);
        let commons_url = format!("http://{commons_addr}");
        tokio::spawn(async move {
            serve(proxy_addr.to_string(), commons_url).await.unwrap();
        });

        let client = Client::new();
        let block_url = format!("http://{proxy_addr}/webhooks/block");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            match client
                .post(&block_url)
                .json(&serde_json::json!({"block_hash":"aa","block_height":1,"block":{}}))
                .send()
                .await
            {
                Ok(response) => {
                    assert_eq!(response.status(), reqwest::StatusCode::OK);
                    break;
                }
                Err(_) if std::time::Instant::now() < deadline => {
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
                Err(error) => panic!("proxy did not accept a block: {error}"),
            }
        }
        let forwarded = seen
            .lock()
            .unwrap()
            .clone()
            .expect("commons received a block");
        let forwarded: serde_json::Value = serde_json::from_slice(&forwarded).unwrap();
        assert_eq!(forwarded["block_hash"], "aa");
        assert_eq!(forwarded["block_height"], 1);

        let ignored = client
            .post(&block_url)
            .body(r#"{"zen":"github"}"#)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        assert_eq!(ignored.status(), reqwest::StatusCode::NO_CONTENT);
        let still = seen.lock().unwrap().clone().unwrap();
        let still: serde_json::Value = serde_json::from_slice(&still).unwrap();
        assert_eq!(still["block_hash"], "aa");
    }
}
