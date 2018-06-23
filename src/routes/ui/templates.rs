use askama::Template;
use db;
use db::models::{Account, Status};
use rocket::request::FlashMessage;
use routes::ui::view_helpers::*;
use std::ops::Deref;

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<'a> {
    pub revision: &'a str,
    pub flash:    Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "status.html")]
pub struct StatusTemplate<'a> {
    pub status:  Status,
    pub link:    bool,
    pub account: Account,
    pub _parent: BaseTemplate<'a>,
}

#[derive(Template)]
#[template(path = "user.html")]
pub struct UserTemplate<'a> {
    pub account_to_show: Account,
    pub account: Option<Account>,
    pub statuses: Vec<Status>,
    pub prev_page_id: Option<i64>,
    pub connection: db::Connection,
    pub link: bool,
    pub _parent: BaseTemplate<'a>,
}

#[derive(Template)]
#[template(path = "edit_profile.html")]
pub struct EditProfileTemplate<'a> {
    pub account: Account,
    pub _parent: BaseTemplate<'a>,
}

#[derive(Template)]
#[template(path = "signin.html")]
pub struct SigninTemplate<'a> {
    pub _parent: BaseTemplate<'a>,
}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignupTemplate<'a> {
    pub _parent: BaseTemplate<'a>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub account: Option<Account>,
    pub _parent: BaseTemplate<'a>,
}
