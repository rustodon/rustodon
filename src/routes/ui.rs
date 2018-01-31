use std::path::{Path, PathBuf};
use rocket::Route;
use rocket::response::NamedFile;
use maud::{html, PreEscaped};

use db;
use db::models::Account;
use templates::Page;
use error::Perhaps;

pub fn routes() -> Vec<Route> {
    routes![index, user_page, auth_signin_get, static_files]
}

#[get("/auth/sign_in")]
pub fn auth_signin_get() -> Page {
    Page::new()
        .title("sign in")
        .content(html! {
            form method="post" {
                header h2 "sign in"

                div {
                    label for="username" "username:"
                    input type="text" id="username" name="username";
                }

                div {
                    label for="password" "password:"
                    input type="text" id="password" name="password";
                }
            }
        })
}

#[get("/users/<username>", format = "text/html")]
pub fn user_page(username: String, db_conn: db::Connection) -> Perhaps<Page> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));

    let rendered = Page::new()
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
        });

    Ok(Some(rendered))
}

#[get("/")]
pub fn index() -> Page {
    Page::new().content(html! {
        header h1 "Rustodon"

        div {
            a href="/auth/sign_in" "sign in!"
            " | "
            a href="/auth/sign_up" "sign up?"
        }
    })
}

#[get("/static/<path..>")]
fn static_files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}
