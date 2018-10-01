use std::time::Duration;
use std::error::Error;

pub trait Job {
    /// Returns a textual identifier for this job.
    fn kind() -> &'static str;

    /// Returns true, if this job is due to execute.
    fn should_run(&self) -> bool;

    /// Returns the execution contract of this job.
    fn execution_contract(&self) -> ExecutionContract;
}

pub enum Backoff {
    ConstantWait(Duration),
    Exponential { base: Duration },
}

pub enum FailBehavior {
    Retry(Backoff),
    Destroy,
}

pub struct ExecutionContract {
    pub timeout: Option<Duration>,
    pub fail_behavior: FailBehavior,
}

pub trait Perform {
    /// Runs this job's action.
    fn perform(&self) -> Result<(), Box<Error>>;
}
