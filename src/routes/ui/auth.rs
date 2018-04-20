use failure::Error;
use itertools::Itertools;
use maud::html;
use rocket::http::{Cookie, Cookies};
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use std::borrow::Cow;
use validator::Validate;

use db::models::{id_generator, validators, NewAccount, NewUser, User};
use db::{self, DieselConnection};
use templates::Page;

#[get("/auth/sign_in")]
pub fn signin_get(flash: Option<FlashMessage>) -> Page {
    Page::new().title("sign in").flash(flash).content(html! {
        header h2 "sign in"

        form method="post" {
            div {
                label for="username" "username:"
                input type="text" id="username" name="username";
            }

            div {
                label for="password" "password:"
                input type="password" id="password" name="password";
            }

            button type="submit" "sign in"
        }
    })
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
    let form_data = form.get();
    let user = User::by_username(&db_conn, &form_data.username)?;

    if let Some(user) = user {
        if user.valid_password(&form_data.password) {
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
    #[validate(length(min = "1", max = "32"))]
    #[validate(
        regex(
            path = "validators::VALID_USERNAME_RE",
            message = "Username must consist of {A-Z, a-z, 0-9, _}."
        )
    )]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(
        length(
            min = "3", max = "64", message = "Password must be between 3 and 64 characters long."
        )
    )]
    password: String,
}

#[get("/auth/sign_up")]
pub fn signup_get(flash: Option<FlashMessage>) -> Page {
    Page::new().title("sign up").flash(flash).content(html! {
        header h2 "sign up"

        form method="post" {
            div {
                label for="username" "username:"
                input type="text" id="username" name="username";
            }

            div {
                label for="email" "email:"
                input type="email" id="email" name="email";
            }

            div {
                label for="password" "password:"
                input type="password" id="password" name="password";
            }

            button type="submit" "sign up"
        }
    })
}

#[post("/auth/sign_up", data = "<form>")]
pub fn signup_post(
    form: Form<SignupForm>,
    db_conn: db::Connection,
) -> Result<Flash<Redirect>, Error> {
    let form_data = form.get();

    if let Err(errs) = form_data.validate() {
        let errs = errs.inner();

        // concatenate the error descriptions, with commas between them.
        // TODO: make this less ugly :(
        let error_desc = errs.iter()
            .flat_map(|(_, errs)| errs)
            .map(|e| {
                let msg = e.message.to_owned();
                msg.unwrap_or(Cow::Borrowed("unknown error"))
            })
            .join(", ");

        return Ok(Flash::error(Redirect::to("/auth/sign_up"), error_desc));
    }

    (*db_conn).transaction::<_, _, _>(|| {
        let mut id_gen = id_generator();

        let account = NewAccount {
            id: id_gen.next(),
            domain: None,
            uri: None,

            username: form_data.username.to_owned(),

            display_name: None,
            summary: None,
        }.insert(&db_conn)?;

        NewUser {
            id: id_gen.next(),
            email: form_data.email.to_owned(),
            encrypted_password: User::encrypt_password(&form_data.password),
            account_id: account.id,
        }.insert(&db_conn)
    })?;

    Ok(Flash::success(Redirect::to("/"), "signed up!"))
}
