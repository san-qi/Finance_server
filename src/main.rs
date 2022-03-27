mod route;
mod util;

use route::prelude::*;

use sqlx::mysql::MySqlPoolOptions;
use tide_sqlx::SQLxMiddleware;

/* code: [register, password, login, upload, delete, download]
0:  SUCCEED                         [register, password, login, upload, delete, download]
1:  USER IS EXISTS                  [register]
2:  PASSWORD DON'T MATCH            [login]
3:  PASSWORD CHANGED FAIL           [password]
4:  SOME RECORDS FAILED             [upload]
5:  RECORDS DELETE FAILED           [delete]
10: POST DATA NOT EXISTS            [register, password, login]
11: USER NOT LOGIN/EXISTS           [password, login, upload, delete, download]
21: INCORRECT UID FORMAT            [register, login]
22: INCORRECT PASSWORD FORMAT       [register, password, login]
23: INCORRECT EMAIL FORMAT          [register]
 */
#[async_std::main]
async fn main() -> tide::Result<()> {
    // tide::log::start();

    let mut app = tide::new();

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect("mysql://sanqi:password@localhost/finance")
        .await?;
    app.with(SQLxMiddleware::from(pool));

    app.at("/register").post(register);
    app.at("/password").post(password);
    app.at("/login").post(login);
    app.at("/upload").post(upload);
    app.at("/delete").post(delete);
    app.at("/download").get(download);
    app.listen("0.0.0.0:8084").await?;

    Ok(())
}
