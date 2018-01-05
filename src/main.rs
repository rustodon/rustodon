#![feature(plugin, nll, custom_derive)]
#![plugin(rocket_codegen)]
#![recursion_limit="128"]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_infer_schema;
#[macro_use] extern crate try_opt;
#[macro_use] extern crate lazy_static;
extern crate itertools;
extern crate rocket;
extern crate rocket_contrib;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;
extern crate pwhash;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;

mod error;
mod db;
mod routes;
mod activitypub;

use std::env;
use dotenv::dotenv;

lazy_static! {
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("DOMAIN must be set").to_owned();
}

fn main() {
    // load environment variables fron .env
    dotenv().ok();

    // extract the database url from the environment and create the db connection pool
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_connection_pool = db::init_connection_pool(db_url)
        .expect("Couldn't establish connection to database!");


    rocket::ignite()
        .mount("/", routes::ui::routes())
        .mount("/", routes::ap::routes())
        .manage(db_connection_pool) // store the db pool as Rocket managed state
                                    // (this lets us use the db::Connection guard)
        .launch();
}
