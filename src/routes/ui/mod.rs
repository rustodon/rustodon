use chrono::offset::Utc;
use maud::{html, PreEscaped};
use rocket::request::{FlashMessage, Form};
use rocket::response::{NamedFile, Redirect};
use rocket::Route;
use std::path::{Path, PathBuf};

use db;
use db::models::{Account, NewStatus, Status, User};
use error::Perhaps;
use failure::Error;
use templates::Page;

mod auth;

pub fn routes() -> Vec<Route> {
    routes![
        index,
        user_page,
        user_page_paginated,
        status_page,
        create_status,
        auth::signin_get,
        auth::signin_post,
        auth::signout,
        auth::signup_get,
        auth::signup_post,
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

    let _status = NewStatus {
        created_at: Utc::now(),
        text: form_data.content.to_owned(),
        content_warning: None,
        account_id: user.account_id,
    }.insert(&db_conn)?;

    Ok(Redirect::to("/"))
}

#[get("/users/<username>/statuses/<status_id>", format = "text/html")]
pub fn status_page(username: String, status_id: u64, db_conn: db::Connection) -> Perhaps<Page> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));
    let status = try_resopt!(Status::by_account_and_id(
        &db_conn,
        account.id,
        status_id as i64
    ));

    let rendered = Page::new()
        .title(format!(
            "@{user}: {id}",
            user = account.username,
            id = status.id
        ))
        .content(html! {
            div.status {
                header {
                    span {
                        ("published: ")
                        time datetime=(status.created_at.to_rfc3339()) (status.humanized_age())
                    }
                    div.content (status.text)
                }
            }
        });

    Ok(Some(rendered))
}

#[derive(FromForm, Debug)]
pub struct UserPageParams {
    max_id: Option<i64>,
}

// This is due to [SergioBenitez/Rocket#376](https://github.com/SergioBenitez/Rocket/issues/376).
// If you don't like this, please complain over there.
#[get("/users/<username>", format = "text/html")]
pub fn user_page(username: String, db_conn: db::Connection) -> Perhaps<Page> {
    user_page_paginated(username, UserPageParams { max_id: None }, db_conn)
}

#[get("/users/<username>?<params>", format = "text/html")]
pub fn user_page_paginated(
    username: String,
    params: UserPageParams,
    db_conn: db::Connection,
) -> Perhaps<Page> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));
    let statuses: Vec<Status> = account.statuses_before_id(&db_conn, params.max_id, 10)?;

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

                @for status in &statuses {
                    div.status {
                        header {
                            a href=(status.get_uri(&db_conn)?) { span {
                                ("published: ")
                                time datetime=(status.created_at.to_rfc3339())
                                    (status.humanized_age())
                            }}
                        }
                        div.content (status.text)
                    }
                }

                nav.pagination {
                    @if let Some(prev_page_max_id) = statuses.iter().map(|s| s.id).min() {
                        @let bounds = account.status_id_bounds(&db_conn)?;
                        // unwrap is safe since we already know we have statuses
                        @if prev_page_max_id > bounds.unwrap().0 {
                            a href=(format!("?max_id={}", prev_page_max_id)) rel="next" "⇐ older posts"
                        }
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
