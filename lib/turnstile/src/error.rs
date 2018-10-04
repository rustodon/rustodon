use std::error::Error as StdError;
use std::any::Any;
use serde_json;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        JobInnerError(err: Box<StdError>) {
            cause(&**err)
            description(err.description())
        }

        JobPanicked(panic_inner: Box<Any + Send + 'static>)

        DeserializeError(err: serde_json::Error) {
            cause(err)
            description(err.description())
        }

        InvalidKind
    }
}
