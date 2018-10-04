#[macro_use]
extern crate quick_error;
extern crate serde_json;
extern crate serde;
extern crate threadpool;

mod error;
pub mod job;
pub mod worker;

pub use error::Error;
pub use job::{Backoff, ExecutionContract, FailBehavior, Job, Perform};
pub use worker::Worker;
