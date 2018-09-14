use std::time::Duration;

pub trait Job<E> {
    fn run(&self) -> Result<(), E>;
}

enum Backoff {
    ConstantWait(Duration),
    Exponential { base: Duration },
}

enum FailBehavior {
    Retry(Backoff),
    Destroy,
}

struct ExecutionContract {
    timeout: Option<Duration>,
    fail_behavior: FailBehavior,
}
