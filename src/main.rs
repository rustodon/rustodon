#![feature(plugin, nll, custom_derive, proc_macro, proc_macro_non_items)]
#![plugin(rocket_codegen)]
#![recursion_limit = "128"]

extern crate chrono;
extern crate dotenv;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate maud;
#[macro_use]
extern crate resopt;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate validator;
#[macro_use]
extern crate validator_derive;

extern crate rustodon_database as db;

#[macro_use]
mod error;
mod activitypub;
mod routes;
mod templates;

use dotenv::dotenv;
use std::env;

lazy_static! {
    pub static ref BASE_URL: String = format!(
        "https://{}",
        env::var("DOMAIN").expect("DOMAIN must be set")
    );
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("DOMAIN must be set");
}

pub const GIT_REV: &str = include_str!(concat!(env!("OUT_DIR"), "/commit-info.txt"));

fn main() {
    // load environment variables fron .env
    dotenv().ok();

    // extract the database url from the environment and create the db connection pool
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_connection_pool =
        db::init_connection_pool(db_url).expect("Couldn't establish connection to database!");

    rocket::ignite()
        .mount("/", routes::ui::routes())
        .mount("/", routes::ap::routes())
        .manage(db_connection_pool) // store the db pool as Rocket managed state
                                    // (this lets us use the db::Connection guard)
        .launch();
}
