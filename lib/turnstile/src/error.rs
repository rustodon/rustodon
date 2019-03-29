use serde_json;
use std::any::Any;
use failure::Fail;
use std::sync::Mutex;
use std::fmt::{self, Debug, Display};

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Inner error in job: {}", _0)]
    JobInnerError(#[fail(cause)] failure::Error),

    #[fail(display = "Job panicked: {:?}", _0)]
    JobPanicked(#[fail(cause)] SyncPanicError),

    #[fail(display = "Error deserializing job data: {}", _0)]
    DeserializeError(#[fail(cause)] serde_json::Error),

    #[fail(display = "Invalid kind for job")]
    InvalidKind,
}

pub struct SyncPanicError {
    inner: Mutex<Box<dyn Any + Send + 'static>>
}

impl SyncPanicError {
    pub(crate) fn new(inner: Box<dyn Any + Send + 'static>) -> Self {
        Self {
            inner: Mutex::new(inner)
        }
    }
}

impl Display for SyncPanicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self.inner.lock().unwrap()).fmt(f)
    }
}

impl Debug for SyncPanicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self.inner.lock().unwrap()).fmt(f)
    }
}

impl Fail for SyncPanicError {}