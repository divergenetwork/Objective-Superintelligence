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

/// What a run needs of the one registration, once it has happened.
pub struct Registered {
    /// The agents, as the tool's call takes them, in the arguments'
    /// order.
    pub agents: Vec<Value>,
    /// Each agent's weight, in the same order.
    pub weights: Vec<Decimal>,
}

/// The one registration, once it has happened.
static REGISTERED: OnceCell<Registered> = OnceCell::const_new();

/// What the agent was registered with; none before it was.
pub fn registered() -> Option<&'static Registered> {
    REGISTERED.get()
}

/// `POST /register`: arguments that are not [`Arguments`], that list
/// no agent, or that carry a weight that is not greater than zero are
/// `400`; a second registration, whatever it carries, is `409`;
/// otherwise `200`, declaring the one [`dependency`], of the image the
/// arguments name as `tool`, and nothing else — no secret, and no permission, which an agent has none of.
pub async fn register(Json(request): Json<register::request::Request>) -> Result<(StatusCode, Json<Response>), (StatusCode, Json<Value>)> {
    let refused = |error: String| (StatusCode::BAD_REQUEST, Json(json!({ "kind": "arguments", "error": error })));
    let arguments = serde_json::from_value::<Arguments>(request.arguments).map_err(|error| refused(error.to_string()))?;
    let weights = arguments.weights().map_err(refused)?;
    let agents = arguments.called();
    let dependency = dependency(arguments.tool);
    match REGISTERED.set(Registered { agents, weights }) {
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
