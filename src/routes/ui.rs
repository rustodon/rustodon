use rocket::Route;
use maud::{html, Markup};

use db;
use db::models::Account;
use templates::{Page, PageBuilder};
use error::Perhaps;

pub fn routes() -> Vec<Route> {
    routes![index, user_page]
}

#[get("/users/<username>", format = "text/html")]
pub fn user_page(username: String, db_conn: db::Connection) -> Perhaps<()> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));

    Ok(Some(()))
}

#[get("/")]
pub fn index() -> Page {
    PageBuilder::default()
        .content(html! {
            h1 "Rustodon"
            p small "Templated with Maud!"
        })
        .build()
        .unwrap()
}
