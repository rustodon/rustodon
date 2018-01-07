use std::ops::Deref;
use r2d2;
use r2d2_diesel::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use rocket::http::Status;

pub mod schema;
pub mod models;

/// Convenient type alias for the postgres database pool so we don't have to type this out.
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Type alias for the pooled connection.
type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

/// Initializes a new connection pool for the database at `url`.
pub fn init_connection_pool<S>(url: S) -> Result<Pool, r2d2::Error>
where
    S: Into<String>,
{
    let manager = ConnectionManager::<SqliteConnection>::new(url);

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

/// A convenient way to use a `&db::Connection` as a `&SqliteConnection`.
///
/// Just allows deref-ing the inner `PooledConnection`.
impl Deref for Connection {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
