//! Exercise 2: a custom extractor that parses JSON *and* validates it.
//!
//! Handlers that take `ValidatedJson<TaskInput>` only ever see valid input.
//! Both failure modes become Problems:
//!   * body isn't valid JSON for the type -> 400 (axum's own rejection, reworded)
//!   * JSON is fine but breaks a rule     -> 422 with per-field messages

use crate::problem::Problem;
use axum::Json;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationErrors};

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Problem;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        todo!("Exercise 2")
    }
}

pub fn validation_problem(errors: ValidationErrors) -> Problem {
    todo!("Exercise 2")
}
