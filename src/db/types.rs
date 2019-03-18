#[derive(Debug, PartialEq, Eq, DbEnum)]
#[PgType = "job_status"]
#[DieselType = "Job_status"]
pub enum JobStatus {
    Waiting,
    Running,
    Dead,
}
