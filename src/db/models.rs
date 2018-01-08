//! Database models.
//!
//! Note: do _not_ change the ordering of fields in these structs!
//! The ordering must match that in the generated schema, which
//! you can obtain with `diesel print-schema`.

use std::borrow::Cow;
use chrono::DateTime;
use chrono::offset::Utc;
use diesel::prelude::*;
use db::schema::{accounts, users, statuses, follows};
use db::Connection;
use pwhash::bcrypt;
use ::{BASE_URL, DOMAIN};

/// Represents an account (local _or_ remote) on the network, storing federation-relevant information.
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
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Account)]
#[table_name = "statuses"]
pub struct Status {
    pub id: i64,
    pub text: String,
    pub content_warning: Option<String>,
    pub created_at: DateTime<Utc>,

    pub account_id: i64,
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
        where S: Into<String>
    {
        bcrypt::verify(&self.encrypted_password, &password.into())
    }

    /// Hashes a plaintext password for storage in the database.
    pub fn encrypt_password<S>(password: S) -> String
        where S: Into<String>
    {
        bcrypt::hash(&password.into()).expect("Couldn't hash password!")
    }

    // TODO: should probably be Result<Option<T>>?
    // requires a bit of thought, because we can't just try_opt! then :(
    pub fn by_username(db_conn: &Connection, username: String) -> Option<User> {
        let account = try_opt!({
            use db::schema::accounts::dsl;
            dsl::accounts
                .filter(dsl::username.eq(username))
                .filter(dsl::domain.is_null())
                .first::<Account>(&**db_conn).optional().unwrap()
        });

        use db::schema::users::dsl;
        dsl::users
            .filter(dsl::account_id.eq(account.id))
            .first::<User>(&**db_conn).optional().unwrap()
    }
}

impl Account {
    // TODO: result
    pub fn fetch_local_by_username<S>(db_conn: &Connection, username: S) -> Option<Account>
        where S: Into<String>
    {
        use db::schema::accounts::dsl;
        dsl::accounts
            .filter(dsl::username.eq(username.into()))
            .filter(dsl::domain.is_null())
            .first::<Account>(&**db_conn).optional().unwrap()
    }

    pub fn fully_qualified_username(&self) -> String {
        format!("@{user}@{domain}", user=self.username,
                                    domain=self.get_domain())
    }

    pub fn get_domain(&self) -> &str {
        self.domain.as_ref().map(String::as_str)
            .unwrap_or(DOMAIN.as_str())
    }

    // TODO: gross, should probably clean up sometime
    pub fn get_uri<'a>(&'a self) -> Cow<'a, str> {
        self.uri.as_ref().map(|x| String::as_str(x).into())
            .unwrap_or(format!("{base}/users/{user}", base=BASE_URL.as_str(),
                                                      user=self.username).into())
    }

    pub fn get_inbox_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri.as_ref().map(|x| String::as_str(x).into())
            .unwrap_or(format!("{base}/users/{user}/inbox", base=BASE_URL.as_str(),
                                                            user=self.username).into())
    }

    pub fn get_outbox_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri.as_ref().map(|x| String::as_str(x).into())
            .unwrap_or(format!("{base}/users/{user}/outbox", base=BASE_URL.as_str(),
                                                             user=self.username).into())
    }

    pub fn get_following_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri.as_ref().map(|x| String::as_str(x).into())
            .unwrap_or(format!("{base}/users/{user}/following", base=BASE_URL.as_str(),
                                                                user=self.username).into())
    }

    pub fn get_followers_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri.as_ref().map(|x| String::as_str(x).into())
            .unwrap_or(format!("{base}/users/{user}/followers", base=BASE_URL.as_str(),
                                                                user=self.username).into())
    }
}
