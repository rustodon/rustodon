use crate::db::{self, DbConnection, LOCAL_ACCOUNT_DOMAIN};
use diesel;
use diesel::prelude::*;
use pwhash::bcrypt;
use resopt::try_resopt;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};

use super::Account;
use crate::db::schema::users;

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

/// Represents a new user for insertion into the database.
#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i64,
    pub email: String,
    pub encrypted_password: String,

    pub account_id: i64,
}

impl NewUser {
    pub fn insert(self, conn: &DbConnection) -> QueryResult<User> {
        use crate::db::schema::users::dsl::*;

        diesel::insert_into(users).values(&self).get_result(conn)
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
    pub fn by_username<S>(db_conn: &DbConnection, username: S) -> QueryResult<Option<User>>
    where
        S: AsRef<str>,
    {
        let account = try_resopt!({
            use crate::db::schema::accounts::dsl;
            dsl::accounts
                .filter(dsl::username.eq(username.as_ref()))
                .filter(dsl::domain.eq(LOCAL_ACCOUNT_DOMAIN))
                .first::<Account>(db_conn)
                .optional()
        });

        use crate::db::schema::users::dsl;
        dsl::users
            .filter(dsl::account_id.eq(account.id))
            .first::<User>(db_conn)
            .optional()
    }

    /// Finds a local user by their ID, returning an `Option<User>`.
    pub fn by_id(db_conn: &DbConnection, uid: i64) -> QueryResult<Option<User>> {
        use crate::db::schema::users::dsl::*;

        users.find(uid).first(db_conn).optional()
    }

    /// Returns the number of local users.
    pub fn count(db_conn: &DbConnection) -> QueryResult<i64> {
        use crate::db::schema::users::dsl::users;

        users.count().get_result(db_conn)
    }

    /// Returns the corresponding `Account` of a local user.
    ///
    /// Note: panics if the account does not exist. This _will_ be caught by
    /// Rocket, but this _should be_ an irrecoverable error - there's no concievable
    /// circumstance outside of horrible database meddling that would cause this.
    pub fn get_account(self, db_conn: &DbConnection) -> QueryResult<Account> {
        use crate::db::schema::accounts::dsl::*;

        accounts
            .find(self.account_id)
            .first(db_conn)
            .optional()
            .map(|x| x.expect("All users should have an account!"))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        use rocket::http::Status;
        use rocket::Outcome;

        let db_conn = request.guard::<db::Connection>()?;

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
