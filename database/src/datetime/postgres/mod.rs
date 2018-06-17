use chrono;
use datetime::NewDateTime;

impl NewDateTime for chrono::DateTime<chrono::offset::Utc> {
    fn now() -> chrono::DateTime<chrono::offset::Utc> {
        chrono::offset::Utc::now()
    }
}
