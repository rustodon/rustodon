use chrono::offset::Utc;
use chrono::DateTime;
use chrono_humanize::Humanize;
use diesel;
use diesel::prelude::*;
use std::borrow::Cow;
use Connection;
use BASE_URL;

use models::Account;
use schema::statuses;

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

impl NewStatus {
    pub fn insert(self, conn: &Connection) -> QueryResult<Status> {
        use schema::statuses::dsl::*;

        diesel::insert_into(statuses)
            .values(&self)
            .get_result(&**conn)
    }
}

impl Status {
    /// Returns the `Account` which authored this status.
    pub fn account(&self, db_conn: &Connection) -> QueryResult<Account> {
        use schema::accounts::dsl;
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
        use schema::statuses::dsl;
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

    /// Returns a URI to the ActivityPub object of this status.
    pub fn get_uri<'a>(&'a self, db_conn: &Connection) -> QueryResult<Cow<'a, str>> {
        let uri = self.uri.as_ref().map(|x| String::as_str(x).into());
        match uri {
            Some(x) => Ok(x),
            None => {
                let account_result = self.account(db_conn);
                match account_result {
                    Ok(account) => {
                        return Ok(format!(
                            "{base}{path}",
                            base = BASE_URL.as_str(),
                            path = self.status_path(&account).unwrap_or_else(|| "".into())
                        ).into())
                    },
                    Err(error) => return Err(error),
                }
            },
        }
    }

    /// Returns the server local path to this status if it exists, or None
    /// if the status does not reside on this server, or if the account provided
    /// by the caller is not the creator of this status.
    pub fn status_path<'a>(&'a self, account: &Account) -> Option<Cow<'a, str>> {
        match self.uri {
            Some(_) => None,
            None => {
                if account.id == self.account_id {
                    Some(
                        format!(
                            "/users/{user}/statuses/{id}",
                            user = account.username,
                            id = self.id
                        ).into(),
                    )
                } else {
                    None
                }
            },
        }
    }
}
