use std::ops::Deref;
use r2d2;
use r2d2_diesel::ConnectionManager;

#[cfg(feature = "sqlite")]
use diesel::sqlite::SqliteConnection;

#[cfg(feature = "postgres")]
use diesel::pg::PgConnection;

use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use rocket::http::Status;

pub mod schema;
pub mod models;

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("sqlite and postgres features cannot be simultaneously selected");

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
type DieselConnection = SqliteConnection;

#[cfg(all(not(feature = "sqlite"), feature = "postgres"))]
type DieselConnection = PgConnection;

/// Convenient type alias for the postgres database pool so we don't have to type this out.
type Pool = r2d2::Pool<ConnectionManager<DieselConnection>>;

/// Type alias for the pooled connection.
type PooledConnection = r2d2::PooledConnection<ConnectionManager<DieselConnection>>;

/// Initializes a new connection pool for the database at `url`.
pub fn init_connection_pool<S>(url: S) -> Result<Pool, r2d2::Error>
where
    S: Into<String>,
{
    let manager = ConnectionManager::<DieselConnection>::new(url);

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

/// A convenient way to use a `&db::Connection` as a `&DieselConnection`.
///
/// Just allows deref-ing the inner `PooledConnection`.
impl Deref for Connection {
    type Target = DieselConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
