use chrono::offset::Utc;
use chrono::DateTime;
use serde_json::Value;
use turnstile::Job;

use schema::jobs;
use types::JobStatus;
use std::error::Error;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name = "jobs"]
pub struct JobRecord {
    pub id: i64,
    pub created_at: DateTime<Utc>,

    pub status: JobStatus,

    pub kind: String,
    pub data: Value,
}
