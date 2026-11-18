//! The other callers' one tool: two numbers added.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The tool's name.
pub const NAME: &str = "sum";

/// The tool's description, as `tools/list` states it.
pub const DESCRIPTION: &str = "Add two numbers. `sum` comes back as `a` plus `b`.";

/// The tool's parameters: the two numbers.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    /// The first number.
    pub a: f64,
    /// The second number.
    pub b: f64,
}

/// The tool's answer.
#[derive(Debug, Clone, Serialize)]
pub struct Sum {
    /// `a` plus `b`.
    pub sum: f64,
}

/// The sum of the two.
pub fn call(parameters: Parameters) -> Sum {
    Sum { sum: parameters.a + parameters.b }
}
