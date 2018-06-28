use diesel;
use diesel::prelude::*;
use std::borrow::Cow;
use Connection;
use {BASE_URL, DOMAIN};

use models::{Status, User};
use rocket::request::{self, FromRequest, Request};

use schema::accounts;

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

impl NewAccount {
    pub fn insert(self, conn: &Connection) -> QueryResult<Account> {
        use schema::accounts::dsl::*;
        let inserted = diesel::insert_into(accounts)
            .values(&self)
            .execute(&**conn)?;
        if inserted == 1 {
            if let Some(account) = Account::fetch_by_id(conn, self.id)? {
                Ok(account)
            } else {
                Err(diesel::NotFound)
            }
        } else {
            Err(diesel::NotFound)
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
        use schema::accounts::dsl;
        dsl::accounts
            .filter(dsl::username.eq(username.into()))
            .filter(dsl::domain.is_null())
            .first::<Account>(&**db_conn)
            .optional()
    }

    pub fn fetch_by_username_domain(
        db_conn: &Connection,
        username: impl Into<String>,
        domain: Option<impl Into<String>>,
    ) -> QueryResult<Option<Account>> {
        use schema::accounts::dsl;
        let mut query = dsl::accounts
            .filter(dsl::username.eq(username.into()))
            .into_boxed();

        if let Some(domain) = domain.map(Into::into) {
            query = query.filter(dsl::domain.eq(domain));
        } else {
            query = query.filter(dsl::domain.is_null());
        };

        query.first::<Account>(&**db_conn).optional()
    }

    pub fn fetch_by_id(db_conn: &Connection, id: i64) -> QueryResult<Option<Account>> {
        use schema::accounts::dsl;
        let mut query = dsl::accounts.filter(dsl::id.eq(id)).into_boxed();
        query.first::<Account>(&**db_conn).optional()
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
        use schema::statuses::dsl::*;
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
        use diesel::dsl::sql;
        use schema::statuses::dsl::*;
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
        use schema::accounts::dsl::summary;

        diesel::update(self)
            .set(summary.eq(new_summary))
            .execute(&**db_conn)
            .and(Ok(()))
    }

    pub fn display_name_or_username(&self) -> &str {
        self.display_name.as_ref().unwrap_or(&self.username)
    }
}

impl AsRef<Account> for Account {
    fn as_ref(&self) -> &Account {
        &self
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Account {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Account, ()> {
        use rocket::http::Status;
        use rocket::Outcome;

        let db_conn = request.guard::<Connection>()?;
        if let Some(user) = request.guard::<User>().succeeded() {
            match user.get_account(&db_conn) {
                Ok(account) => Outcome::Success(account),
                Err(_) => Outcome::Failure((Status::InternalServerError, ())),
            }
        } else {
            Outcome::Forward(())
        }
    }
}
