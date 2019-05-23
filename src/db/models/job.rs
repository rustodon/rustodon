use chrono::offset::Utc;
use chrono::prelude::*;
use diesel::prelude::*;
use serde::Serialize;
use serde_json::{self, Value};
use std::error::Error;

use turnstile::Job;

use crate::db::idgen::id_generator;
use crate::db::schema::jobs;
use crate::db::types::JobStatus;
use crate::db::DbConnection;

#[derive(Identifiable, Queryable, Associations, PartialEq, Clone, Debug)]
#[table_name = "jobs"]
pub struct JobRecord {
    pub id: i64,
    pub created_at: DateTime<Utc>,

    pub status: JobStatus,

    pub queue: String,
    pub kind:  String,
    pub data:  Value,

    pub last_attempt: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug)]
#[table_name = "jobs"]
pub struct NewJobRecord<'a> {
    pub id: i64,
    pub created_at: DateTime<Utc>,

    pub status: JobStatus,

    pub queue: &'a str,
    pub kind:  &'a str,
    pub data:  Value,
}

impl<'a> NewJobRecord<'a> {
    pub fn on_queue<J>(x: J, queue: &str) -> Result<NewJobRecord, serde_json::Error>
    where
        J: Job + Serialize,
    {
        Ok(NewJobRecord {
            id: id_generator().next(),
            created_at: Utc::now(),
            data: serde_json::to_value(x)?,
            kind: J::kind(),
            queue: queue,
            status: JobStatus::Waiting,
        })
    }
}

impl JobRecord {
    pub fn kill(&self, conn: &DbConnection) -> QueryResult<usize> {
        use crate::db::schema::jobs::dsl::*;

        diesel::update(jobs)
            .filter(id.eq(self.id))
            .set(status.eq(JobStatus::Dead))
            .execute(conn)
    }

    pub fn drop(&self, conn: &DbConnection) -> QueryResult<usize> {
        use crate::db::schema::jobs::dsl::*;

        diesel::delete(jobs.filter(id.eq(self.id))).execute(conn)
    }
}
