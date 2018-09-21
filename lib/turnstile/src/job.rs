use std::time::Duration;

pub trait Job {
    fn should_run(&self) -> bool;
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
    timeout: Option<Duration>,
    fail_behavior: FailBehavior,
}

pub trait Perform {
    /// Runs the job's duty
    fn perform(&self);
}
