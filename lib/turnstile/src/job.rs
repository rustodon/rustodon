use failure::Fallible;
use std::time::Duration;

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

impl ExecutionContract {
    pub const fn immediate_fail() -> Self {
        Self {
            fail_behavior: FailBehavior::Destroy,
            timeout: None,
        }
    }
}

pub trait Perform {
    /// Runs this job's action.
    fn perform(&self) -> Fallible<()>;
}
