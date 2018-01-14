//! Database models.
//!
//! Note: do _not_ change the ordering of fields in these structs!
//! The ordering must match that in the generated schema, which
//! you can obtain with `diesel print-schema`.

use std::borrow::Cow;
use chrono::DateTime;
use chrono::offset::Utc;
use diesel::prelude::*;
use pwhash::bcrypt;
use db::schema::{accounts, follows, statuses, users};
use db::Connection;
use diesel::result::Error as DieselError;
use {BASE_URL, DOMAIN};

/// Type representing the result of a database operation.
///
/// The nested `Option<T>` in the `Result` allows a semantic distinction bewteen "query error"
/// and "no records returned".
type DatabaseResult<T> = Result<Option<T>, DieselError>;

/// Represents an account (local _or_ remote) on the network, storing federation-relevant information.
///
/// A uri of None implies a local account.
#[derive(Identifiable, Queryable, Debug, Serialize, PartialEq)]
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
#[derive(Identifiable, Queryable, Associations, PartialEq, Serialize, Debug)]
#[belongs_to(Account)]
#[table_name = "users"]
pub struct User {
    pub id: i64,
    pub email: String,
    pub encrypted_password: String,

    account_id: i64,
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

impl User {
    /// Checks if a plaintext password is valid.
    pub fn valid_password<S>(&self, password: S) -> bool
    where
        S: Into<String>,
    {
        bcrypt::verify(&self.encrypted_password, &password.into())
    }

    /// Hashes a plaintext password for storage in the database.
    pub fn encrypt_password<S>(password: S) -> String
    where
        S: Into<String>,
    {
        bcrypt::hash(&password.into()).expect("Couldn't hash password!")
    }

    pub fn by_username(db_conn: &Connection, username: String) -> DatabaseResult<User> {
        let account = try_resopt!({
            use db::schema::accounts::dsl;
            dsl::accounts
                .filter(dsl::username.eq(username))
                .filter(dsl::domain.is_null())
                .first::<Account>(&**db_conn)
                .optional()
        });

        use db::schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::account_id.eq(account.id))
            .first::<User>(&**db_conn)
            .optional()?)
    }
}

impl Account {
    // TODO: result
    pub fn fetch_local_by_username<S>(db_conn: &Connection, username: S) -> DatabaseResult<Account>
    where
        S: Into<String>,
    {
        use db::schema::accounts::dsl;
        Ok(dsl::accounts
            .filter(dsl::username.eq(username.into()))
            .filter(dsl::domain.is_null())
            .first::<Account>(&**db_conn)
            .optional()?)
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

    // TODO: gross, should probably clean up sometime
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
}

impl Status {
    pub fn account(&self, db_conn: &Connection) -> DatabaseResult<Account> {
        use db::schema::accounts::dsl;
        dsl::accounts
            .find(self.account_id)
            .first::<Account>(&**db_conn)
            .optional()
    }

    pub fn get_uri<'a>(&'a self, db_conn: &Connection) -> DatabaseResult<Cow<'a, str>> {
        Ok(Some(
            self.uri
                .as_ref()
                .map(|x| String::as_str(x).into())
                .unwrap_or(
                    format!(
                        "{base}/users/{user}/updates/{id}",
                        base = BASE_URL.as_str(),
                        user = try_resopt!(self.account(db_conn)).username,
                        id = self.id
                    ).into(),
                ),
        ))
    }
}
