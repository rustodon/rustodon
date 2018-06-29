use askama::Template;
use db;
use db::models::{Account, Status};
use rocket::request::FlashMessage;
use routes::ui::view_helpers::*;

macro_rules! HtmlTemplate {
    ($x:tt) => {{
        $x {
            flash: None,
            revision: $crate::GIT_REV,
        }
    }};

    ($x:tt, $flash: ident) => {{
        $x {
            flash: $flash,
            revision: $crate::GIT_REV,
        }
    }};

    ($x:tt, { $( $y:ident : $z:expr ),* }) => {{
        $x {
            $( $y: $z ),*
            ,flash: None
            ,revision: $crate::GIT_REV,
        }
    }};
    ($x:tt, $flash: ident, { $( $y:ident : $z:expr ),* }) => {{
        $x {
            $( $y: $z ),*
            ,flash: $flash
            ,revision: $crate::GIT_REV,
        }
    }};
}

macro_rules! PerhapsHtmlTemplate {
    ($($x:tt)*) => {{
        Ok(Some(HtmlTemplate!($($x)*)))
    }};
}

#[derive(Template)]
#[template(path = "status.html")]
pub struct StatusTemplate<'a> {
    pub status:   Status,
    pub account:  Account,
    pub revision: &'a str,
    pub flash:    Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "user.html")]
pub struct UserTemplate<'a> {
    pub account_to_show: Account,
    pub account: Option<Account>,
    pub statuses: Vec<Status>,
    pub prev_page_id: Option<i64>,
    pub connection: db::Connection,
    pub revision: &'a str,
    pub flash: Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "edit_profile.html")]
pub struct EditProfileTemplate<'a> {
    pub account:  Account,
    pub revision: &'a str,
    pub flash:    Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "signin.html")]
pub struct SigninTemplate<'a> {
    pub revision: &'a str,
    pub flash:    Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignupTemplate<'a> {
    pub revision: &'a str,
    pub flash:    Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub account:  Option<Account>,
    pub local_statuses: Vec<Status>,
    pub prev_page_id: Option<i64>,
    pub connection: db::Connection,
    pub revision: &'a str,
    pub flash:    Option<FlashMessage>,
}
