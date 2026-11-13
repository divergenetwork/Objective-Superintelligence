//! `POST /register`: the arguments, once.

use axum::Json;
use axum::http::StatusCode;
use diverge_sdk::container_proxy::inside::register;
use diverge_sdk::container_proxy::inside::register::response::Response;
use serde_json::{Value, json};
use tokio::sync::OnceCell;

use crate::arguments::Arguments;
use crate::permissions::permissions;

/// The arguments of the one registration, once it has happened.
static REGISTERED: OnceCell<Arguments> = OnceCell::const_new();

/// The arguments the tool was registered with; none before it was.
pub fn arguments() -> Option<&'static Arguments> {
    REGISTERED.get()
}

/// `POST /register`: arguments that are not [`Arguments`], or that
/// list no agent, are `400`; a second registration, whatever it
/// carries, is `409`; otherwise `200` with the tool's [`permissions`]
/// — it depends on nothing and takes no secret.
pub async fn register(Json(request): Json<register::request::Request>) -> Result<(StatusCode, Json<Response>), (StatusCode, Json<Value>)> {
    let arguments = match serde_json::from_value::<Arguments>(request.arguments) {
        Ok(arguments) => arguments,
        Err(error) => return Err((StatusCode::BAD_REQUEST, Json(json!({ "kind": "arguments", "error": error.to_string() })))),
    };
    if arguments.agents.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "kind": "arguments", "error": "`agents` lists no agent, and there must be at least one" }))));
    }
    let permissions = permissions(&arguments);
    match REGISTERED.set(arguments) {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(Response {
                permissions,
                ..Response::default()
            }),
        )),
        Err(_) => Err((StatusCode::CONFLICT, Json(json!({ "kind": "registered", "error": "the tool is registered, and it never changes" })))),
    }
}
