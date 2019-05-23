use failure::Fallible;
use std::time::Duration;

pub trait Job {
    /// Returns a textual identifier for this job.
    fn kind() -> &'static str;

    /// Returns true, if this job is due to execute.
    fn should_run(&self) -> bool;

    /// Returns the execution contract of this job.
    fn execution_contract() -> ExecutionContract;
}

#[derive(Debug, Clone, Copy)]
pub enum Backoff {
    ConstantWait(Duration),
    Exponential { base: Duration },
}

#[derive(Debug, Clone, Copy)]
pub enum RetryBehavior {
    Backoff(Backoff),
    Immediate,
}

#[derive(Debug, Clone, Copy)]
pub enum PanicBehavior {
    Fail,
    Retry(RetryBehavior),
}

#[derive(Debug, Clone, Copy)]
pub struct ExecutionContract {
    pub timeout: Option<Duration>,
    pub retry_behavior: RetryBehavior,
    pub autoretry: Option<RetryBehavior>,
    pub panic: PanicBehavior,
}

impl ExecutionContract {
    pub const fn new() -> Self {
        Self {
            panic: PanicBehavior::Fail,
            retry_behavior: RetryBehavior::Immediate,
            autoretry: None,
            timeout: None,
        }
    }
}

pub trait Perform {
    /// Runs this job's action.
    fn perform(&self) -> Fallible<()>;
}
