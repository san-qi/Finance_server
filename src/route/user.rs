use chrono::Utc;
use serde_json::json;
use sqlx::{Acquire, MySql, Row};
use tide::{http::Cookie, log, Request, Response, StatusCode};
use tide_sqlx::SQLxRequestExt;

use crate::util::prelude::*;

pub async fn register(mut req: Request<()>) -> tide::Result {
    // Get body from request.
    let body_json = get_json(&mut req).await;
    // If the body is not exists then return .
    if body_json.as_ref().is_none() {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":10, "data":[], "details":"POST DATA NOT EXISTS"}))
            .build());
    }
    let body_json = body_json.unwrap();

    let uid = body_json
        .get("uid")
        .unwrap_or(&json!("0"))
        .to_string()
        .parse::<i64>()
        .unwrap();
    let password = String::from(
        body_json
            .get("password")
            .unwrap_or(&json!(""))
            .as_str()
            .unwrap(),
    );
    let email = String::from(
        body_json
            .get("email")
            .unwrap_or(&json!(""))
            .as_str()
            .unwrap(),
    );
    log::info!("{},{},{}", uid, password, email);

    // Check the format by regex.
    let res_msg = if !match_id(uid.to_string().as_str()) {
        Some(json!({"code":21, "data":[], "details":"INCORRECT UID FORMAT"}))
    } else if !match_password(password.as_str()) {
        Some(json!({"code":22, "data":[], "details":"INCORRECT PASSWORD FORMAT"}))
    } else if !match_email(email.as_str()) {
        Some(json!({"code":23, "data":[], "details":"INCORRECT EMAIL FORMAT"}))
    } else {
        None
    };
    if res_msg.is_some() {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(res_msg.unwrap())
            .build());
    }

    // Insert data into database.
    let mut conn = req.sqlx_conn::<MySql>().await;
    sqlx::query(
        format!(
            "insert into user(uid, psd, email) values('{}', '{}', '{}')",
            uid, password, email
        )
        .as_str(),
    )
    .execute(conn.acquire().await?)
    .await
    // on succeed
    .and_then(|_| {
        Ok(Response::builder(StatusCode::Ok)
            .body(json!({"code":0, "data":[], "details":"SUCCESSED"}))
            .build())
    })
    // or else user already exists, return statusCode 202 which mean accept but not slove
    .or_else(|_| {
        Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":1, "data":[], "details":"USER IS EXISTS"}))
            .build())
    })
}

pub async fn login(mut req: Request<()>) -> tide::Result {
    // Get body from request.
    let body_json = get_json(&mut req).await;
    // If the body is not exists then return .
    if body_json.as_ref().is_none() {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":10, "data":[], "details":"POST DATA NOT EXISTS"}))
            .build());
    }
    let body_json = body_json.unwrap();
    let uid = body_json
        .get("uid")
        .unwrap_or(&json!("0"))
        .to_string()
        .parse::<i64>()
        .unwrap();
    let password = String::from(
        body_json
            .get("password")
            .unwrap_or(&json!(""))
            .as_str()
            .unwrap(),
    );
    log::info!("{},{}", uid, password);

    // check the format by regex.
    let res_msg = if !match_id(uid.to_string().as_str()) {
        Some(json!({"code":21, "data":[], "details":"INCORRECT UID FORMAT"}))
    } else if !match_password(password.as_str()) {
        Some(json!({"code":22, "data":[], "details":"INCORRECT PASSWORD FORMAT"}))
    } else {
        None
    };
    if res_msg.is_some() {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(res_msg.unwrap())
            .build());
    }

    // Fetch password in database if user is exists.
    let mut conn = req.sqlx_conn::<MySql>().await;
    let row = sqlx::query("select psd from user where uid=?")
        .bind(uid)
        .fetch_optional(conn.acquire().await?)
        .await?;
    // Return if user is not exists.
    if row.as_ref().is_none() {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":11, "data":[], "details":"USER NOT EXISTS"}))
            .build());
    }
    let row = row.unwrap();
    let _password = row.get::<String, &str>("psd");
    if _password == password {
        let mut res = Response::new(StatusCode::Ok);
        let _info = format!("{}+{}", uid, Utc::now().timestamp());
        // The cookie is used to confirm id
        let info = encrypt_str(&_info, &password).unwrap();
        res.insert_cookie(Cookie::new("uid", uid.to_string()));
        res.insert_cookie(Cookie::new("info", info));
        res.set_body(json!({"code":0, "data":[], "details":"SUCCESSED"}));
        Ok(res)
    } else {
        Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":2, "data":[], "details": "PASSWORD DON'T MATCH"}))
            .build())
    }
}

pub async fn password(mut req: Request<()>) -> tide::Result {
    // Only exists user can login so that there is no necessary to check user's exists, and the format is correct.
    if !check_login(&req).await {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":11, "data":[], "details":"USER NOT LOGIN"}))
            .build());
    }

    // Get body from request.
    let body_json = get_json(&mut req).await;
    // If the password is not exists then return .
    if body_json.as_ref().is_none() {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":10, "data":[], "details":"POST DATA NOT EXISTS"}))
            .build());
    }
    let body_json = body_json.unwrap();

    // If user is logined there must have the cookie which is uid.
    let uid = req.cookie("uid").unwrap().value().parse::<i64>().unwrap();
    let password = String::from(
        body_json
            .get("password")
            .unwrap_or(&json!(""))
            .as_str()
            .unwrap(),
    );
    // Return if password's format is not right.
    if !match_password(password.as_str()) {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":22, "data":[], "details":"INCORRECT PASSWORD FORMAT"}))
            .build());
    }
    log::info!("{},{}", uid, password);

    let mut conn = req.sqlx_conn::<MySql>().await;
    let row = sqlx::query("update user set psd=? where uid=?")
        .bind(password)
        .bind(uid)
        .execute(conn.acquire().await?)
        .await;
    if row.is_err() {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":3, "data":[], "details":"PASSWORD CHANGED FAIL"}))
            .build());
    }

    Ok(Response::builder(StatusCode::Ok)
        .body(json!({"code":0, "data":[], "details":"SUCCESSED"}))
        .build())
}
