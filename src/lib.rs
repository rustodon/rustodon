#![feature(proc_macro_hygiene, decl_macro)]
#![recursion_limit = "128"]
// Allow some clippy lints that would otherwise warn on various Rocket-generated code.
// Unfortunately, this means we lose these lints on _our_ code, but it's a small price to pay
// for less line noise running `cargo clippy`.
#![allow(clippy::needless_pass_by_value, clippy::suspicious_else_formatting)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate validator_derive;
#[macro_use]
extern crate diesel_derive_enum;

mod activitypub;
pub mod crypto;
pub mod db;
mod error;
mod routes;
mod transform;
mod util;
mod workers;

use lazy_static::lazy_static;
use rocket::config::Config;
use rocket::Rocket;
use rocket_slog::SlogFairing;
use slog::Drain;
use slog::{slog_debug, slog_o, slog_warn};
use slog_scope::{debug, warn};
use std::env;

lazy_static! {
    pub static ref BASE_URL: String = format!(
        "https://{}",
        env::var("DOMAIN").expect("DOMAIN must be set")
    );
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("DOMAIN must be set");
}

pub const GIT_REV: &str = include_str!(concat!(env!("OUT_DIR"), "/commit-info.txt"));

pub fn init_logger() -> slog::Logger {
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

pub fn app(db: db::Pool, logger: slog::Logger) -> Rocket {
    // initialize the worker queues
    debug!("starting worker queues");
    workers::init(db.clone());

    {
        use diesel::prelude::*;
        let conn = db.get().unwrap();

        for i in 0..1 {
            use crate::db::models::NewJobRecord;
            let r = NewJobRecord::on_queue(
                workers::TestJob {
                    msg: format!("bengis:{}", i),
                },
                "default_queue",
            )
            .unwrap();
            debug!("injecting test job");
            diesel::insert_into(db::schema::jobs::table)
                .values(&r)
                .execute(&conn)
                .unwrap();
            debug!("done injecting test job");
        }
        // println!("{:?}", r);
    }

    rocket::custom(rocket_load_config()) // use our own config loading which turns off Rocket's built-in logging.
        .mount("/", routes::ui::routes())
        .mount("/", routes::ap::routes())
        .mount("/", routes::well_known::routes())
        .manage(db) // store the db pool as Rocket managed state
                    // (this lets us use the db::Connection guard)
        .attach(SlogFairing::new(logger))
}
