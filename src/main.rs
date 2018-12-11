#![feature(nll, proc_macro_hygiene, decl_macro)]
#![recursion_limit = "128"]
// Allow some clippy lints that would otherwise warn on various Rocket-generated code.
// Unfortunately, this means we lose these lints on _our_ code, but it's a small price to pay
// for less line noise running `cargo clippy`.
#![allow(clippy::needless_pass_by_value, clippy::suspicious_else_formatting)]

extern crate ammonia;
extern crate chrono;
extern crate dotenv;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate resopt;
#[macro_use]
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
#[macro_use(slog_o, slog_warn)]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
#[macro_use]
extern crate slog_scope;
extern crate rocket_slog;

extern crate rustodon_database as db;

#[macro_use]
mod error;
mod activitypub;
mod routes;
mod transform;
mod util;

use dotenv::dotenv;
use rocket::config::Config;
use rocket_slog::SlogFairing;
use slog::Drain;
use std::env;

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

/// Loads the Rocket.toml config.
///
/// Basically replicating rocket::config::init() so that we can do Rocket::custom() to
/// disable logging, but still load Rocket.toml like Rocket::ignite() does.
fn rocket_load_config() -> Config {
    use rocket::config::ConfigError::{self, *};
    use rocket::config::LoggingLevel;
    use rocket::config::RocketConfig;

    let bail = |e: ConfigError| -> ! {
        use rocket::logger::{self, LoggingLevel};
        use std::process;

        logger::init(LoggingLevel::Debug);
        e.pretty_print();
        process::exit(1)
    };

    let config = RocketConfig::read().unwrap_or_else(|e| {
        match e {
            ParseError(..) | BadEntry(..) | BadEnv(..) | BadType(..) | Io(..) | BadFilePath(..)
            | BadEnvVal(..) | UnknownKey(..) | Missing(..) => bail(e),
            IoError => warn!("Failed reading Rocket.toml. Using defaults."),
            NotFound => { /* try using the default below */ },
        }

        RocketConfig::active_default().unwrap_or_else(|e| bail(e))
    });

    let mut config = config.active().clone();
    config.set_log_level(LoggingLevel::Off); // disable Rocket's built-in logging

    config
}

fn main() {
    // load environment variables fron .env
    dotenv().ok();

    // set up slog logger
    let log = init_logger();
    let rocket_logger = log.new(slog_o!());
    let _guard = slog_scope::set_global_logger(log);

    // extract the database url from the environment and create the db connection pool
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_connection_pool =
        db::init_connection_pool(db_url).expect("Couldn't establish connection to database!");

    rocket::custom(rocket_load_config()) // use our own config loading which turns off Rocket's built-in logging.
        .mount("/", routes::ui::routes())
        .mount("/", routes::ap::routes())
        .mount("/", routes::well_known::routes())
        .manage(db_connection_pool) // store the db pool as Rocket managed state
                                    // (this lets us use the db::Connection guard)
        .attach(SlogFairing::new(rocket_logger))
        .launch();
}
