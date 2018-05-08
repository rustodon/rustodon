//! Database models.
//!
//! Note: do _not_ change the ordering of fields in these structs!
//! The ordering must match that in the generated schema, which
//! you can obtain with `diesel print-schema`.

use chrono::offset::Utc;
use chrono::DateTime;
use chrono_humanize::Humanize;
use diesel;
use diesel::prelude::*;
use flaken::Flaken;
use pwhash::bcrypt;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use sanitize;
use schema::{accounts, follows, statuses, users};
use std::borrow::Cow;
use std::cell::Cell;
use std::sync::Mutex;
use Connection;
use {BASE_URL, DOMAIN};

pub struct IdGenerator {
    flaken: Flaken,
}

/// Constructs an IdGenerator, which can be used to provide one or more snowflake IDs
/// for a database transaction.
///
/// Example use:
///
/// ```
/// # use rustodon_database::models::id_generator;
/// # struct ModelA { id: i64 }
/// # struct ModelB { id: i64 }
/// let mut id_gen = id_generator();
///
/// let modelA = ModelA {
///     id: id_gen.next(),
/// };
///
/// let modelB = ModelB {
///     id: id_gen.next(),
/// };
/// ```
pub fn id_generator() -> IdGenerator {
    IdGenerator {
        flaken: Flaken::default().node(node_id()),
    }
}

lazy_static! {
    static ref THREAD_COUNTER: Mutex<u64> = Mutex::new(0);
}

thread_local! {
    static THREAD_ID: Cell<u64> = Cell::new(0);
}

/// Generates a node ID for the IdGenerator.
fn node_id() -> u64 {
    THREAD_ID.with(|f| {
        let mut g = THREAD_COUNTER.lock().unwrap();
        *g += 1;
        f.set(*g);
        f.get()
    })
}

impl IdGenerator {
    pub fn next(&mut self) -> i64 {
        self.flaken.next() as i64
    }
}

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
    pub id: i64,
    pub email: String,
    pub encrypted_password: String,

    pub account_id: i64,
}

/// Represents a new account for insertion into the database.
#[derive(Insertable, Debug)]
#[table_name = "accounts"]
pub struct NewAccount {
    pub id: i64,
    pub uri: Option<String>,
    pub domain: Option<String>,

    pub username: String,

    pub display_name: Option<String>,
    pub summary: Option<String>,
}

/// Represents a new status for insertion into the database.
#[derive(Insertable, Debug)]
#[table_name = "statuses"]
pub struct NewStatus {
    pub id: i64,
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
    pub fn insert(self, conn: &Connection) -> QueryResult<usize> {
        use super::schema::accounts::dsl::*;

        diesel::insert_into(accounts)
            .values(&self)
            .execute(&**conn)
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

    /// Returns the hash of a plaintext password, for storage in the database.
    pub fn encrypt_password<S>(password: S) -> String
    where
        S: AsRef<str>,
    {
        bcrypt::hash(password.as_ref()).expect("Couldn't hash password!")
    }

    /// Finds a local user by their username, returning an `Option<User>`.
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

    /// Finds a local user by their ID, returning an `Option<User>`.
    pub fn by_id(db_conn: &Connection, uid: i64) -> QueryResult<Option<User>> {
        use super::schema::users::dsl::*;

        users.find(uid).first(&**db_conn).optional()
    }

    /// Returns the corresponding `Account` of a local user.
    ///
    /// Note: panics if the account does not exist. This _will_ be caught by
    /// Rocket, but this _should be_ an irrecoverable error - there's no concievable
    /// circumstance outside of horrible database meddling that would cause this.
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
        use rocket::http::Status;
        use rocket::Outcome;

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
    /// Finds a local account by username, returning an `Option<Account>`.
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

    /// Returns the fully-qualified (`@user@domain`) username of an account.
    pub fn fully_qualified_username(&self) -> String {
        format!(
            "@{user}@{domain}",
            user = self.username,
            domain = self.get_domain()
        )
    }

    /// Returns the domain on which an account resides.
    pub fn get_domain(&self) -> &str {
        self.domain
            .as_ref()
            .map(String::as_str)
            .unwrap_or_else(|| DOMAIN.as_str())
    }

    /// Returns the URI of the account's ActivityPub object.
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

    /// Returns the URI of the ActivityPub `inbox` endpoint for this account.
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

    /// Returns the URI of the ActivityPub `outbox` endpoint for this account.
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

    /// Returns the URI of the ActivityPub `following` endpoint for this account.
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

    /// Returns the URI of the ActivityPub `followers` endpoint for this account.
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

    /// Returns `n` statuses authored by this account, authored
    // _strictly before_ the status `max_id`.
    pub fn statuses_before_id(
        &self,
        db_conn: &Connection,
        max_id: Option<i64>,
        n: usize,
    ) -> QueryResult<Vec<Status>> {
        use super::schema::statuses::dsl::*;
        let mut query = statuses.filter(account_id.eq(self.id)).into_boxed();

        if let Some(max_id) = max_id {
            query = query.filter(id.lt(max_id))
        }

        query
            .order(id.desc())
            .limit(n as i64)
            .get_results::<Status>(&**db_conn)
    }

    /// Returns a tuple of upper and lower bounds on the IDs of statuses authored by this account
    /// (i.e., `min(ids)` and `max(ids)` where `ids` is a list of status ids authored by this user).
    ///
    /// If this account has no statuses attached to it in the database, return `None`.
    pub fn status_id_bounds(&self, db_conn: &Connection) -> QueryResult<Option<(i64, i64)>> {
        use super::schema::statuses::dsl::*;
        use diesel::dsl::sql;
        // Yes, this is gross and we don't like having to use sql() either.
        // See [diesel-rs/diesel#3](https://github.com/diesel-rs/diesel/issues/3) for why this is necessary.
        statuses
            .select(sql("min(id), max(id)"))
            .filter(account_id.eq(self.id))
            .first::<(Option<i64>, Option<i64>)>(&**db_conn)
            .map(|result| match result {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            })
    }

    pub fn set_summary(
        &self,
        db_conn: &Connection,
        new_summary: Option<String>,
    ) -> QueryResult<()> {
        use super::schema::accounts::dsl::summary;

        diesel::update(self)
            .set(summary.eq(new_summary))
            .execute(&**db_conn)
            .and(Ok(()))
    }

    pub fn safe_summary(&self) -> Option<String> {
        self.summary.as_ref().map(sanitize::summary)
    }
}

impl Status {
    /// Returns the `Account` which authored this status.
    pub fn account(&self, db_conn: &Connection) -> QueryResult<Account> {
        use super::schema::accounts::dsl;
        dsl::accounts
            .find(self.account_id)
            .first::<Account>(&**db_conn)
    }

    /// Returns an optional status given an account ID and a status ID.
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

    /// Returns a human-readble description of the age of this status.
    pub fn humanized_age(&self) -> String {
        self.created_at.humanize()
    }

    /// Retunrs a URI to the ActivityPub object of this status.
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
