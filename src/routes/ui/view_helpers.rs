use db;
use db::models::Account;
use transform;

pub trait HasBio {
    fn transformed_bio<'a>(&'a self, connection: &db::Connection) -> Option<String>;
}

impl HasBio for Account {
    fn transformed_bio<'a>(&'a self, connection: &db::Connection) -> Option<String> {
        if let Some(raw_bio) = self.summary.as_ref().map(String::as_str) {
            match transform::bio(raw_bio, connection) {
                Ok(transformed) => Some(transformed),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}
