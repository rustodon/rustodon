use crate::routes::ui::view_helpers::*;
use askama::Template;
use db::models::{Account, Status};
use rocket::request::FlashMessage;

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
pub struct StatusTemplate<'a, 'b, 'c> {
    pub status: Status,
    pub account: Account,
    pub revision: &'a str,
    pub flash: Option<FlashMessage<'b, 'c>>,
    pub connection: db::Connection,
}

#[derive(Template)]
#[template(path = "user.html")]
pub struct UserTemplate<'a, 'b, 'c> {
    pub account_to_show: Account,
    pub account: Option<Account>,
    pub statuses: Vec<Status>,
    pub prev_page_id: Option<i64>,
    pub connection: db::Connection,
    pub revision: &'a str,
    pub flash: Option<FlashMessage<'b, 'c>>,
}

#[derive(Template)]
#[template(path = "edit_profile.html")]
pub struct EditProfileTemplate<'a, 'b, 'c> {
    pub account:  Account,
    pub revision: &'a str,
    pub flash:    Option<FlashMessage<'b, 'c>>,
}

#[derive(Template)]
#[template(path = "signin.html")]
pub struct SigninTemplate<'a, 'b, 'c> {
    pub revision: &'a str,
    pub flash:    Option<FlashMessage<'b, 'c>>,
}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignupTemplate<'a, 'b, 'c> {
    pub revision: &'a str,
    pub flash:    Option<FlashMessage<'b, 'c>>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a, 'b, 'c> {
    pub account: Option<Account>,
    pub statuses: Vec<Status>,
    pub timeline: &'a str,
    pub prev_page_id: Option<i64>,
    pub connection: db::Connection,
    pub revision: &'a str,
    pub flash: Option<FlashMessage<'b, 'c>>,
}
