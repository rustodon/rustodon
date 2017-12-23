#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;

mod db;

use std::env;
use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_connection_pool = db::init_connection_pool(db_url)
        .expect("Couldn't establish connection to database!");


    rocket::ignite()
        .manage(db_connection_pool)
        .launch();
}
