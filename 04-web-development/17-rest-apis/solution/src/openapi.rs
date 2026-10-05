//! Exercise 5: an OpenAPI 3 document, generated from the code.
//!
//! `#[derive(ToSchema)]` on types and `#[utoipa::path(...)]` on handlers
//! describe the API; `#[derive(OpenApi)]` collects them. Because the document
//! is generated from the same code that serves the requests, it can't drift
//! out of date the way a hand-written spec does.
//!
//! Paste http://127.0.0.1:3000/openapi.json into https://editor.swagger.io to
//! browse it, or generate clients from it with openapi-generator.

use crate::api::{self, Links, TaskV2};
use crate::model::{Status, Task, TaskInput};
use crate::pagination::Page;
use crate::problem::Problem;
use axum::Json;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Tasks API", version = "2.0.0", description = "The rustLearn module 17 example API"),
    paths(
        api::list_tasks, api::create_task, api::get_task, api::replace_task, api::delete_task,
        api::list_tasks_v2, api::get_task_v2,
    ),
    components(schemas(Task, TaskInput, Status, Problem, TaskV2, Links, Page<Task>, Page<TaskV2>)),
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
