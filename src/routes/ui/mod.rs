use chrono::offset::Utc;
use db::models::{Account, NewStatus, Status, User};
use db::{self, id_generator};
use error::Perhaps;
use failure::Error;
use itertools::Itertools;
use rocket::http::RawStr;
use rocket::request::{FlashMessage, Form, FromFormValue};
use rocket::response::{Flash, NamedFile, Redirect};
use rocket::Route;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use util::Either;
use validator::Validate;

#[macro_use]
mod templates;
mod auth;
pub mod view_helpers;

use self::templates::*;

pub fn routes() -> Vec<Route> {
    routes![
        index,
        index_paginated,
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

#[derive(FromForm, Debug)]
pub struct UserPageParams {
    max_id: Option<i64>,
}

#[derive(Debug)]
pub enum Timeline {
    Local,
    Federated,
}

impl<'v> FromFormValue<'v> for Timeline {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        match form_value.as_str() {
            "local" => Ok(Timeline::Local),
            "federated" => Ok(Timeline::Federated),
            _ => Err(form_value),
        }
    }
}

#[derive(FromForm, Debug)]
pub struct IndexPageParams {
    max_id:   Option<i64>,
    timeline: Option<Timeline>,
}

#[derive(Debug, FromForm, Validate)]
pub struct CreateStatusForm {
    #[validate(length(min = "1", message = "Content must not be empty"))]
    content: String,
    content_warning: String,
}

#[post("/statuses/create", data = "<form>")]
pub fn create_status(
    user: User,
    db_conn: db::Connection,
    form: Form<CreateStatusForm>,
) -> Result<Either<Flash<Redirect>, Redirect>, Error> {
    let form_data = form.get();

    if let Err(errs) = form_data.validate() {
        let errs = errs.field_errors();

        // concatenate the error descriptions, with commas between them.
        // TODO: make this less ugly :(
        let error_desc = errs
            .iter()
            .flat_map(|(_, errs)| errs)
            .map(|e| {
                let msg = e.message.to_owned();
                msg.unwrap_or(Cow::Borrowed("unknown error"))
            }).join(", ");

        return Ok(Either::Left(Flash::error(Redirect::to("/"), error_desc)));
    }

    // convert CW to option if present, so we get proper nulls in DB
    let content_warning: Option<String> = if form_data.content_warning.len() > 0 {
        Some(form_data.content_warning.to_owned())
    } else {
        None
    };

    let _status = NewStatus {
        id: id_generator().next(),
        created_at: Utc::now(),
        text: form_data.content.to_owned(),
        content_warning: content_warning,
        account_id: user.account_id,
    }.insert(&db_conn)?;

    Ok(Either::Right(Redirect::to("/")))
}

#[get(
    "/users/<username>/statuses/<status_id>",
    format = "text/html"
)]
pub fn status_page(
    username: String,
    status_id: u64,
    db_conn: db::Connection,
) -> Perhaps<StatusTemplate<'static>> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));
    let status = try_resopt!(Status::by_account_and_id(
        &db_conn,
        account.id,
        status_id as i64
    ));

    PerhapsHtmlTemplate!(StatusTemplate, {
        status:  status,
        account: account,
        connection: db_conn
    })
}

// This is due to [SergioBenitez/Rocket#376](https://github.com/SergioBenitez/Rocket/issues/376).
// If you don't like this, please complain over there.
#[get("/users/<username>", format = "text/html")]
pub fn user_page(
    username: String,
    db_conn: db::Connection,
    account: Option<Account>,
) -> Perhaps<UserTemplate<'static>> {
    user_page_paginated(username, UserPageParams { max_id: None }, db_conn, account)
}

#[get("/users/<username>?<params>", format = "text/html")]
pub fn user_page_paginated(
    username: String,
    params: UserPageParams,
    db_conn: db::Connection,
    account: Option<Account>,
) -> Perhaps<UserTemplate<'static>> {
    let account_to_show = try_resopt!(Account::fetch_local_by_username(&db_conn, username));
    let statuses: Vec<Status> = account_to_show.statuses_before_id(&db_conn, params.max_id, 10)?;
    let prev_page_id = if let Some(prev_page_max_id) = statuses.iter().map(|s| s.id).min() {
        let bounds = account_to_show.status_id_bounds(&db_conn)?;
        // unwrap is safe since we already know we have statuses
        if prev_page_max_id > bounds.unwrap().0 {
            Some(prev_page_max_id)
        } else {
            None
        }
    } else {
        None
    };
    PerhapsHtmlTemplate!(UserTemplate, {
        account_to_show: account_to_show,
        account: account,
        statuses: statuses,
        prev_page_id: prev_page_id,
        connection: db_conn
    })
}

#[get("/settings/profile")]
pub fn settings_profile(
    db_conn: db::Connection,
    user: User,
) -> Perhaps<EditProfileTemplate<'static>> {
    PerhapsHtmlTemplate!(EditProfileTemplate, {
        account: user.get_account(&db_conn)?
    })
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
    account: Option<Account>,
    db_conn: db::Connection,
) -> Result<IndexTemplate<'static>, Error> {
    index_paginated(
        flash,
        account,
        IndexPageParams {
            max_id:   None,
            timeline: None,
        },
        db_conn,
    )
}

#[get("/?<params>")]
pub fn index_paginated(
    flash: Option<FlashMessage>,
    account: Option<Account>,
    params: IndexPageParams,
    db_conn: db::Connection,
) -> Result<IndexTemplate<'static>, Error> {
    let statuses: Vec<Status> = match params.timeline {
        Some(Timeline::Local) | None => Status::local_before_id(&db_conn, params.max_id, 10)?,
        Some(Timeline::Federated) => Status::federated_before_id(&db_conn, params.max_id, 10)?,
    };

    let prev_page_id = if let Some(prev_page_max_id) = statuses.iter().map(|s| s.id).min() {
        let bounds = match params.timeline {
            Some(Timeline::Local) | None => Status::local_status_id_bounds(&db_conn)?,
            Some(Timeline::Federated) => Status::federated_status_id_bounds(&db_conn)?,
        };
        // unwrap is safe since we already know we have statuses
        if prev_page_max_id > bounds.unwrap().0 {
            Some(prev_page_max_id)
        } else {
            None
        }
    } else {
        None
    };

    let timeline_str = match params.timeline {
        Some(Timeline::Local) | None => "local",
        Some(Timeline::Federated) => "federated",
    };

    Ok(HtmlTemplate!(IndexTemplate, flash, {
        account: account,
        statuses: statuses,
        timeline: timeline_str,
        prev_page_id: prev_page_id,
        connection: db_conn
    }))
}

#[get("/static/<path..>")]
fn static_files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}
