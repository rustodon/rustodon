// silence warnings due to diesel-rs/diesel#1785
#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

pub use diesel::connection::Connection as DieselConnection;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use std::env;
use std::ops::Deref;

pub mod idgen;
pub mod models;
pub mod schema;
pub mod validators;

pub use self::idgen::id_generator;

pub static LOCAL_ACCOUNT_DOMAIN: &'static str = "";

/// The raw database connection type. Aliased so it's easy to switch when using, eg, an sqlite backend.
pub type DbConnection = PgConnection;

/// Convenient type alias for the postgres database pool so we don't have to type this out.
pub type Pool = r2d2::Pool<ConnectionManager<DbConnection>>;

/// Type alias for the pooled connection.
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager<DbConnection>>;

/// Initializes a new connection pool for the database at `url`.
pub fn init_connection_pool<S>(url: S) -> Result<Pool, r2d2::PoolError>
where
    S: Into<String>,
{
    let manager = ConnectionManager::<DbConnection>::new(url);

    r2d2::Pool::builder().build(manager)
}

/// Request guard type for handing out db connections from the pool.
pub struct Connection(pub PooledConnection);

/// Custom guard implementation so routes can grab a database connection easily.
///
/// Attempts to retrieve a single connection from the database pool;
/// if no pool is online, fails with `InternalServerError`.
/// If no connections are available, fails with `ServiceUnavailable`.
impl<'a, 'r> FromRequest<'a, 'r> for Connection {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Connection, ()> {
        // retrieve the database connection from Rocket's managed data
        let pool = request.guard::<State<Pool>>()?;

        match pool.get() {
            // .get() a connection from the pool
            Ok(conn) => Outcome::Success(Connection(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

/// A convenient way to use a `&db::Connection` as a `&DbConnection`.
///
/// Just allows deref-ing the inner `PooledConnection`.
impl Deref for Connection {
    type Target = DbConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Connection> for Connection {
    fn as_ref(&self) -> &Connection {
        &self
    }
}
