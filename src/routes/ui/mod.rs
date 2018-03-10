use std::path::{Path, PathBuf};
use std::default::Default;
use rocket::Route;
use rocket::request::{FlashMessage, Form};
use rocket::response::{NamedFile, Redirect};
use maud::{html, PreEscaped};
use chrono::offset::Utc;

use db;
use db::models::{Account, User, NewStatus};
use templates::Page;
use failure::Error;
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
        create_status,
        static_files
    ]
}

#[derive(Debug, FromForm)]
pub struct CreateStatusForm {
    content: String,
}

#[post("/statuses/create", data = "<form>")]
pub fn create_status(
    user: User,
    db_conn: db::Connection,
    form: Form<CreateStatusForm>,
) -> Result<Redirect, Error> {
    let form_data = form.get();

    let status = NewStatus {
        created_at: Utc::now(),
        text: form_data.content.to_owned(),
        content_warning: None,
        account_id: user.account_id,
    }.insert(&db_conn)?;

    Ok(Redirect::to("/"))
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

            section.statuses {
                header h2 "Posts"
                @for status in account.recent_statuses(&db_conn, 10)? {
                    div.status {
                        header {
                            span {
                                ("published at ")
                                time datetime=(status.created_at.to_rfc3339())
                                    (status.created_at.format("%H:%M %d %a %b %y"))
                            }
                        }
                        div.content (status.text)
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
                div {
                    a href="/auth/sign_in" "sign in!"
                    " | "
                    a href="/auth/sign_up" "sign up?"
                }
            } @else {
                div {
                    form.inline method="post" action="/auth/sign_out" {
                        input type="hidden" name="stub"
                        button.link type="submit" name="submit" "sign out."
                    }
                }

                form method="post" action="/statuses/create" {
                    div textarea name="content" {}

                    button type="submit" "post"
                }
            }
        }
    })
}

#[get("/static/<path..>")]
fn static_files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}
