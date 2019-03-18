use dotenv::dotenv;
use slog::slog_o;
use std::env;

use rustodon::{app, db, init_logger};

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

    let app = app(db_connection_pool, rocket_logger);
    app.launch();
}
