//! Database models.
//!
//! Note: do _not_ change the ordering of fields in these structs!
//! The ordering must match that in the generated schema, which
//! you can obtain with `diesel print-schema`.

use std::borrow::Cow;
use chrono::DateTime;
use chrono::offset::Utc;
use diesel;
use diesel::prelude::*;
use pwhash::bcrypt;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use super::schema::{accounts, follows, statuses, users};
use super::Connection;
use {BASE_URL, DOMAIN};

/// Represents an account (local _or_ remote) on the network, storing federation-relevant information.
///
/// A uri of None implies a local account.
#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "accounts"]
pub struct Account {
    pub id: i64,
    pub uri: Option<String>,
    pub domain: Option<String>,
    pub username: String,

    pub display_name: Option<String>,
    pub summary: Option<String>,
}

/// Represents a local user, and information required to authenticate that user.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Account)]
#[table_name = "users"]
pub struct User {
    pub id: i64,
    pub email: String,
    pub encrypted_password: String,

    pub account_id: i64,
}

/// Represents a post.
///
/// A uri of None implies a local status.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Account)]
#[table_name = "statuses"]
pub struct Status {
    pub id: i64,
    pub text: String,
    pub content_warning: Option<String>,
    pub created_at: DateTime<Utc>,
    pub account_id: i64,
    pub uri: Option<String>,
}

/// Represents a following relationship `[source user] -> [target user]`.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name = "follows"]
pub struct Follow {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
}

/// Represents a new user for insertion into the database.
#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub encrypted_password: String,

    pub account_id: i64,
}

/// Represents a new account for insertion into the database.
#[derive(Insertable, Debug)]
#[table_name = "accounts"]
pub struct NewAccount {
    pub uri:    Option<String>,
    pub domain: Option<String>,

    pub username: String,

    pub display_name: Option<String>,
    pub summary: Option<String>,
}

/// Represents a new status for insertion into the database.
#[derive(Insertable, Debug)]
#[table_name = "statuses"]
pub struct NewStatus {
    pub text: String,
    pub content_warning: Option<String>,
    pub created_at: DateTime<Utc>,
    pub account_id: i64,
}

impl NewUser {
    pub fn insert(self, conn: &Connection) -> QueryResult<User> {
        use super::schema::users::dsl::*;

        diesel::insert_into(users).values(&self).get_result(&**conn)
    }
}

impl NewAccount {
    pub fn insert(self, conn: &Connection) -> QueryResult<Account> {
        use super::schema::accounts::dsl::*;

        diesel::insert_into(accounts)
            .values(&self)
            .get_result(&**conn)
    }
}

impl NewStatus {
    pub fn insert(self, conn: &Connection) -> QueryResult<Status> {
        use super::schema::statuses::dsl::*;

        diesel::insert_into(statuses)
            .values(&self)
            .get_result(&**conn)
    }
}

impl User {
    /// Checks if a plaintext password is valid.
    pub fn valid_password<S>(&self, password: S) -> bool
    where
        S: AsRef<str>,
    {
        bcrypt::verify(password.as_ref(), &self.encrypted_password)
    }

    /// Hashes a plaintext password for storage in the database.
    pub fn encrypt_password<S>(password: S) -> String
    where
        S: AsRef<str>,
    {
        bcrypt::hash(password.as_ref()).expect("Couldn't hash password!")
    }

    pub fn by_username<S>(db_conn: &Connection, username: S) -> QueryResult<Option<User>>
    where
        S: AsRef<str>,
    {
        let account = try_resopt!({
            use super::schema::accounts::dsl;
            dsl::accounts
                .filter(dsl::username.eq(username.as_ref()))
                .filter(dsl::domain.is_null())
                .first::<Account>(&**db_conn)
                .optional()
        });

        use super::schema::users::dsl;
        dsl::users
            .filter(dsl::account_id.eq(account.id))
            .first::<User>(&**db_conn)
            .optional()
    }

    pub fn by_id(db_conn: &Connection, uid: i64) -> QueryResult<Option<User>> {
        use super::schema::users::dsl::*;

        users.find(uid).first(&**db_conn).optional()
    }

    pub fn get_account(self, db_conn: &Connection) -> QueryResult<Account> {
        use super::schema::accounts::dsl::*;

        accounts
            .find(self.account_id)
            .first(&**db_conn)
            .optional()
            .map(|x| x.expect("All users should have an account!"))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        use rocket::Outcome;
        use rocket::http::Status;

        let db_conn = request.guard::<Connection>()?;

        let uid = request
            .cookies()
            .get_private("uid")
            .and_then(|cookie| cookie.value().parse::<i64>().ok())
            .or_forward(())?;

        match User::by_id(&db_conn, uid) {
            Ok(Some(user)) => Outcome::Success(user),
            Ok(None) => Outcome::Forward(()),
            Err(_) => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}

impl Account {
    pub fn fetch_local_by_username<S>(
        db_conn: &Connection,
        username: S,
    ) -> QueryResult<Option<Account>>
    where
        S: Into<String>,
    {
        use super::schema::accounts::dsl;
        dsl::accounts
            .filter(dsl::username.eq(username.into()))
            .filter(dsl::domain.is_null())
            .first::<Account>(&**db_conn)
            .optional()
    }

    pub fn fully_qualified_username(&self) -> String {
        format!(
            "@{user}@{domain}",
            user = self.username,
            domain = self.get_domain()
        )
    }

    pub fn get_domain(&self) -> &str {
        self.domain
            .as_ref()
            .map(String::as_str)
            .unwrap_or_else(|| DOMAIN.as_str())
    }

    pub fn get_uri<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}/users/{user}",
                    base = BASE_URL.as_str(),
                    user = self.username
                ).into()
            })
    }

    pub fn get_inbox_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}/users/{user}/inbox",
                    base = BASE_URL.as_str(),
                    user = self.username
                ).into()
            })
    }

    pub fn get_outbox_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}/users/{user}/outbox",
                    base = BASE_URL.as_str(),
                    user = self.username
                ).into()
            })
    }

    pub fn get_following_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}/users/{user}/following",
                    base = BASE_URL.as_str(),
                    user = self.username
                ).into()
            })
    }

    pub fn get_followers_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}/users/{user}/followers",
                    base = BASE_URL.as_str(),
                    user = self.username
                ).into()
            })
    }

    pub fn statuses_before_id(
        &self,
        db_conn: &Connection,
        max_id: Option<i64>,
        n: usize,
    ) -> QueryResult<Vec<Status>> {
        use super::schema::statuses::dsl::*;
        let mut query = statuses
            .filter(account_id.eq(self.id)).into_boxed();

        if let Some(max_id) = max_id {
            query = query.filter(id.lt(max_id))
        }

        query.order(id.desc())
            .limit(n as i64)
            .get_results::<Status>(&**db_conn)
    }
}

impl Status {
    pub fn account(&self, db_conn: &Connection) -> QueryResult<Account> {
        use super::schema::accounts::dsl;
        dsl::accounts
            .find(self.account_id)
            .first::<Account>(&**db_conn)
    }

    pub fn by_account_and_id(
        db_conn: &Connection,
        account_id: i64,
        id: i64,
    ) -> QueryResult<Option<Status>> {
        use super::schema::statuses::dsl;
        dsl::statuses
            .find(id)
            .filter(dsl::account_id.eq(account_id))
            .first::<Status>(&**db_conn)
            .optional()
    }

    pub fn get_uri<'a>(&'a self, db_conn: &Connection) -> QueryResult<Cow<'a, str>> {
        let uri = self.uri.as_ref().map(|x| String::as_str(x).into());
        match uri {
            Some(x) => Ok(x),
            None => {
                return Ok(format!(
                    "{base}/users/{user}/statuses/{id}",
                    base = BASE_URL.as_str(),
                    user = self.account(db_conn)?.username,
                    id = self.id
                ).into())
            },
        }
    }
}

pub mod validators {
    use regex::Regex;

    lazy_static! {
        pub static ref VALID_USERNAME_RE: Regex = Regex::new(r"^[[:alnum:]_]+$").unwrap();
    }
}
