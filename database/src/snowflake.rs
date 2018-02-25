use super::Connection;

/// Initializes the ID generator.
///
/// The ID generator sets up state in the database, which is why it
/// needs a database connection.  This state applies to *all* connections
/// from that database, so it is only necessary to call this function
/// once.
pub fn init(conn: Connection) -> Result<(), &'static str> {
    return Err("no generator defined");
}

pub fn make_id(conn: &Connection) -> i64 {
    return 0;
}
