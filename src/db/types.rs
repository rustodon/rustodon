#[derive(Debug, PartialEq, Eq, DbEnum)]
#[PgType = "job_status"]
#[DieselType = "Job_status"]
pub enum JobStatus {
    Waiting,
    Running,
    Dead,
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        (match self {
            JobStatus::Waiting => "waiting",
            JobStatus::Running => "running",
            JobStatus::Dead => "dead",
        }).to_string()
    }
}
