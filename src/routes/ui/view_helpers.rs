use db;
use db::models::Account;
use transform;

pub trait HasBio {
    fn transformed_bio<'a>(&'a self, connection: &db::Connection) -> Option<String>;
}

impl HasBio for Account {
    fn transformed_bio<'a>(&'a self, connection: &db::Connection) -> Option<String> {
        if let Some(raw_bio) = self.summary.as_ref().map(String::as_str) {
            transform::bio(raw_bio, connection).ok()
        } else {
            None
        }
    }
}
