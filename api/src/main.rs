#[macro_use]
extern crate rocket;
use rocket::response::status::NotFound;
use rocket_sync_db_pools::{database, postgres};

#[database("postgres_db")]
struct PostgresDB(postgres::Client);

#[get("/")]
async fn get(client: PostgresDB) -> Result<String, NotFound<String>> {
    client
        .run(
            |c| match c.query("SELECT message FROM logs ORDER BY id DESC LIMIT 1", &[]) {
                Err(e) => Err(NotFound(format!("{}", e))),
                Ok(rows) => Ok(match rows.last() {
                    None => "No logs yet!".to_owned(),
                    Some(row) => row.get("message"),
                }),
            },
        )
        .await
}

#[put("/", data = "<message>")]
async fn put(client: PostgresDB, message: String) -> Result<String, NotFound<String>> {
    client
        .run(
            move |c| match c.execute("INSERT INTO logs(message) VALUES($1);", &[&message]) {
                Ok(_) => Ok("Logged!".to_owned()),
                Err(e) => Err(NotFound(format!("{}", e))),
            },
        )
        .await
}

#[post("/")]
async fn post(client: PostgresDB) -> Result<String, NotFound<String>> {
    client
        .run(|c| {
            match c.execute(
                "CREATE TABLE logs ( id SERIAL PRIMARY KEY, message TEXT NOT NULL );",
                &[],
            ) {
                Ok(_) => Ok("Table created!".to_owned()),
                Err(e) => Err(NotFound(format!("{}", e))),
            }
        })
        .await
}

#[delete("/")]
async fn delete(client: PostgresDB) -> Result<String, NotFound<String>> {
    client
        .run(|c| match c.execute("DROP TABLE logs;", &[]) {
            Ok(_) => Ok("Table dropped!".to_owned()),
            Err(e) => Err(NotFound(format!("{}", e))),
        })
        .await
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(PostgresDB::fairing())
        .mount("/", routes![get, put, post, delete])
}
