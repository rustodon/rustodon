use crate::db::Connection;
use crate::BASE_URL;
use chrono::offset::Utc;
use chrono::DateTime;
use chrono_humanize::Humanize;
use diesel;
use diesel::prelude::*;
use std::borrow::Cow;

use super::Account;
use crate::db::schema::statuses;

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
        use crate::db::schema::statuses::dsl::*;

        diesel::insert_into(statuses)
            .values(&self)
            .get_result(&**conn)
    }
}

impl Status {
    /// Returns the `Account` which authored this status.
    pub fn account(&self, db_conn: &Connection) -> QueryResult<Account> {
        use crate::db::schema::accounts::dsl;
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
        use crate::db::schema::statuses::dsl;
        dsl::statuses
            .find(id)
            .filter(dsl::account_id.eq(account_id))
            .first::<Status>(&**db_conn)
            .optional()
    }

    /// Returns the number of local statuses
    pub fn count_local(db_conn: &Connection) -> QueryResult<i64> {
        use crate::db::schema::statuses::dsl::{statuses, uri};
        statuses
            .filter(uri.is_null()) // is local status
            .count()
            .get_result(&**db_conn)
    }

    /// Returns a URI to the ActivityPub object of this status.
    pub fn get_uri(&self, db_conn: &Connection) -> QueryResult<Cow<'_, str>> {
        let account = self.account(db_conn)?;
        Ok(self.uri_with_account(&account))
    }

    /// Returns a human-readble description of the age of this status.
    pub fn humanized_age(&self) -> String {
        self.created_at.humanize()
    }

    /// Returns `n` local statuses which were authored _strictly before_ the status `max_id`.
    pub fn local_before_id(
        db_conn: &Connection,
        max_id: Option<i64>,
        n: usize,
    ) -> QueryResult<Vec<Status>> {
        use crate::db::schema::statuses::dsl;

        let mut query = dsl::statuses.filter(dsl::uri.is_null()).into_boxed();

        if let Some(max_id) = max_id {
            query = query.filter(dsl::id.lt(max_id));
        }

        query
            .order(dsl::id.desc())
            .limit(n as i64)
            .get_results::<Status>(&**db_conn)
    }

    /// Returns `n` statuses in the database, authored _strictly before_ the
    /// status `max_id`.
    pub fn federated_before_id(
        db_conn: &Connection,
        max_id: Option<i64>,
        n: usize,
    ) -> QueryResult<Vec<Status>> {
        use crate::db::schema::statuses::{all_columns, dsl};

        let mut query = dsl::statuses.select(all_columns).into_boxed();

        if let Some(max_id) = max_id {
            query = query.filter(dsl::id.lt(max_id));
        }

        query
            .order(dsl::id.desc())
            .limit(n as i64)
            .get_results::<Status>(&**db_conn)
    }

    /// Returns a tuple of upper and lower bounds on the IDs of statuses authored locally
    /// (i.e., `min(ids)` and `max(ids)` where `ids` is a list of status ids authored locally).
    ///
    /// If there are no local statuses in the database, return `None`.
    pub fn local_status_id_bounds(db_conn: &Connection) -> QueryResult<Option<(i64, i64)>> {
        use crate::db::schema::statuses::dsl::*;
        use diesel::dsl::sql;
        // Yes, this is gross and we don't like having to use sql() either.
        // See [diesel-rs/diesel#3](https://github.com/diesel-rs/diesel/issues/3) for why this is necessary.
        statuses
            .select((sql("min(id)"), sql("max(id)")))
            .filter(uri.is_null())
            .first::<(Option<i64>, Option<i64>)>(&**db_conn)
            .map(|result| match result {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            })
    }

    /// Returns a tuple of upper and lower bounds on the IDs of statuses in the database
    /// (i.e., `min(ids)` and `max(ids)` where `ids` is a list of status ids).
    ///
    /// If there are no statuses in the database, return `None`.
    pub fn federated_status_id_bounds(db_conn: &Connection) -> QueryResult<Option<(i64, i64)>> {
        use crate::db::schema::statuses::dsl::*;
        use diesel::dsl::sql;
        // Yes, this is gross and we don't like having to use sql() either.
        // See [diesel-rs/diesel#3](https://github.com/diesel-rs/diesel/issues/3) for why this is necessary.
        statuses
            .select((sql("min(id)"), sql("max(id)")))
            .first::<(Option<i64>, Option<i64>)>(&**db_conn)
            .map(|result| match result {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            })
    }

    pub fn uri_with_account<'a>(&'a self, account: &Account) -> Cow<'a, str> {
        assert_eq!(
            account.id, self.account_id,
            "Account {} did not create Status {}, cannot present URI",
            account.id, self.id
        );
        match self.uri.as_ref().map(|x| String::as_str(x).into()) {
            Some(x) => x,
            None => format!(
                "{base}{path}",
                base = BASE_URL.as_str(),
                path = self.path_with_account(account)
            )
            .into(),
        }
    }

    pub fn path_with_account<'a>(&'a self, account: &Account) -> Cow<'a, str> {
        assert_eq!(
            account.id, self.account_id,
            "Account {} did not create Status {}, cannot present URI",
            account.id, self.id
        );
        format!(
            "/users/{user}/statuses/{id}",
            user = account.username,
            id = self.id
        )
        .into()
    }
}
