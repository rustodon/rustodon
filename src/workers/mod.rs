use std::thread;
use diesel::prelude::*;

use db::models::JobRecord;
use db::types::JobStatus;
use db::Pool;
use turnstile::Worker;

const BATCH_SIZE: i64 = 10;
const CHECK_PERIOD: f32 = 1.0; // 1/(1 hz)


pub fn init(pool: Pool) {
    let worker = Worker::new();

    thread::Builder::new()
        .name("job_collector".to_string())
        .spawn(move || loop {
            let conn = pool.get().expect("hecking");
            // SELECT * FROM jobs WHERE jobstatus = WAITING
            let top_of_queue = {
                use db::schema::jobs::dsl::*;
                jobs
                    .filter(status.eq(JobStatus::Waiting))
                    .limit(BATCH_SIZE)
                    .order(id.asc())
                    .load::<JobRecord>(&conn)
                    .expect("h*ck")
            };

            // worker.collector_tick(&top_of_queue[..]);

            // .filter(|j| j.should_run())
        }).unwrap(); // TODO: don't unwrap
}
