use chrono;

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub type DateTimeType = chrono::NaiveDateTime;

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub mod sqlite;

#[cfg(all(not(feature = "sqlite"), feature = "postgres"))]
pub type DateTimeType = chrono::DateTime<chrono::offset::Utc>;

#[cfg(all(not(feature = "sqlite"), feature = "postgres"))]
pub mod postgres;

pub trait Humanizable {
    fn humanize(&self) -> String;
}

pub trait Rfc339able {
    fn to_rfc3339(&self) -> String;
}

pub trait NewDateTime {
    fn now() -> DateTimeType;
}
