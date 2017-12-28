//! Database models.
//!
//! Note: do _not_ change the ordering of fields in these structs!
//! The ordering must match that in the generated schema, which
//! you can obtain with `diesel print-schema`.

use diesel::prelude::*;
use db::schema::{accounts, users, statuses, follows};
use db::Connection;
use pwhash::bcrypt;

/// Represents an account (local _or_ remote) on the network, storing federation-relevant information.
#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "accounts"]
pub struct Account {
    pub id: i64,
    pub domain: Option<String>,
    pub username: String,

    pub summary: Option<String>,
    pub display_name: Option<String>,
}

/// Represents a local user, and information required to authenticate that user.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
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
}

impl User {
    // TODO: should probably be Result<Option<T>>?
    pub fn by_username(db_conn: &Connection, username: String) -> Option<User> {
        let account = try_opt!({
            use db::schema::accounts::dsl;
            dsl::accounts
                .filter(dsl::username.eq(username))
                .filter(dsl::domain.is_null())
                .first::<Account>(&**db_conn).ok()
        });

        use db::schema::users::dsl;
        dsl::users
            .filter(dsl::account_id.eq(account.id))
            .first::<User>(&**db_conn).ok()
    }
}
