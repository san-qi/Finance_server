use chrono::{Duration, TimeZone, Utc};
use sqlx::{Acquire, MySql, Row};
use tide::Request;
use tide_sqlx::SQLxRequestExt;

use super::prelude::decrypt_str;

pub async fn check_login(req: &Request<()>) -> bool {
    // Get uid from cookie.
    let uid = req
        .cookie("uid")
        .and_then(|it| Some(it.value().parse::<i64>().unwrap_or_default()))
        .or_else(|| Some(0i64))
        .unwrap();
    // Get password by uid from database.
    // Actually if uid is exists the password must be exists so that could unwrap dirextlly
    let mut conn = req.sqlx_conn::<MySql>().await;
    let conn = conn.acquire().await;
    if conn.is_err() {
        return false;
    }
    let conn = conn.unwrap();
    let row = sqlx::query("select psd from user where uid=?")
        .bind(uid)
        .fetch_optional(conn)
        .await;
    if row.as_ref().is_err() || row.as_ref().unwrap().is_none() {
        return false;
    }
    let password = row.unwrap().unwrap().get::<String, &str>("psd");
    // Check info to confirm whether the user is logged in.
    match req.cookie("info") {
        Some(info_cookie) => {
            let info = info_cookie.value();
            let info = decrypt_str(info, &password);
            if info.is_none() {
                return false;
            }
            let info = info.unwrap();
            let ans = info.split("+").collect::<Vec<_>>();
            if ans[0].parse::<i64>().unwrap_or_default() != uid {
                return false;
            }
            let timestamp = ans[1].parse::<i64>().unwrap();
            let dt = Utc.timestamp(timestamp, 0);

            // expires in 3 days
            dt + Duration::days(3) > Utc::now()
        }
        None => false,
    }
}
