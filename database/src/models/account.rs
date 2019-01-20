use diesel;
use diesel::prelude::*;
use std::borrow::Cow;
use Connection;
use {BASE_URL, DOMAIN, LOCAL_ACCOUNT_DOMAIN};

use models::{Status, User};
use rocket::request::{self, FromRequest, Request};

use schema::accounts;

/// Represents an account (local _or_ remote) on the network, storing federation-relevant information.
///
/// A uri of None implies a local account.
///
/// Uniqueness is enforced both on the username/domain pair and on the uri.

#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "accounts"]
pub struct Account {
    pub id: i64,
    pub uri: Option<String>,
    pub domain: Option<String>,
    pub username: String,

    pub display_name: Option<String>,
    pub summary: Option<String>,

    pub pubkey:  Vec<u8>,
    pub privkey: Vec<u8>,
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

        diesel::insert_into(accounts)
            .values(&self)
            .get_result(&**conn)
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
            .filter(dsl::domain.eq(LOCAL_ACCOUNT_DOMAIN))
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
            query = query.filter(dsl::domain.eq(LOCAL_ACCOUNT_DOMAIN));
        };

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
        if let Some(domain_str) = self.domain.as_ref() {
            if domain_str.as_str() == LOCAL_ACCOUNT_DOMAIN {
                DOMAIN.as_str()
            } else {
                domain_str.as_str()
            }
        } else {
            // This is not quite "correct", in that a domain that is NULL is neither a
            // local domain nor a remote domain, however it reflects the intent of the
            // application the closest. There may be downstream security concerns here
            // tho if a user is able to somehow manufacture an Account with a NULL
            // domain.
            DOMAIN.as_str()
        }
    }

    /// Returns the URI of the account's ActivityPub object.
    pub fn get_uri<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}{path}",
                    base = BASE_URL.as_str(),
                    path = self.profile_path()
                )
                .into()
            })
    }

    /// Returns the server local path to the Account profile page for this account.
    pub fn profile_path<'a>(&'a self) -> Cow<'a, str> {
        format!("/users/{user}", user = self.username).into()
    }

    /// Returns the URI of the ActivityPub `inbox` endpoint for this account.
    pub fn get_inbox_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}{path}",
                    base = BASE_URL.as_str(),
                    path = self.inbox_path()
                )
                .into()
            })
    }

    /// Returns the server local path to the `inbox` endpoint for this account.
    pub fn inbox_path<'a>(&'a self) -> Cow<'a, str> {
        format!("/users/{user}/inbox", user = self.username).into()
    }

    /// Returns the URI of the ActivityPub `outbox` endpoint for this account.
    pub fn get_outbox_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}{path}",
                    base = BASE_URL.as_str(),
                    path = self.outbox_path()
                )
                .into()
            })
    }

    /// Returns the server local path to the `outbox` endpoint for this account.
    pub fn outbox_path<'a>(&'a self) -> Cow<'a, str> {
        format!("/users/{user}/outbox", user = self.username).into()
    }

    /// Returns the URI of the ActivityPub `following` endpoint for this account.
    pub fn get_following_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}{path}",
                    base = BASE_URL.as_str(),
                    path = self.following_path()
                )
                .into()
            })
    }

    /// Returns the server local path to the `following` endpoint for this account.
    pub fn following_path<'a>(&'a self) -> Cow<'a, str> {
        format!("/users/{user}/following", user = self.username).into()
    }

    /// Returns the URI of the ActivityPub `followers` endpoint for this account.
    pub fn get_followers_endpoint<'a>(&'a self) -> Cow<'a, str> {
        self.uri
            .as_ref()
            .map(|x| String::as_str(x).into())
            .unwrap_or_else(|| {
                format!(
                    "{base}{path}",
                    base = BASE_URL.as_str(),
                    path = self.followers_path()
                )
                .into()
            })
    }

    /// Returns the server local path to the `followers` resource on this account.
    pub fn followers_path<'a>(&'a self) -> Cow<'a, str> {
        format!("/users/{user}/followers", user = self.username).into()
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
            .select((sql("min(id)"), sql("max(id)")))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn it_calculates_domain_correctly() {
        let local_domain: &str = "local.domain";
        let test_env_domain = env::var("DOMAIN");
        env::set_var("DOMAIN", local_domain);
        let account_with_null_domain: Account = Account {
            id: 1,
            uri: Some("http://fake.value/1".to_string()),
            domain: None,
            username: "account1".to_string(),
            display_name: None,
            summary: None,
        };
        let account_with_local_domain: Account = Account {
            id: 2,
            uri: Some("http://fake.value/2".to_string()),
            domain: Some(LOCAL_ACCOUNT_DOMAIN.to_string()),
            username: "account2".to_string(),
            display_name: None,
            summary: None,
        };
        let account_with_remote_domain: Account = Account {
            id: 3,
            uri: Some("http://fake.value/3".to_string()),
            domain: Some("fake.value".to_string()),
            username: "account3".to_string(),
            display_name: None,
            summary: None,
        };
        assert_eq!(
            account_with_null_domain.get_domain(),
            local_domain,
            "Account with NULL domain should be calculated as the local domain"
        );
        assert_eq!(
            account_with_local_domain.get_domain(),
            local_domain,
            "Account created locally should be calculated as the local domain"
        );
        assert_eq!(
            account_with_remote_domain.get_domain(),
            "fake.value",
            "Account from remote should be calculated as their origin"
        );
        if let Ok(original_domain) = test_env_domain {
            env::set_var("DOMAIN", original_domain);
        }
    }
}
