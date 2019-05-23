use serde::de::Deserialize;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::panic::{self, RefUnwindSafe};
use std::sync::Arc;
use threadpool::{Builder as ThreadPoolBuilder, ThreadPool};

use crate::error::{Error, SyncPanicError};
use crate::job::{ExecutionContract, Job, Perform};

type HandlerFn = Box<(Fn(Value) -> Result<(), Error> + Send + Sync + 'static)>;
type ShouldRunFn = Box<(Fn(Value) -> Result<bool, Error> + Send + Sync + 'static)>;

pub struct Worker {
    handlers: HashMap<&'static str, Arc<HandlerFn>>,
    run_checks: HashMap<&'static str, Arc<ShouldRunFn>>,
    execution_contracts: HashMap<&'static str, ExecutionContract>,
    thread_pool: ThreadPool,
}

impl Worker {
    pub fn new() -> Worker {
        Worker {
            handlers: HashMap::new(),
            run_checks: HashMap::new(),
            execution_contracts: HashMap::new(),
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
                let job: J =
                    serde_json::from_value(value).map_err(|e| Error::DeserializeError(e.into()))?;

                panic::catch_unwind(|| Perform::perform(&job))
                    .map_err(|panic| Error::JobPanicked(SyncPanicError::new(panic)))?
                    .map_err(Error::JobInnerError)?;

                Ok(())
            })),
        );

        self.run_checks.insert(
            J::kind(),
            Arc::new(Box::new(|value| {
                let job: J =
                    serde_json::from_value(value).map_err(|e| Error::DeserializeError(e.into()))?;

                Ok(job.should_run())
            })),
        );

        self.execution_contracts
            .insert(J::kind(), J::execution_contract());
    }

    pub fn job_tick(
        &mut self,
        kind: &str,
        data: Value,
        on_final: impl Fn(Result<(), Error>, ExecutionContract) + Send + 'static,
    ) -> Result<(), Error> {
        let handler = self.handlers.get(kind).ok_or(Error::InvalidKind)?.clone();
        let execution_contract = self
            .execution_contracts
            .get(kind)
            .ok_or(Error::InvalidKind)?
            .clone();
        self.thread_pool.execute(move || {
            let result = handler(data);

            on_final(result, execution_contract);
        });

        Ok(())
    }

    pub fn should_run(&mut self, kind: &str, data: Value) -> Result<bool, Error> {
        let run_check = self.run_checks.get(kind).ok_or(Error::InvalidKind)?.clone();

        run_check(data)
    }
}
