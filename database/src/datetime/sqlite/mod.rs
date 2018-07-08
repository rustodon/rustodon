use chrono;
use chrono_humanize::Humanize;
use datetime::{Humanizable, NewDateTime, Rfc339able};

impl Humanizable for chrono::NaiveDateTime {
    fn humanize(&self) -> String {
        chrono::DateTime::<chrono::offset::Utc>::from_utc(*self, chrono::offset::Utc).humanize()
    }
}

impl Rfc339able for chrono::NaiveDateTime {
    fn to_rfc3339(&self) -> String {
        chrono::DateTime::<chrono::offset::Utc>::from_utc(*self, chrono::offset::Utc).to_rfc3339()
    }
}

impl NewDateTime for chrono::NaiveDateTime {
    fn now() -> chrono::NaiveDateTime {
        chrono::offset::Utc::now().naive_utc()
    }
}
