use crate::db;
use crate::db::models::Account;
use crate::transform;
use failure::Error;

pub trait HasBio {
    fn transformed_bio(&self, connection: &db::DbConnection) -> Option<String>;
}

impl HasBio for Account {
    fn transformed_bio(&self, connection: &db::DbConnection) -> Option<String> {
        if let Some(raw_bio) = self.summary.as_ref().map(String::as_str) {
            transform::bio(raw_bio, |username, domain| {
                Account::fetch_by_username_domain(connection, username, domain).map_err(Error::from)
            })
            .ok()
        } else {
            None
        }
    }
}
