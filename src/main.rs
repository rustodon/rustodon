#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_infer_schema;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;
extern crate pwhash;

mod db;

use std::env;
use dotenv::dotenv;
use diesel::prelude::*;


#[get("/")]
fn hello_world(db_conn: db::Connection) -> String {
    use db::schema::users::dsl::*;
    use db::models::User;

    let found_users = users.load::<User>(&*db_conn)
        .expect("error loading users");
    format!("users: {:?}", found_users)
}

fn main() {
    // load environment variables fron .env
    dotenv().ok();

    // extract the database url from the environment and create the db connection pool
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_connection_pool = db::init_connection_pool(db_url)
        .expect("Couldn't establish connection to database!");


    rocket::ignite()
        .mount("/", routes![hello_world])
        .manage(db_connection_pool) // store the db pool as Rocket managed state
                                    // (this lets us use the db::Connection guard)
        .launch();
}
