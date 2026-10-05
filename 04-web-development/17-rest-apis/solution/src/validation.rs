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
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| Problem::bad_request(rejection.body_text()))?;
        value.validate().map_err(validation_problem)?;
        Ok(ValidatedJson(value))
    }
}

pub fn validation_problem(errors: ValidationErrors) -> Problem {
    let mut problem = Problem::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "validation",
        "Validation failed",
    )
    .with_detail("The request body has invalid fields");
    for (field, errs) in errors.field_errors() {
        let messages = errs
            .iter()
            .map(|e| {
                e.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| e.code.to_string())
            })
            .collect();
        problem.errors.insert(field.to_string(), messages);
    }
    problem
}
