use std::collections::HashMap;

use ::job::Job;

type WorkerFn = Fn() -> Result<(), ()>;

pub struct Worker {
    handlers: HashMap<&'static str, Box<WorkerFn>>,
}

impl Worker {
    pub fn new() -> Worker {
        Worker {
            handlers: HashMap::new(),
        }
    }
}
