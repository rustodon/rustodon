#[derive(Debug, PartialEq, Eq, DbEnum, Clone, Copy)]
#[PgType = "job_status"]
#[DieselType = "Job_status"]
pub enum JobStatus {
    Waiting,
    Running,
    Dead,
    RetryQueued,
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        (match self {
            JobStatus::Waiting => "waiting",
            JobStatus::Running => "running",
            JobStatus::Dead => "dead",
            JobStatus::RetryQueued => "retry-queued",
        })
        .to_string()
    }
}
