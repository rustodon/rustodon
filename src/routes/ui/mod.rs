use chrono::offset::Utc;
use maud::{html, Markup, PreEscaped};
use rocket::request::{FlashMessage, Form};
use rocket::response::{NamedFile, Redirect};
use rocket::Route;
use std::path::{Path, PathBuf};

use db::datetime::{DateTimeType, NewDateTime, Rfc339able};
use db::models::{Account, NewStatus, Status, User};
use db::{self, id_generator};
use error::Perhaps;
use failure::Error;
use templates::Page;
use transform;

mod auth;

pub fn routes() -> Vec<Route> {
    routes![
        index,
        user_page,
        user_page_paginated,
        settings_profile,
        settings_profile_update,
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
    content_warning: String,
}

#[post("/statuses/create", data = "<form>")]
pub fn create_status(
    user: User,
    db_conn: db::Connection,
    form: Form<CreateStatusForm>,
) -> Result<Redirect, Error> {
    let form_data = form.get();

    // convert CW to option if present, so we get proper nulls in DB
    let content_warning: Option<String> = if form_data.content_warning.len() > 0 {
        Some(form_data.content_warning.to_owned())
    } else {
        None
    };

    let _status = NewStatus {
        id: id_generator().next(),
        created_at: DateTimeType::now(),
        text: form_data.content.to_owned(),
        content_warning: content_warning,
        account_id: user.account_id,
    }.insert(&db_conn)?;

    Ok(Redirect::to("/"))
}

fn render_status(db_conn: &db::Connection, status: &Status, link: bool) -> Result<Markup, Error> {
    let meta_line = html !{
        span {
            ("published: ")
            time datetime=(status.created_at.to_rfc3339()) (status.humanized_age())
        }
    };

    let rendered = html! {
        div.status {
            header {
                @if link {
                    a href=(status.get_uri(db_conn)?) (meta_line)
                } @else {
                    (meta_line)
                }
            }
            div {
                @if let Some(cw) = &status.content_warning {
                    @let collapse_id = format!("collapsible-{}", status.id);

                    span (cw)
                    input.collapse--toggle id=(collapse_id) type="checkbox";
                    label.collapse--lbl-toggle for=(collapse_id) tabindex="0" { "Toggle CW" }
                    div.content.collapse--content (status.text)
                } @else {
                    div.content (status.text)
                }
            }
        }
    };

    Ok(rendered)
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
        .content(render_status(&db_conn, &status, false)?);

    Ok(Some(rendered))
}

#[derive(FromForm, Debug)]
pub struct UserPageParams {
    max_id: Option<i64>,
}

// This is due to [SergioBenitez/Rocket#376](https://github.com/SergioBenitez/Rocket/issues/376).
// If you don't like this, please complain over there.
#[get("/users/<username>", format = "text/html")]
pub fn user_page(username: String, db_conn: db::Connection, user: Option<User>) -> Perhaps<Page> {
    user_page_paginated(username, UserPageParams { max_id: None }, db_conn, user)
}

#[get("/users/<username>?<params>", format = "text/html")]
pub fn user_page_paginated(
    username: String,
    params: UserPageParams,
    db_conn: db::Connection,
    user: Option<User>,
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
                    @if let Some(raw_bio) = account.summary.as_ref().map(String::as_str)
                    {
                        (PreEscaped(transform::bio(raw_bio, &db_conn)?))
                    } @else {
                        p {}
                    }
                }

                @if let Some(user) = user {
                    @if user.get_account(&db_conn)? == account {
                        div.action-edit-note {
                            "(" a href="/settings/profile" "edit" ")"
                        }
                    }
                }
            }

            section.statuses {
                header h2 "Posts"

                @for status in &statuses {
                    (render_status(&db_conn, status, true)?)
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

#[get("/settings/profile")]
pub fn settings_profile(db_conn: db::Connection, user: User) -> Result<Page, Error> {
    let account = user.get_account(&db_conn)?;

    let rendered = Page::new().title("edit your profile").content(html! {
        header h2 "Edit your profile"

        form method="post" action="/settings/profile" {
            div textarea name="summary" {(account.summary.as_ref().map(String::as_ref).unwrap_or(""))}

            button type="submit" "update"
        }
    });

    Ok(rendered)
}

#[derive(Debug, FromForm)]
pub struct UpdateProfileForm {
    summary: String,
}

#[post("/settings/profile", data = "<form>")]
pub fn settings_profile_update(
    db_conn: db::Connection,
    user: User,
    form: Form<UpdateProfileForm>,
) -> Result<Redirect, Error> {
    let form_data = form.get();
    let account = user.get_account(&db_conn)?;

    // `as &str` defeat an incorrect deref coercion (due to the second match arm)
    let new_summary = match &form_data.summary as &str {
        "" => None,
        x => Some(x.to_string()),
    };
    account.set_summary(&db_conn, new_summary)?;

    Ok(Redirect::to("/settings/profile"))
}

#[get("/")]
pub fn index(
    flash: Option<FlashMessage>,
    user: Option<User>,
    db_conn: db::Connection,
) -> Result<Page, Error> {
    let rendered = Page::new().flash(flash).content(html! {
        header h1 "Rustodon"

        div {
            @if let Some(user) = user {
                @let account = user.get_account(&db_conn)?;
                div {
                    a href=(account.get_uri()) "your profile"
                    " | "
                    form.inline method="post" action="/auth/sign_out" {
                        input type="hidden" name="stub"
                        button.link type="submit" name="submit" "sign out."
                    }
                }

                form method="post" action="/statuses/create" {
                    div input name="content_warning" placeholder="content warning" {}
                    div textarea name="content" {}

                    button type="submit" "post"
                }
            } @else {
                div {
                    a href="/auth/sign_in" "sign in!"
                    " | "
                    a href="/auth/sign_up" "sign up?"
                }
            }
        }
    });

    Ok(rendered)
}

#[get("/static/<path..>")]
fn static_files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}
