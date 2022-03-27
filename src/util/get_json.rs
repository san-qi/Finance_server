use serde_json::Value as Json;
use tide::{Request, Result};

pub async fn get_json(req: &mut Request<()>) -> Option<Json> {
    let req = &mut *req;
    let body_json: Result<Json> = req.body_json().await;
    // If the post data is not exists then return .
    if body_json.is_err() {
        None
    } else {
        Some(body_json.unwrap())
    }
}
