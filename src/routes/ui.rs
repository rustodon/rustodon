use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::borrow::Cow;
use rocket::Route;
use rocket::http::{Cookie, Cookies};
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, NamedFile, Redirect};
use maud::{html, PreEscaped};
use validator::Validate;

use db;
use db::models::{Account, User};
use templates::Page;
use failure::Error;
use error::Perhaps;

pub fn routes() -> Vec<Route> {
    routes![
        index,
        user_page,
        auth_signin_get,
        auth_signin_post,
        auth_signout,
        auth_signup_get,
        auth_signup_post,
        static_files
    ]
}

#[get("/auth/sign_in")]
pub fn auth_signin_get(flash: Option<FlashMessage>) -> Page {
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
pub fn auth_signin_post(
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
pub fn auth_signout(user: Option<User>, mut cookies: Cookies) -> Redirect {
    if user.is_some() {
        cookies.remove_private(Cookie::named("uid"));
    }

    Redirect::to("/")
}

#[derive(FromForm, Validate, Debug)]
pub struct SignupForm {
    #[validate(length(min = "1", max = "32"))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = "3", max = "64",
                      message = "Password must be between 3 and 64 characters long."))]
    password: String,
}

#[get("/auth/sign_up")]
pub fn auth_signup_get(flash: Option<FlashMessage>) -> Page {
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
pub fn auth_signup_post(
    form: Form<SignupForm>,
    db_conn: db::Connection,
) -> Result<Flash<Redirect>, Error> {
    let form_data = form.get();

    let signup_data = match form_data.validate() {
        Ok(v) => v,
        Err(errs) => {
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
        },
    };

    Ok(Flash::success(Redirect::to("/"), "signed up!"))
}

#[get("/users/<username>", format = "text/html")]
pub fn user_page(username: String, db_conn: db::Connection) -> Perhaps<Page> {
    let account = try_resopt!(Account::fetch_local_by_username(&db_conn, username));

    let rendered = Page::new()
        .title(format!("@{user}", user = account.username))
        .content(html! {
            div.h-card {
                header {
                    h1 a.u-url.u-uid href=(account.get_uri()) {
                        span.p-name (account.display_name.as_ref().unwrap_or(&account.username))
                    }

                    div (account.fully_qualified_username())
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

        p {"Current user session: " code (format!("{:?}", user))}
    })
}

#[get("/static/<path..>")]
fn static_files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}
