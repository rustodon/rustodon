use serde::de::Deserialize;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::panic::{self, RefUnwindSafe};
use std::sync::Arc;
use threadpool::{Builder as ThreadPoolBuilder, ThreadPool};

use error::Error;
use job::Job;
use job::Perform;

type HandlerFn = Box<(Fn(Value) -> Result<(), Box<StdError>> + Send + Sync + 'static)>;

pub struct Worker {
    handlers:    HashMap<&'static str, Arc<HandlerFn>>,
    thread_pool: ThreadPool,
}

impl Worker {
    pub fn new() -> Worker {
        Worker {
            handlers:    HashMap::new(),
            thread_pool: ThreadPoolBuilder::new()
                .thread_name("worker_thread".to_string())
                .build(),
        }
    }

    pub fn register_job<J>(&mut self)
    where
        for<'de> J: Job + Perform + Deserialize<'de> + RefUnwindSafe,
    {
        self.handlers.insert(
            J::kind(),
            Arc::new(Box::new(|value| {
                let job: J = serde_json::from_value(value).map_err(Error::DeserializeError)?;

                panic::catch_unwind(|| Perform::perform(&job).map_err(Error::JobInnerError))
                    .map_err(Error::JobPanicked)??;

                Ok(())
            })),
        );
    }

    pub fn job_tick(
        &mut self,
        kind: &str,
        data: Value,
        on_final: impl Fn(Result<(), Box<StdError>>) + Send + 'static,
    ) -> Result<(), Error> {
        let handler = self.handlers.get(kind).ok_or(Error::InvalidKind)?.clone();
        self.thread_pool.execute(move || {
            // TODO: don't discard error?
            let result = handler(data);

            on_final(result);
        });

        Ok(())
    }
}
