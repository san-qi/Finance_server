use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use sqlx::{Acquire, MySql, Row};
use tide::{log, Request, Response, StatusCode};
use tide_sqlx::SQLxRequestExt;

use crate::util::prelude::check_login;

#[derive(Deserialize)]
struct Query {
    rid: i64,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            rid: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Record {
    id: i64,
    date: String,
    record_type: String,
    amount: f32,
    is_income: bool,
}

// This function do not need rid as query.
pub async fn upload(mut req: Request<()>) -> tide::Result {
    // Only exists user can login so that there is no necessary to check user's exists.
    if !check_login(&req).await {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":11, "data":[], "details":"USER NOT LOGIN"}))
            .build());
    }
    // If user is logined there must have the cookie which is uid and the format is correct.
    let uid = req.cookie("uid").unwrap().value().parse::<i64>().unwrap();

    let records_json: Vec<Record> = req.body_json().await?;
    // Insert data into database.
    let mut conn = req.sqlx_conn::<MySql>().await;
    // Insert data and return response.

    // Get the difference.
    let max_rid = sqlx::query("select rid from record where uid=?")
        .bind(uid)
        .fetch_all(conn.acquire().await?)
        .await?
        .into_iter()
        .max_by_key(|record| record.get::<i64, &str>("rid"))
        .map(|row| row.get::<i64, &str>("rid"))
        .unwrap_or_default();
    let start_rid = records_json
        .clone()
        .into_iter()
        .min_by_key(|record| record.id)
        .map(|record| record.id)
        .unwrap_or_default();

    let mut err_count = 0;
    for mut ele in records_json {
        ele.id += max_rid + 1 - start_rid;
        let res = sqlx::query(&String::from(format!(
            "insert into record(uid, rid, details) values('{}', '{}', '{}')",
            uid,
            &ele.id,
            serde_json::to_string(&ele)?
        )))
        .execute(conn.acquire().await?)
        .await;
        log::info!("{:?}", ele);
        if let Err(e) = res {
            err_count += 1;
            println!("{}", e);
        }
    }
    if err_count > 0 {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":4, "data":[], "details":format!("{} RECORDS FAILED", err_count)}))
            .build());
    }

    Ok(Response::builder(StatusCode::Ok)
        .body(json!({"code":0, "data":[], "details":"SUCCESSED"}))
        .build())
}

// Delete from table record whose rid between 0 and end_rid if necessary [0, end_rid].
// The end_rid default value is 0.
// The end_rid is include.
pub async fn delete(mut req: Request<()>) -> tide::Result {
    // Only exists user can login so that there is no necessary to check user's exists.
    if !check_login(&req).await {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":11, "data":[], "details":"USER NOT LOGIN"}))
            .build());
    }
    // If user is logined there must have the cookie which is uid and the format is correct.
    let uid = req.cookie("uid").unwrap().value().parse::<i64>().unwrap();
    // Delete from table record whose rid between 0 and end_rid if necessary [0, end_rid], and end_rid default value is 0.
    let end_rid = (req.query::<Query>().unwrap_or_default() as Query).rid;

    let delete_vec: Vec<i64> = req.body_json().await?;
    let delete_vec = delete_vec
        .into_iter()
        .filter(|it| it <= &end_rid)
        .map(|it| it.to_string())
        .reduce(|old, item| format!("{}, {}", old, item));
    if delete_vec.is_none() {
        return Ok(Response::builder(StatusCode::Ok)
            .body(json!({"code":0, "data":[], "details":"SUCCESSED"}))
            .build());
    }
    // Delete data
    let mut conn = req.sqlx_conn::<MySql>().await;
    let res = sqlx::query(&String::from(format!(
        "delete from record where uid={} and rid in ({})",
        uid,
        delete_vec.unwrap()
    )))
    .execute(conn.acquire().await?)
    .await;
    if let Err(_) = res {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":5, "data":[], "details":"RECORDS DELETE FAILED"}))
            .build());
    }

    Ok(Response::builder(StatusCode::Ok)
        .body(json!({"code":0, "data":[], "details":"SUCCESSED"}))
        .build())
}

// Retry from table record whose rid between start_rid and max_rid if necessary (start_rid, ..].
// The start_rid default value is 0.
// The start_rid is exclude.
pub async fn download(req: Request<()>) -> tide::Result {
    // Only exists user can login so that there is no necessary to check user's exists.
    if !check_login(&req).await {
        return Ok(Response::builder(StatusCode::Accepted)
            .body(json!({"code":11, "data":[], "details":"USER NOT LOGIN"}))
            .build());
    }
    // If user is logined there must have the cookie which is uid and the format is correct.
    let uid = req.cookie("uid").unwrap().value().parse::<i64>().unwrap();
    // Retry from table record whose rid between start_rid and max_rid if necessary (start_rid, ..], default value is 0.
    let start_rid: i64 = (req.query::<Query>().unwrap_or_default() as Query).rid;

    let mut conn = req.sqlx_conn::<MySql>().await;
    let rows = sqlx::query("select details from record where uid=? and rid>?")
        .bind(uid)
        .bind(start_rid)
        .fetch_all(conn.acquire().await?)
        .await?;
    let download_data: Vec<Json> = rows
        .into_iter()
        .map(|row| row.get::<Json, usize>(0))
        .collect();

    Ok(Response::builder(StatusCode::Ok)
        .body(json!({"code":0, "data":json!(download_data), "details":"SUCCESSED"}))
        .build())
}
