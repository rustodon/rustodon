use r2d2;
use r2d2_diesel::ConnectionManager;
use diesel::pg::PgConnection;

/// Convenient type alias for the postgres database pool so we don't have to type this out.
type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Initializes a new connection pool for the database at `url`.
pub fn init_connection_pool<S>(url: S) -> Result<Pool, r2d2::Error>
    where S: Into<String>
{
    let manager = ConnectionManager::<PgConnection>::new(url);

    r2d2::Pool::builder()
        .build(manager)
}
