//! Database models.
//!
//! Note: do _not_ change the ordering of fields in model structs!
//! The ordering must match that in the generated schema, which
//! you can obtain with `diesel print-schema`.

pub use self::account::{Account, NewAccount};
pub use self::follow::Follow;
pub use self::job::{JobRecord, NewJobRecord};
pub use self::status::{NewStatus, Status};
pub use self::user::{NewUser, User};

mod account;
mod follow;
mod job;
mod status;
mod user;
