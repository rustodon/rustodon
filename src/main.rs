#![feature(plugin, nll, custom_derive, proc_macro_non_items)]
#![plugin(rocket_codegen)]
#![recursion_limit = "128"]

extern crate ammonia;
extern crate chrono;
extern crate dotenv;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate maud_htmlescape;
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
#[macro_use]
extern crate maplit;
extern crate posticle;
extern crate regex;
#[macro_use(slog_o, slog_info, slog_log, slog_record, slog_record_static, slog_b, slog_kv)]
extern crate slog;
extern crate slog_term;
extern crate slog_async;
#[macro_use]
extern crate slog_scope;

extern crate rustodon_database as db;

#[macro_use]
mod error;
mod activitypub;
mod routes;
mod transform;
mod util;

use dotenv::dotenv;
use std::env;
use slog::Drain;

#[macro_use]
extern crate askama;

lazy_static! {
    pub static ref BASE_URL: String = format!(
        "https://{}",
        env::var("DOMAIN").expect("DOMAIN must be set")
    );
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("DOMAIN must be set");
}

pub const GIT_REV: &str = include_str!(concat!(env!("OUT_DIR"), "/commit-info.txt"));

fn init_logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, slog_o!())
}

fn main() {
    // load environment variables fron .env
    dotenv().ok();

    // set up slog logger
    let log = init_logger();
    let _guard = slog_scope::set_global_logger(log);

    info!("hi!");

    // extract the database url from the environment and create the db connection pool
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_connection_pool =
        db::init_connection_pool(db_url).expect("Couldn't establish connection to database!");

    rocket::ignite()
        .mount("/", routes::ui::routes())
        .mount("/", routes::ap::routes())
        .mount("/", routes::well_known::routes())
        .manage(db_connection_pool) // store the db pool as Rocket managed state
                                    // (this lets us use the db::Connection guard)
        .launch();
}
