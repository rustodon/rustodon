use std::path::{Path, PathBuf};
use rocket::Route;
use rocket::request::FlashMessage;
use rocket::response::NamedFile;
use maud::{html, PreEscaped};

use db;
use db::models::{Account, User};
use templates::Page;
use error::Perhaps;

mod auth;

pub fn routes() -> Vec<Route> {
    routes![
        index,
        user_page,
        auth::signin_get,
        auth::signin_post,
        auth::signout,
        auth::signup_get,
        auth::signup_post,
        static_files
    ]
}

#[get("/users/<username>", format = "text/html")]
pub fn user_page(username: String, db_conn: db::Connection) -> Perhaps<Page> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));

    let rendered = Page::new()
        .title(format!("@{user}", user = account.username))
        .content(html! {
            div.h-card {
                header {
                    span.p-name (account.display_name.as_ref().unwrap_or(&account.username))

                    span.fq-username {
                        a.url.u-uid href=(account.get_uri()) (account.fully_qualified_username())
                    }
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
pub fn index(flash: Option<FlashMessage>, user: Option<User>) -> Page {
    Page::new().flash(flash).content(html! {
        header h1 "Rustodon"

        div {
            @if let None = user {
                a href="/auth/sign_in" "sign in!"
                " | "
                a href="/auth/sign_up" "sign up?"
            } @else {
                form.inline method="post" action="/auth/sign_out" {
                    input type="hidden" name="stub"
                    button.link type="submit" name="submit" "sign out."
                }
            }
        }
    })
}

#[get("/static/<path..>")]
fn static_files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}
