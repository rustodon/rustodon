use db::schema::{accounts, users};

/// Represents an account (local _or_ remote) on the network, storing federation-relevant information.
#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "accounts"]
pub struct Account {
    pub id: i64,
    pub username: String,
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
