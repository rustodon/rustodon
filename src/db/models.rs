use db::schema::{accounts, users};
use pwhash::bcrypt;

/// Represents an account (local _or_ remote) on the network, storing federation-relevant information.
#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "accounts"]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub summary: Option<String>,
    pub domain: Option<String>,
}

/// Represents a local user, and information required to authenticate that user.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Account)]
#[table_name = "users"]
pub struct User {
    pub id: i64,
    pub email: String,
    pub encrypted_password: String,

    account_id: i64,
}


impl User {
    /// Checks if a plaintext password is valid.
    pub fn valid_password<S>(&self, password: S) -> bool
        where S: Into<String>
    {
        bcrypt::verify(&self.encrypted_password, &password.into())
    }

    /// Hashes a plaintext password for storage in the database.
    pub fn encrypt_password<S>(password: S) -> String
        where S: Into<String>
    {
        bcrypt::hash(&password.into()).expect("Couldn't hash password!")
    }
}
