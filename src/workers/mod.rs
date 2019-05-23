use diesel;
use diesel::prelude::*;
use failure::{format_err, Error};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use slog::{slog_debug, slog_error, slog_info, slog_trace};
use slog_scope::{debug, error, info, trace};
use std::thread;
use std::time::Duration;
use turnstile::{self, ExecutionContract, Job, PanicBehavior, Perform, Worker};

use crate::db::models::JobRecord;
use crate::db::types::JobStatus;
use crate::db::Pool;

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

        // panic!("🅱️anic");

        Err(format_err!("a constructed error"))

        // Ok(())
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
                let job_id = job_record.id;

                if let Err(e) = worker.job_tick(
                    &job_record.kind.clone(),
                    job_record.data.clone(),
                    move |result| {
                        match result {
                            // If the job encountered an inner error, fail/reschedule it, following the job type's execution policy.
                            Err(turnstile::Error::JobInnerError(inner_error)) => {
                                error!("Job encountered inner error"; "error" => %inner_error);
                            },
                            // If the job panicked, fail/reschedule it, following the job type's execution policy.
                            Err(turnstile::Error::JobPanicked(panic_msg)) => {
                                error!("Job panicked!"; "panic_message" => %panic_msg);
                            },
                            
                            // Immediately terminate the job if we failed to deserialize, since serde is generally deterministic,
                            // and won't succeed if we try again.
                            Err(turnstile::Error::DeserializeError(serde_error)) => {
                                error!("Job failed to deserialize"; "error" => %serde_error);
                                use crate::db::schema::jobs::dsl::*;
                                let conn = pool.get().unwrap();
                                diesel::update(jobs)
                                    .filter(id.eq(job_id))
                                    .set(status.eq(JobStatus::Dead))
                                    .execute(&conn)
                                    .unwrap();
                            },

                            Err(turnstile::Error::InvalidKind) => unreachable!(), // wouldn't be accepted in the first place

                            // The job completed successfully! We can just delete it from the database.
                            Ok(_) => {
                                debug!("Job execution succeeded!"; "id" => job_id);

                                use crate::db::schema::jobs::dsl::*;
                                let conn = pool.get().unwrap();
                                diesel::delete(jobs.filter(id.eq(job_id)))
                                    .execute(&conn)
                                    .unwrap();
                            },
                        }
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
