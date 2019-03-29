use diesel::prelude::*;
use serde_json;
use std::thread;
use std::time::Duration;

use crate::db::models::JobRecord;
use crate::db::types::JobStatus;
use crate::db::Pool;
use diesel;
use failure::{format_err, Error};
use serde_derive::{Deserialize, Serialize};
use slog::{slog_error, slog_info, slog_trace, slog_debug};
use slog_scope::{error, info, trace, debug};
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
    fn perform(&self) -> Result<(), Error> {
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
            // -- pull the top BATCH_SIZE jobs from the queue that are in wait state
            let top_of_queue = {
                use crate::db::schema::jobs::dsl::*;
                jobs.filter(status.eq(JobStatus::Waiting))
                    .limit(BATCH_SIZE)
                    .order(id.asc())
                    .load::<JobRecord>(&conn)
                    .expect("couldn't load from job queue")
            };

            trace!("job collection tick"; "top_of_queue" => ?top_of_queue);

            // -- compute which jobs should run, and set those to running state
            let should_run: Vec<&JobRecord> = top_of_queue.iter().filter(|_| true).collect();
            {
                use crate::db::schema::jobs::dsl::*;
                diesel::update(jobs)
                    .filter(id.eq_any(should_run.iter().map(|j| j.id).collect::<Vec<i64>>()))
                    .set(status.eq(JobStatus::Running))
                    .execute(&conn)
                    .unwrap();
            }

            // -- submit jobs which should be run to the thread pool
            let mut failed_to_submit = Vec::new();
            for job_record in top_of_queue {
                let pool = pool.clone();

                if let Err(e) = worker.job_tick(
                    &job_record.kind.clone(),
                    job_record.data.clone(),
                    move |_result| {
                        use crate::db::schema::jobs::dsl::*;
                        let conn = pool.get().unwrap();
                        diesel::delete(jobs.filter(id.eq(id)))
                            .execute(&conn)
                            .unwrap();
                    },
                ) {
                    error!("submitting job to thread pool failed"; "error" => %e, "job" => ?job_record);
                    failed_to_submit.push(job_record.id);
                }
            }

            // -- kill jobs that weren't successfully submitted to the thread pool
            {
                use crate::db::schema::jobs::dsl::*;
                diesel::update(jobs)
                    .filter(id.eq_any(failed_to_submit))
                    .set(status.eq(JobStatus::Dead))
                    .execute(&conn)
                    .unwrap();
            }

            thread::sleep(CHECK_PERIOD);
        })
        .expect("failed to spawn job_collector thread");
}
