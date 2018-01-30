use rocket::Route;
use maud::{html, Markup, PreEscaped};

use db;
use db::models::Account;
use templates::{Page, PageBuilder};
use error::Perhaps;

pub fn routes() -> Vec<Route> {
    routes![index, user_page]
}

#[get("/users/<username>", format = "text/html")]
pub fn user_page(username: String, db_conn: db::Connection) -> Perhaps<Page> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));

    let rendered = PageBuilder::default()
        .title(format!("@{user}", user = account.username))
        .content(html! {
            div.h-card {
                header {
                    h1 a.u-url.u-uid href=(account.get_uri()) {
                        span.p-name (account.display_name.as_ref().unwrap_or(&account.username))
                    }

                    div (account.fully_qualified_username())
                }

                div.p-note {
                    @if let Some(bio) = account.summary.as_ref() {
                        (PreEscaped(bio))
                    } @else {
                        p {}
                    }
                }
            }
        })
        .build()
        .unwrap(); // note: won't panic since content is provided.

    Ok(Some(rendered))
}

#[get("/")]
pub fn index() -> Page {
    PageBuilder::default()
        .content(html! {
            h1 "Rustodon"
            p small "Templated with Maud!"
        })
        .build()
        .unwrap() // note: won't panic since content is provided.
}
