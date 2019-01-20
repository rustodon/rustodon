use crate::db::models::{NewAccount, NewUser, User};
use crate::db::validators;
use crate::db::{self, id_generator, LOCAL_ACCOUNT_DOMAIN};
use crate::routes::ui::templates::{SigninTemplate, SignupTemplate};
use diesel::Connection;
use failure::Error;
use itertools::Itertools;
use rocket::http::{Cookie, Cookies};
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use std::borrow::Cow;
use validator::Validate;

#[get("/auth/sign_in")]
pub fn signin_get<'b, 'c>(flash: Option<FlashMessage<'b, 'c>>) -> SigninTemplate<'static, 'b, 'c> {
    HtmlTemplate!(SigninTemplate, flash)
}

#[derive(Debug, FromForm)]
pub struct SigninForm {
    username: String,
    password: String,
}

#[post("/auth/sign_in", data = "<form>")]
pub fn signin_post(
    form: Form<SigninForm>,
    mut cookies: Cookies,
    db_conn: db::Connection,
) -> Result<Flash<Redirect>, Error> {
    let user = User::by_username(&db_conn, &form.username)?;

    if let Some(user) = user {
        if user.valid_password(&form.password) {
            cookies.add_private(Cookie::new("uid", user.id.to_string()));

            return Ok(Flash::success(Redirect::to("/"), "signed in!"));
        }
    }

    Ok(Flash::error(
        Redirect::to("/auth/sign_in"),
        "wrong password (or username!)",
    ))
}

#[post("/auth/sign_out")]
pub fn signout(user: Option<User>, mut cookies: Cookies) -> Redirect {
    if user.is_some() {
        cookies.remove_private(Cookie::named("uid"));
    }

    Redirect::to("/")
}

#[derive(FromForm, Validate, Debug)]
pub struct SignupForm {
    #[validate(
        length(
            min = "1",
            max = "32",
            message = "Username must be between 1 and 32 characters long."
        ),
        regex(
            path = "validators::VALID_USERNAME_RE",
            message = "Username must consist of {A-Z, a-z, 0-9, _}."
        )
    )]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(
        min = "3",
        max = "64",
        message = "Password must be between 3 and 64 characters long."
    ))]
    password: String,
}

#[get("/auth/sign_up")]
pub fn signup_get<'b, 'c>(flash: Option<FlashMessage<'b, 'c>>) -> SignupTemplate<'static, 'b, 'c> {
    HtmlTemplate!(SignupTemplate, flash)
}

#[post("/auth/sign_up", data = "<form>")]
pub fn signup_post(
    form: Form<SignupForm>,
    db_conn: db::Connection,
) -> Result<Flash<Redirect>, Error> {
    if let Err(errs) = form.validate() {
        let errs = errs.field_errors();

        // concatenate the error descriptions, with commas between them.
        // TODO: make this less ugly :(
        let error_desc = errs
            .iter()
            .flat_map(|(_, errs)| errs)
            .map(|e| {
                let msg = e.message.to_owned();
                msg.unwrap_or(Cow::Borrowed("unknown error"))
            })
            .join(", ");

        return Ok(Flash::error(Redirect::to("/auth/sign_up"), error_desc));
    }
    if let Ok(Some(_)) =
        db::models::Account::fetch_local_by_username(&db_conn, form.username.as_str())
    {
        return Ok(Flash::error(
            Redirect::to("/auth/sign_up"),
            "Username taken",
        ));
    }

    (*db_conn).transaction::<_, _, _>(|| {
        let mut id_gen = id_generator();
        let account = NewAccount {
            id: id_gen.next(),
            domain: Some(LOCAL_ACCOUNT_DOMAIN.to_string()),
            uri: None,

            username: form.username.to_owned(),

            display_name: None,
            summary: None,
        }
        .insert(&db_conn)?;

        NewUser {
            id: id_gen.next(),
            email: form.email.to_owned(),
            encrypted_password: User::encrypt_password(&form.password),
            account_id: account.id,
        }
        .insert(&db_conn)
    })?;

    Ok(Flash::success(Redirect::to("/"), "signed up!"))
}
