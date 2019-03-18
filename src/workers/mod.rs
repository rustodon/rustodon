use diesel::prelude::*;
use serde_json;
use std::error::Error;
use std::thread;
use std::time::Duration;

use crate::db::models::JobRecord;
use crate::db::types::JobStatus;
use crate::db::Pool;
use diesel;
use serde_derive::{Deserialize, Serialize};
use slog::{slog_info, slog_trace};
use slog_scope::{info, trace};
use turnstile::{ExecutionContract, Job, Perform, Worker};

const BATCH_SIZE: i64 = 10;
const CHECK_PERIOD: Duration = Duration::from_secs(1); // 1/(1 hz)

#[derive(Serialize, Deserialize)]
pub struct TestJob {
    pub msg: String,
}

impl Job for TestJob {
    fn kind() -> &'static str {
        "test_job"
    }

    fn should_run(&self) -> bool {
        true
    }

    fn execution_contract(&self) -> ExecutionContract {
        ExecutionContract::immediate_fail()
    }
}

impl Perform for TestJob {
    fn perform(&self) -> Result<(), Box<Error>> {
        info!("+++++++ {a} {a} {a} {a} +++++++", a = &self.msg);
        Ok(())
    }
}

pub fn init(pool: Pool) {
    let mut worker = Worker::new();

    worker.register_job::<TestJob>();

    thread::Builder::new()
        .name("job_collector".to_string())
        .spawn(move || loop {
            let conn = pool.get().expect("couldn't connect to database");
            let top_of_queue = {
                use crate::db::schema::jobs::dsl::*;
                jobs.filter(status.eq(JobStatus::Waiting))
                    .limit(BATCH_SIZE)
                    .order(id.asc())
                    .load::<JobRecord>(&conn)
                    .expect("couldn't load from job queue")
            };

            trace!("job collection tick"; "top_of_queue" => ?top_of_queue);

            let should_run: Vec<&JobRecord> = top_of_queue.iter().filter(|_| true).collect();
            {
                use crate::db::schema::jobs::dsl::*;
                diesel::update(jobs)
                    .filter(id.eq_any(should_run.iter().map(|j| j.id).collect::<Vec<i64>>()))
                    .set(status.eq(JobStatus::Running))
                    .execute(&conn)
                    .unwrap();
            }

            for job_record in top_of_queue {
                let job_id = job_record.id;
                let pool = pool.clone();

                worker
                    .job_tick(&job_record.kind, job_record.data, move |_result| {
                        use crate::db::schema::jobs::dsl::*;
                        let conn = pool.get().unwrap();
                        diesel::delete(jobs.filter(id.eq(job_id)))
                            .execute(&conn)
                            .unwrap();
                    })
                    .unwrap();
            }

            thread::sleep(CHECK_PERIOD);
        })
        .expect("failed to spawn job_collector thread");
}
