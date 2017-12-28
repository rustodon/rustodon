use diesel;

enum Error {
    DBError(diesel::result::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Error {
        Error::DBError(err)
    }
}
