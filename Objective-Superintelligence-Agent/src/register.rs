//! `POST /register`: the arguments, once.

use axum::Json;
use axum::http::StatusCode;
use diverge_sdk::container_proxy::inside::register;
use diverge_sdk::container_proxy::inside::register::response::Response;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use tokio::sync::OnceCell;

use crate::arguments::Arguments;
use crate::dependency::dependency;

/// The weights of the one registration, in the arguments' order, once
/// it has happened: all a run needs of the arguments, since the tool
/// holds the agents themselves.
static REGISTERED: OnceCell<Vec<Decimal>> = OnceCell::const_new();

/// The weights the agent was registered with; none before it was.
pub fn weights() -> Option<&'static [Decimal]> {
    REGISTERED.get().map(Vec::as_slice)
}

/// `POST /register`: arguments that are not [`Arguments`], that list
/// no agent, or that carry a weight that is not greater than zero are
/// `400`; a second registration, whatever it carries, is `409`;
/// otherwise `200`, declaring the one [`dependency`] and nothing
/// else — no secret, and no permission, which an agent has none of.
pub async fn register(Json(request): Json<register::request::Request>) -> Result<(StatusCode, Json<Response>), (StatusCode, Json<Value>)> {
    let refused = |error: String| (StatusCode::BAD_REQUEST, Json(json!({ "kind": "arguments", "error": error })));
    let arguments = serde_json::from_value::<Arguments>(request.arguments).map_err(|error| refused(error.to_string()))?;
    let weights = arguments.weights().map_err(refused)?;
    let dependency = dependency(&arguments).map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "kind": "image", "error": error }))))?;
    match REGISTERED.set(weights) {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(Response {
                dependencies: vec![dependency],
                ..Response::default()
            }),
        )),
        Err(_) => Err((StatusCode::CONFLICT, Json(json!({ "kind": "registered", "error": "the agent is registered, and it never changes" })))),
    }
}
